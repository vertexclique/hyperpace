//! The owner thread: one per connected device, the only thing that ever touches its
//! [`Transport`].
//!
//! [`spawn`] starts the thread and returns a [`DeviceHandle`] to it. The thread runs the connect
//! sequence (protocol reference section 9.1: handshake, then the base settings walk), then
//! alternates between servicing queued `OwnerCommand`s, polling the battery every 5 seconds,
//! and listening for unsolicited pushes, all through one transport it owns exclusively. Requests
//! are matched to replies by command byte; a `StatusChanged` (command 10) push is dispatched as a
//! [`DeviceEvent`] and never mistaken for a reply, whichever step of the loop below is waiting.

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use hyperpace_protocol::{
    Command, FRAME_LEN, Frame, Shadow, Transport, TransportError, request, response, scalar_pair,
};

use crate::error::DeviceError;
use crate::event::{Access, DeviceEvent};
use crate::handle::{DeviceHandle, OwnerCommand};

/// Bytes the base settings walk covers at connect: `js(0, 256)`, protocol reference section 4.9.
const WALK_LEN: u16 = 256;
/// Bytes per chunk during the walk; matches [`hyperpace_protocol::PAYLOAD_LEN`] (pinned together
/// by a unit test below so the two cannot drift).
const WALK_CHUNK: u16 = 10;
/// Battery poll interval once connected, protocol reference section 9.6.
const BATTERY_INTERVAL: Duration = Duration::from_secs(5);
/// Overall deadline for one handshake or walk-step exchange, including its own retries.
const CONNECT_TIMEOUT: Duration = Duration::from_millis(600);
/// Overall deadline for one battery-poll exchange (the online probe or the battery read).
const POLL_TIMEOUT: Duration = Duration::from_millis(600);
/// Overall deadline for one write's acknowledgement.
const WRITE_TIMEOUT: Duration = Duration::from_millis(600);
/// How long the idle branch waits for a spontaneous push before checking commands again.
const IDLE_SLICE: Duration = Duration::from_millis(100);
/// How long a single receive slice inside [`send_and_await`] waits, so a push arriving mid-wait
/// is picked up promptly instead of after the whole exchange's deadline.
const RECV_SLICE: Duration = Duration::from_millis(50);
/// Sends attempted per exchange before [`send_and_await`] gives up early on a `Timeout`
/// (retries are still bounded overall by the caller's own deadline).
const MAX_ATTEMPTS: u32 = 3;

/// Where the owner thread is in the connect sequence.
#[derive(Clone, Copy)]
enum ConnectPhase {
    /// Waiting for a handshake reply carrying the device's identity.
    Handshake,
    /// Walking the base settings block, `offset` bytes in.
    Walk { offset: u16 },
    /// Connected: servicing commands, polling the battery, and listening for pushes.
    Ready,
}

/// Why the owner thread's main loop stopped.
enum ServiceExit {
    /// The transport reported the device unreachable.
    Disconnected,
}

/// Everything the owner thread carries between loop iterations.
struct OwnerState {
    phase: ConnectPhase,
    subscribers: Vec<mpsc::Sender<DeviceEvent>>,
    shadow: Shadow,
    access: Access,
    last_battery_poll: Option<Instant>,
}

/// Send `event` to every live subscriber, dropping any whose receiver is gone.
fn broadcast(subscribers: &mut Vec<mpsc::Sender<DeviceEvent>>, event: DeviceEvent) {
    subscribers.retain(|subscriber| subscriber.send(event).is_ok());
}

/// Decode `bytes`; if it is a `StatusChanged` push, dispatch it as an event and return `None`, so
/// a push can never be mistaken for the reply a caller is waiting on. Any other frame is handed
/// back for the caller to match against what it expects.
fn handle_incoming(
    bytes: &[u8; FRAME_LEN],
    subscribers: &mut Vec<mpsc::Sender<DeviceEvent>>,
) -> Option<Frame> {
    let frame = Frame::decode(bytes).ok()?;
    if frame.command == 10 {
        if let Ok(changed) = response::status_changed(&frame) {
            broadcast(subscribers, DeviceEvent::Changed(changed));
        }
        None
    } else {
        Some(frame)
    }
}

/// Send `frame` and wait up to `timeout` for a reply whose command byte matches `command`,
/// resending up to [`MAX_ATTEMPTS`] times. A push encountered while waiting is dispatched and
/// does not consume an attempt or shrink the deadline (protocol reference section 4.11: a
/// reimplementation must reject an unsolicited frame instead of counting it as a failed attempt).
fn send_and_await(
    transport: &mut dyn Transport,
    subscribers: &mut Vec<mpsc::Sender<DeviceEvent>>,
    command: u8,
    frame: &Frame,
    timeout: Duration,
) -> Result<Frame, DeviceError> {
    let deadline = Instant::now() + timeout;
    let resend_every = timeout / MAX_ATTEMPTS;
    let mut next_send = Instant::now();
    let mut attempts = 0u32;

    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(DeviceError::Timeout);
        }
        if attempts < MAX_ATTEMPTS && now >= next_send {
            transport.send(&frame.encode())?;
            attempts += 1;
            next_send = now + resend_every;
        }

        let remaining = deadline.saturating_duration_since(now);
        let slice = RECV_SLICE.min(remaining);
        if let Some(bytes) = transport.recv(slice)?
            && let Some(reply) = handle_incoming(&bytes, subscribers)
            && reply.command == command
        {
            return Ok(reply);
        }
    }
}

/// Command bytes that mutate device state (protocol reference section 5: `DongleEnterPair`,
/// `WriteFlashData`, `ClearSetting`, `SetCurrentConfig`, `SetLongRangeMode`, `SetDongleLight`),
/// gated by [`Access::ReadOnly`] when they arrive through [`DeviceHandle::request`].
fn is_write_command(command: u8) -> bool {
    matches!(command, 5 | 7 | 9 | 15 | 22 | 24)
}

/// Advance the handshake step. Stays in [`ConnectPhase::Handshake`] for the next tick on a
/// timeout or transient I/O error; only a reported disconnect ends the thread.
fn advance_handshake(
    transport: &mut dyn Transport,
    state: &mut OwnerState,
) -> Result<(), ServiceExit> {
    // The device never compares the challenge bytes back (protocol reference section 3.2), so a
    // fixed nonce serves exactly as well as a random one.
    let frame = request::handshake([0; 4]);
    match send_and_await(
        transport,
        &mut state.subscribers,
        Command::Handshake as u8,
        &frame,
        CONNECT_TIMEOUT,
    ) {
        Ok(reply) => {
            if let Ok(identity) = response::identity(&reply) {
                broadcast(&mut state.subscribers, DeviceEvent::Connected(identity));
                state.phase = ConnectPhase::Walk { offset: 0 };
            }
            Ok(())
        }
        Err(DeviceError::Disconnected) => Err(ServiceExit::Disconnected),
        Err(_) => Ok(()),
    }
}

/// Advance the base settings walk by one chunk, or finish it once `offset` reaches
/// [`WALK_LEN`].
fn advance_walk(
    transport: &mut dyn Transport,
    state: &mut OwnerState,
    offset: u16,
) -> Result<(), ServiceExit> {
    if offset >= WALK_LEN {
        state.phase = ConnectPhase::Ready;
        // Force an immediate battery poll now that the walk is done.
        state.last_battery_poll = None;
        return Ok(());
    }

    let len = (WALK_LEN - offset).min(WALK_CHUNK);
    let frame = request::read_flash(offset, len as u8);
    match send_and_await(
        transport,
        &mut state.subscribers,
        Command::ReadFlash as u8,
        &frame,
        CONNECT_TIMEOUT,
    ) {
        Ok(reply) => {
            state
                .shadow
                .apply_read(offset, &reply.payload[..usize::from(len)]);
            state.phase = ConnectPhase::Walk {
                offset: offset + len,
            };
            Ok(())
        }
        Err(DeviceError::Disconnected) => Err(ServiceExit::Disconnected),
        Err(_) => Ok(()),
    }
}

/// One battery-poll tick: `DeviceOnLine` then `BatteryLevel`, protocol reference section 9.6.
///
/// A negative or failed online probe emits [`DeviceEvent::Offline`] and returns the owner thread
/// to [`ConnectPhase::Handshake`], matching the documented behavior of restarting the pre-connect
/// poll; a [`DeviceEvent::Connected`] follows once the device answers again.
fn poll_battery(transport: &mut dyn Transport, state: &mut OwnerState) -> Result<(), ServiceExit> {
    let online_reply = send_and_await(
        transport,
        &mut state.subscribers,
        Command::Online as u8,
        &Command::Online.request(),
        POLL_TIMEOUT,
    );
    let online = match online_reply {
        Ok(reply) => response::online(&reply).is_ok_and(|(online, _address)| online),
        Err(DeviceError::Disconnected) => return Err(ServiceExit::Disconnected),
        Err(_) => false,
    };
    if !online {
        broadcast(&mut state.subscribers, DeviceEvent::Offline);
        state.phase = ConnectPhase::Handshake;
        state.last_battery_poll = None;
        return Ok(());
    }

    match send_and_await(
        transport,
        &mut state.subscribers,
        Command::Battery as u8,
        &Command::Battery.request(),
        POLL_TIMEOUT,
    ) {
        Ok(reply) => {
            if let Ok(battery) = response::battery(&reply) {
                broadcast(&mut state.subscribers, DeviceEvent::Battery(battery));
            }
        }
        Err(DeviceError::Disconnected) => return Err(ServiceExit::Disconnected),
        Err(_) => {} // transient; retried on the next interval
    }
    state.last_battery_poll = Some(Instant::now());
    Ok(())
}

/// Write `data` at `address`, split into [`hyperpace_protocol::PAYLOAD_LEN`]-byte
/// `WriteFlashData` frames (contract note on `request::write_flash`: one call is one frame, and
/// splitting a larger write is this crate's job). Stops at the first chunk that fails; the shadow
/// mirrors every chunk that already succeeded.
fn write_block(
    transport: &mut dyn Transport,
    subscribers: &mut Vec<mpsc::Sender<DeviceEvent>>,
    shadow: &mut Shadow,
    address: u16,
    data: &[u8],
) -> Result<(), DeviceError> {
    for (index, chunk) in data.chunks(hyperpace_protocol::PAYLOAD_LEN).enumerate() {
        let chunk_offset =
            u16::try_from(index * hyperpace_protocol::PAYLOAD_LEN).unwrap_or(u16::MAX);
        let chunk_address = address.saturating_add(chunk_offset);
        let frame = request::write_flash(chunk_address, chunk)?;
        send_and_await(
            transport,
            subscribers,
            Command::WriteFlash as u8,
            &frame,
            WRITE_TIMEOUT,
        )?;
        shadow.apply_read(chunk_address, chunk);
    }
    Ok(())
}

/// Read `len` bytes starting at `address`, split into [`hyperpace_protocol::PAYLOAD_LEN`]-byte
/// `ReadFlashData` requests (the read-side mirror of [`write_block`]). Stops at the first chunk
/// that fails; bytes already read are still mirrored into `shadow` even though the error is what
/// reaches the caller, matching `write_block`'s own "stops at the first chunk that fails" note.
fn read_block(
    transport: &mut dyn Transport,
    subscribers: &mut Vec<mpsc::Sender<DeviceEvent>>,
    shadow: &mut Shadow,
    address: u16,
    len: usize,
) -> Result<Vec<u8>, DeviceError> {
    let mut out = Vec::with_capacity(len);
    let mut offset = 0usize;
    while offset < len {
        let chunk_len = (len - offset).min(hyperpace_protocol::PAYLOAD_LEN);
        let chunk_offset = u16::try_from(offset).unwrap_or(u16::MAX);
        let chunk_address = address.saturating_add(chunk_offset);
        let chunk_len_byte = u8::try_from(chunk_len).unwrap_or(u8::MAX);
        let frame = request::read_flash(chunk_address, chunk_len_byte);
        let reply = send_and_await(
            transport,
            subscribers,
            Command::ReadFlash as u8,
            &frame,
            WRITE_TIMEOUT,
        )?;
        let data = &reply.payload[..chunk_len];
        shadow.apply_read(chunk_address, data);
        out.extend_from_slice(data);
        offset += chunk_len;
    }
    Ok(out)
}

/// Service an [`OwnerCommand::Request`]: send it, and when it was a `FactoryReset` (9) the device
/// actually decoded (not status-1), discard the cached shadow and restart the base settings walk
/// (protocol reference section 10.3: a factory reset is followed by a full flash re-read;
/// `advance_walk` repopulates the shadow from the now-reset device on the next ticks).
fn service_request(
    transport: &mut dyn Transport,
    state: &mut OwnerState,
    frame: Frame,
    timeout: Duration,
    reply: &mpsc::Sender<Result<Frame, DeviceError>>,
) -> Result<(), ServiceExit> {
    if state.access == Access::ReadOnly && is_write_command(frame.command) {
        let _ = reply.send(Err(DeviceError::ReadOnly));
        return Ok(());
    }
    let command_byte = frame.command;
    let result = send_and_await(
        transport,
        &mut state.subscribers,
        command_byte,
        &frame,
        timeout,
    );
    if let Ok(reply_frame) = &result
        && command_byte == Command::FactoryReset as u8
        && reply_frame.status() != hyperpace_protocol::Status::Unsupported
    {
        state.shadow = Shadow::new();
        state.phase = ConnectPhase::Walk { offset: 0 };
    }
    let disconnected = matches!(result, Err(DeviceError::Disconnected));
    let _ = reply.send(result);
    if disconnected {
        return Err(ServiceExit::Disconnected);
    }
    Ok(())
}

/// Service an [`OwnerCommand::ReadBlock`].
fn service_read_block(
    transport: &mut dyn Transport,
    state: &mut OwnerState,
    address: u16,
    len: usize,
    reply: &mpsc::Sender<Result<Vec<u8>, DeviceError>>,
) -> Result<(), ServiceExit> {
    let result = read_block(
        transport,
        &mut state.subscribers,
        &mut state.shadow,
        address,
        len,
    );
    let disconnected = matches!(result, Err(DeviceError::Disconnected));
    let _ = reply.send(result);
    if disconnected {
        return Err(ServiceExit::Disconnected);
    }
    Ok(())
}

/// Service one queued command to completion. Returns [`ServiceExit::Disconnected`] once the
/// transport that answered it (if any) reported the device unreachable, after the caller waiting
/// on its reply channel has already been told.
fn handle_command(
    command: OwnerCommand,
    transport: &mut dyn Transport,
    state: &mut OwnerState,
) -> Result<(), ServiceExit> {
    match command {
        OwnerCommand::Subscribe { reply } => {
            let (tx, rx) = mpsc::channel();
            state.subscribers.push(tx);
            let _ = reply.send(rx);
        }
        OwnerCommand::Request {
            frame,
            timeout,
            reply,
        } => service_request(transport, state, frame, timeout, &reply)?,
        OwnerCommand::ReadSettings { reply } => {
            let _ = reply.send(Ok(state.shadow.clone()));
        }
        OwnerCommand::ReadBlock {
            address,
            len,
            reply,
        } => service_read_block(transport, state, address, len, &reply)?,
        OwnerCommand::WriteScalar {
            address,
            value,
            reply,
        } => {
            if state.access == Access::ReadOnly {
                let _ = reply.send(Err(DeviceError::ReadOnly));
                return Ok(());
            }
            let frame = request::set_scalar(address, value);
            let result = send_and_await(
                transport,
                &mut state.subscribers,
                Command::WriteFlash as u8,
                &frame,
                WRITE_TIMEOUT,
            );
            let disconnected = matches!(result, Err(DeviceError::Disconnected));
            match result {
                Ok(_) => {
                    state.shadow.apply_read(address, &scalar_pair(value));
                    let _ = reply.send(Ok(()));
                }
                Err(error) => {
                    let _ = reply.send(Err(error));
                }
            }
            if disconnected {
                return Err(ServiceExit::Disconnected);
            }
        }
        OwnerCommand::WriteBlock {
            address,
            data,
            reply,
        } => {
            if state.access == Access::ReadOnly {
                let _ = reply.send(Err(DeviceError::ReadOnly));
                return Ok(());
            }
            let result = write_block(
                transport,
                &mut state.subscribers,
                &mut state.shadow,
                address,
                &data,
            );
            let disconnected = matches!(result, Err(DeviceError::Disconnected));
            let _ = reply.send(result);
            if disconnected {
                return Err(ServiceExit::Disconnected);
            }
        }
    }
    Ok(())
}

/// The owner thread's body: connect, then alternate commands, the battery timer, and idle
/// listening, until every [`DeviceHandle`] clone is dropped or the transport disconnects.
fn run(mut transport: Box<dyn Transport>, access: Access, commands: &Receiver<OwnerCommand>) {
    let mut state = OwnerState {
        phase: ConnectPhase::Handshake,
        subscribers: Vec::new(),
        shadow: Shadow::new(),
        access,
        last_battery_poll: None,
    };

    loop {
        match commands.try_recv() {
            Ok(command) => {
                if handle_command(command, transport.as_mut(), &mut state).is_err() {
                    broadcast(&mut state.subscribers, DeviceEvent::Disconnected);
                    return;
                }
                continue;
            }
            Err(TryRecvError::Disconnected) => return,
            Err(TryRecvError::Empty) => {}
        }

        let outcome = match state.phase {
            ConnectPhase::Handshake => advance_handshake(transport.as_mut(), &mut state),
            ConnectPhase::Walk { offset } => advance_walk(transport.as_mut(), &mut state, offset),
            ConnectPhase::Ready
                if state
                    .last_battery_poll
                    .is_none_or(|at| at.elapsed() >= BATTERY_INTERVAL) =>
            {
                poll_battery(transport.as_mut(), &mut state)
            }
            ConnectPhase::Ready => match transport.recv(IDLE_SLICE) {
                Ok(Some(bytes)) => {
                    handle_incoming(&bytes, &mut state.subscribers);
                    Ok(())
                }
                Ok(None) | Err(TransportError::Io(_)) => Ok(()),
                Err(TransportError::Disconnected) => Err(ServiceExit::Disconnected),
            },
        };

        if outcome.is_err() {
            broadcast(&mut state.subscribers, DeviceEvent::Disconnected);
            return;
        }
    }
}

/// Spawn the owner thread for `transport` and return a handle to it.
///
/// The thread starts the connect sequence immediately, polls the battery every 5 seconds once
/// connected, and exits the moment every [`DeviceHandle`] clone is dropped or the transport
/// reports the device unreachable.
///
/// # Errors
///
/// Returns [`DeviceError::Io`] when the operating system could not start the thread.
pub fn spawn(transport: Box<dyn Transport>, access: Access) -> Result<DeviceHandle, DeviceError> {
    let (commands, rx) = mpsc::channel();
    thread::Builder::new()
        .name("hyperpace-device-owner".to_owned())
        .spawn(move || {
            let commands = rx;
            run(transport, access, &commands);
        })
        .map_err(|error| DeviceError::Io(error.to_string()))?;
    Ok(DeviceHandle { commands, access })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use hyperpace_protocol::{Battery, DeviceIdentity, LinkType, Status, StatusChanged, offset};

    use super::*;
    use crate::sim::SimTransport;

    const TEST_TIMEOUT: Duration = Duration::from_secs(2);

    fn expect_event(events: &mpsc::Receiver<DeviceEvent>) -> DeviceEvent {
        events.recv_timeout(TEST_TIMEOUT).unwrap()
    }

    #[test]
    fn walk_chunk_matches_the_protocol_payload_length() {
        assert_eq!(usize::from(WALK_CHUNK), hyperpace_protocol::PAYLOAD_LEN);
    }

    #[test]
    fn connect_emits_the_configured_identity_then_a_battery_reading() {
        let (transport, controller) = SimTransport::new(62, 1, 4);
        controller.set_battery(73, true, 0x0e74);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        let events = handle.events();

        assert_eq!(
            expect_event(&events),
            DeviceEvent::Connected(DeviceIdentity {
                cid: 62,
                mid: 1,
                link: LinkType::Wireless2k,
            })
        );
        assert_eq!(
            expect_event(&events),
            DeviceEvent::Battery(Battery {
                percent: 73,
                charging: true,
                millivolts: 0x0e74,
            })
        );
    }

    #[test]
    fn read_settings_reflects_the_base_walk() {
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        let events = handle.events();
        assert!(matches!(expect_event(&events), DeviceEvent::Connected(_)));
        assert!(matches!(expect_event(&events), DeviceEvent::Battery(_)));

        let shadow = handle.read_settings().unwrap();
        // The walk covers offsets 0..256; nothing past it was read, so it stays erased flash.
        assert_eq!(shadow.scalar(300), 0xff);
    }

    #[test]
    fn write_scalar_is_refused_under_read_only_access() {
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        assert_eq!(
            handle.write_scalar(offset::DEBOUNCE, 8),
            Err(DeviceError::ReadOnly)
        );
    }

    #[test]
    fn a_write_shaped_request_is_also_refused_under_read_only_access() {
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        let frame = request::set_scalar(offset::DEBOUNCE, 8);
        assert_eq!(
            handle.request(frame, Duration::from_millis(200)),
            Err(DeviceError::ReadOnly)
        );
    }

    #[test]
    fn write_scalar_succeeds_under_read_write_access_and_updates_the_shadow() {
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadWrite).unwrap();
        handle.write_scalar(offset::DEBOUNCE, 8).unwrap();
        let shadow = handle.read_settings().unwrap();
        assert_eq!(shadow.scalar(offset::DEBOUNCE), 8);
    }

    #[test]
    fn write_block_splits_into_payload_sized_chunks_and_round_trips() {
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadWrite).unwrap();
        let data = (0u8..25).collect::<Vec<_>>();
        handle.write_block(1000, &data).unwrap();
        let shadow = handle.read_settings().unwrap();
        for (i, byte) in data.iter().enumerate() {
            let address = 1000 + u16::try_from(i).unwrap();
            assert_eq!(shadow.scalar(address), *byte);
        }
    }

    #[test]
    fn a_push_is_dispatched_as_an_event_and_never_consumed_as_a_reply() {
        let (transport, controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        let events = handle.events();
        assert!(matches!(expect_event(&events), DeviceEvent::Connected(_)));
        assert!(matches!(expect_event(&events), DeviceEvent::Battery(_)));

        let mut push = Frame::command(10);
        push.payload[0] = 0x01; // dpi bit
        controller.push(push);

        assert_eq!(
            expect_event(&events),
            DeviceEvent::Changed(StatusChanged {
                dpi: true,
                polling: false,
                profile: false,
                dpi_indicator: false,
                lighting: false,
                battery: false,
            })
        );

        // A request issued right after still gets its own real reply, not the push re-delivered.
        let reply = handle
            .request(Command::Online.request(), Duration::from_millis(500))
            .unwrap();
        assert_eq!(reply.command, 3);
    }

    #[test]
    fn going_offline_then_online_reconnects_and_reemits_connected() {
        let (transport, controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        let events = handle.events();
        assert!(matches!(expect_event(&events), DeviceEvent::Connected(_)));
        assert!(matches!(expect_event(&events), DeviceEvent::Battery(_)));

        controller.set_online(false);

        // The next battery poll (every 5 real seconds) is what notices; give it real headroom
        // instead of a single timeout, per the deadline-plus-backoff rule for a genuine timer.
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut offline = false;
        while Instant::now() < deadline {
            if let Ok(DeviceEvent::Offline) = events.recv_timeout(Duration::from_millis(200)) {
                offline = true;
                break;
            }
        }
        assert!(
            offline,
            "expected Offline within 8 seconds of the next battery poll"
        );

        controller.set_online(true);
        assert!(matches!(expect_event(&events), DeviceEvent::Connected(_)));
    }

    #[test]
    fn a_disconnected_transport_ends_the_thread_and_emits_disconnected() {
        let (transport, controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        let events = handle.events();
        assert!(matches!(expect_event(&events), DeviceEvent::Connected(_)));
        assert!(matches!(expect_event(&events), DeviceEvent::Battery(_)));

        controller.close();
        assert_eq!(expect_event(&events), DeviceEvent::Disconnected);
        assert_eq!(
            handle.request(Command::Online.request(), Duration::from_millis(200)),
            Err(DeviceError::Disconnected)
        );
    }

    #[test]
    fn an_unsupported_status_reply_still_reaches_the_caller() {
        let (transport, controller) = SimTransport::new(62, 1, 0);
        controller.set_unsupported(23, true);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        let reply = handle
            .request(Command::GetLongRange.request(), Duration::from_millis(500))
            .unwrap();
        assert_eq!(reply.status(), Status::Unsupported);
    }

    #[test]
    fn read_block_reaches_flash_the_connect_walk_never_covers() {
        // The keystroke region starts at offset 256 (`offset::KEYSTROKE`), past WALK_LEN's
        // 256-byte span, so read_settings alone can never see it; a fresh slot reads as erased
        // flash, and a write through it round-trips.
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadWrite).unwrap();

        let erased = handle.read_block(offset::KEYSTROKE, 32).unwrap();
        assert_eq!(erased, vec![0xffu8; 32]);

        handle
            .write_block(offset::KEYSTROKE, &[6, 0x81, 4, 0, 0x41, 4, 0, 0xaa])
            .unwrap();
        let written = handle.read_block(offset::KEYSTROKE, 8).unwrap();
        assert_eq!(written, vec![6, 0x81, 4, 0, 0x41, 4, 0, 0xaa]);

        // The read is also mirrored into the shadow, so a later read_settings sees it too.
        let shadow = handle.read_settings().unwrap();
        assert_eq!(shadow.scalar(offset::KEYSTROKE), 6);
    }

    #[test]
    fn factory_reset_through_the_device_layer_erases_and_re_walks_the_shadow() {
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadWrite).unwrap();
        handle.write_scalar(offset::DEBOUNCE, 8).unwrap();
        assert_eq!(handle.read_settings().unwrap().scalar(offset::DEBOUNCE), 8);

        handle
            .request(Command::FactoryReset.request(), TEST_TIMEOUT)
            .unwrap();

        // The re-walk that repopulates the shadow from the now-reset device runs on the owner
        // thread's own idle ticks; poll on a deadline rather than a fixed sleep.
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut erased = false;
        while Instant::now() < deadline {
            if handle.read_settings().unwrap().scalar(offset::DEBOUNCE) == 0xff {
                erased = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            erased,
            "the shadow must read back erased after a reset re-walk"
        );
    }

    #[test]
    fn pairing_progresses_through_its_phases_to_success_via_the_device_layer() {
        let (transport, controller) = SimTransport::new(62, 1, 0);
        controller.set_pair_outcome(true, 2);
        let handle = spawn(Box::new(transport), Access::ReadWrite).unwrap();

        handle
            .request(Command::EnterPair.request(), TEST_TIMEOUT)
            .unwrap();
        let first = response::pair_state(
            &handle
                .request(Command::PairState.request(), TEST_TIMEOUT)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(first.state, hyperpace_protocol::PairPhase::Pairing);

        let second = response::pair_state(
            &handle
                .request(Command::PairState.request(), TEST_TIMEOUT)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(second.state, hyperpace_protocol::PairPhase::Succeeded);
    }

    #[test]
    fn pairing_can_fail_via_the_device_layer() {
        let (transport, controller) = SimTransport::new(62, 1, 0);
        controller.set_pair_outcome(false, 1);
        let handle = spawn(Box::new(transport), Access::ReadWrite).unwrap();

        handle
            .request(Command::EnterPair.request(), TEST_TIMEOUT)
            .unwrap();
        let outcome = response::pair_state(
            &handle
                .request(Command::PairState.request(), TEST_TIMEOUT)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(outcome.state, hyperpace_protocol::PairPhase::Failed);
    }

    #[test]
    fn receiver_light_round_trips_via_the_device_layer() {
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadWrite).unwrap();
        let light = hyperpace_protocol::ReceiverLight {
            mode: 3,
            color: (10, 20, 30),
            speed: 4,
            brightness: 6,
            time: 2,
        };
        handle
            .request(request::receiver_light(&light), TEST_TIMEOUT)
            .unwrap();

        let reply = handle
            .request(Command::GetReceiverLight.request(), TEST_TIMEOUT)
            .unwrap();
        assert_eq!(&reply.payload[..7], &[3, 10, 20, 30, 4, 6, 2]);
    }

    #[test]
    fn a_model_lacking_a_feature_reports_it_unsupported_via_the_device_layer() {
        let (transport, controller) = SimTransport::new(62, 1, 0);
        controller.set_unsupported(9, true); // this model has no factory reset
        let handle = spawn(Box::new(transport), Access::ReadWrite).unwrap();

        let reply = handle
            .request(Command::FactoryReset.request(), TEST_TIMEOUT)
            .unwrap();
        assert_eq!(reply.status(), Status::Unsupported);
    }

    #[test]
    fn dropping_every_handle_ends_the_owner_thread() {
        let (transport, _controller) = SimTransport::new(62, 1, 0);
        let handle = spawn(Box::new(transport), Access::ReadOnly).unwrap();
        let events = handle.events();
        assert!(matches!(expect_event(&events), DeviceEvent::Connected(_)));
        drop(handle);

        let deadline = Instant::now() + TEST_TIMEOUT;
        let mut closed = false;
        while Instant::now() < deadline {
            if let Err(mpsc::RecvTimeoutError::Disconnected) =
                events.recv_timeout(Duration::from_millis(50))
            {
                closed = true;
                break;
            }
        }
        assert!(
            closed,
            "owner thread did not exit after every handle was dropped"
        );
    }
}
