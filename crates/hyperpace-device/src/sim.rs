//! The device simulator every test in this crate, and every test elsewhere in the workspace that
//! needs a device, drives instead of real hardware.
//!
//! Reproduces the quirks `docs/research/mouse-protocol-v2.md` documents rather than an idealized
//! device: the `BatteryLevel` reply's declared length understates its own four-byte payload
//! (section 5), a command the simulator was not told to support answers with status 1 (section
//! 5's status-1 handling), and a test can queue an unsolicited `StatusChanged` push that arrives
//! on the wire exactly like the real device's would (section 4.7).

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use hyperpace_protocol::{FRAME_LEN, Frame, PAYLOAD_LEN, Transport, TransportError, settings};

/// How long [`SimTransport::recv`] sleeps when nothing is waiting, so an idle owner thread's poll
/// loop does not busy-spin. The simulator computes every reply synchronously, so this never
/// delays an actual reply, only the case where there is genuinely nothing to deliver yet.
const IDLE_STEP: Duration = Duration::from_millis(5);

/// State shared between a [`SimTransport`] moved into an owner thread and every [`SimController`]
/// clone a test keeps. Guarded by a plain [`Mutex`]: this is test and development infrastructure,
/// never a production hot path, so the lock is never contended enough to matter.
struct Inner {
    cid: u8,
    mid: u8,
    link_byte: u8,
    online: bool,
    battery: (u8, bool, u16),
    profile: u8,
    version: (u8, u8),
    receiver_version: (u8, u8),
    long_range: bool,
    flash: Vec<u8>,
    unsupported: HashSet<u8>,
    pending_pushes: VecDeque<Frame>,
    closed: bool,
}

fn lock(inner: &Mutex<Inner>) -> std::sync::MutexGuard<'_, Inner> {
    inner.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Compute the reply to one decoded request, per `docs/research/mouse-protocol-v2.md` section 5.
fn handle_request(inner: &mut Inner, request: &Frame) -> Frame {
    if inner.unsupported.contains(&request.command) {
        let mut reply = Frame::command(request.command);
        reply.status = 1;
        return reply;
    }

    match request.command {
        1 => {
            // EncryptionData: cid, mid and link land at payload bytes 4..7 of the reply, not
            // where the request's own challenge bytes sat (section 3.2).
            let mut reply = Frame::command(1);
            reply.payload[4] = inner.cid;
            reply.payload[5] = inner.mid;
            reply.payload[6] = inner.link_byte;
            reply.length = 7;
            reply
        }
        3 => {
            let mut reply = Frame::command(3);
            reply.payload[0] = u8::from(inner.online);
            reply.length = 4;
            reply
        }
        4 => {
            // BatteryLevel: the declared quirk. The real payload is 4 bytes (percent, charging,
            // millivolts big endian) but the device's own length byte understates it; neither
            // vendor host ever reads byte 4 here, and this simulator reproduces that mismatch
            // rather than a device that would make the bug invisible.
            let mut reply = Frame::command(4);
            reply.payload[0] = inner.battery.0;
            reply.payload[1] = u8::from(inner.battery.1);
            let millivolts = inner.battery.2.to_be_bytes();
            reply.payload[2] = millivolts[0];
            reply.payload[3] = millivolts[1];
            reply.length = 1;
            reply
        }
        7 => {
            let address = usize::from(request.address);
            let data = request.declared_payload();
            if address < inner.flash.len() {
                let end = (address + data.len()).min(inner.flash.len());
                let written = end - address;
                inner.flash[address..end].copy_from_slice(&data[..written]);
            }
            let mut reply = Frame::command(7);
            reply.address = request.address;
            reply.length = request.length;
            reply
        }
        8 => {
            let address = usize::from(request.address);
            let len = usize::from(request.length).min(PAYLOAD_LEN);
            let mut reply = Frame::command(8);
            reply.address = request.address;
            reply.length = request.length;
            if address < inner.flash.len() {
                let end = (address + len).min(inner.flash.len());
                let read = end - address;
                reply.payload[..read].copy_from_slice(&inner.flash[address..end]);
                reply.payload[read..len].fill(0xff);
            } else {
                reply.payload[..len].fill(0xff);
            }
            reply
        }
        14 => {
            let mut reply = Frame::command(14);
            reply.payload[0] = inner.profile;
            reply.length = 1;
            reply
        }
        15 => {
            inner.profile = request.payload[0];
            Frame::command(15)
        }
        18 => {
            let mut reply = Frame::command(18);
            reply.payload[0] = inner.version.0;
            reply.payload[1] = inner.version.1;
            reply.length = 2;
            reply
        }
        22 => {
            inner.long_range = request.payload[0] != 0;
            Frame::command(22)
        }
        23 => {
            let mut reply = Frame::command(23);
            reply.payload[0] = u8::from(inner.long_range);
            reply.length = 1;
            reply
        }
        29 => {
            let mut reply = Frame::command(29);
            reply.payload[0] = inner.receiver_version.0;
            reply.payload[1] = inner.receiver_version.1;
            reply.length = 2;
            reply
        }
        // EnterPair, GetPairState, FactoryReset, SetReceiverLight, GetReceiverLight: acknowledged
        // with a bare status-0 echo. No test in this crate depends on their payload shape yet;
        // add a case here (with a test) before one does.
        5 | 6 | 9 | 24 | 25 => Frame::command(request.command),
        other => {
            let mut reply = Frame::command(other);
            reply.status = 1;
            reply
        }
    }
}

/// The device simulator. Implements [`Transport`], so it plugs directly into [`crate::spawn`] in
/// place of [`crate::HidTransport`].
pub struct SimTransport {
    inner: Arc<Mutex<Inner>>,
    outbox: VecDeque<[u8; FRAME_LEN]>,
}

impl SimTransport {
    /// A simulated device reporting `cid`/`mid`/`link_byte` at its handshake (section 3.4 for what
    /// `link_byte` means), online, at 80% battery, and supporting every command this simulator
    /// models. Paired with the [`SimController`] a test uses to change any of that afterward.
    #[must_use]
    pub fn new(cid: u8, mid: u8, link_byte: u8) -> (Self, SimController) {
        let inner = Arc::new(Mutex::new(Inner {
            cid,
            mid,
            link_byte,
            online: true,
            battery: (80, false, 3900),
            profile: 0,
            version: (2, 0x16),
            receiver_version: (1, 0x00),
            long_range: false,
            flash: vec![0xff; settings::SHADOW_LEN],
            unsupported: HashSet::new(),
            pending_pushes: VecDeque::new(),
            closed: false,
        }));
        let transport = Self {
            inner: Arc::clone(&inner),
            outbox: VecDeque::new(),
        };
        (transport, SimController { inner })
    }
}

impl Transport for SimTransport {
    fn send(&mut self, frame: &[u8; FRAME_LEN]) -> Result<(), TransportError> {
        let mut inner = lock(&self.inner);
        if inner.closed {
            return Err(TransportError::Disconnected);
        }
        let Ok(request) = Frame::decode(frame) else {
            // Every request this crate builds decodes cleanly; a caller handing this transport a
            // malformed buffer through the trait object has nothing to reply to.
            return Ok(());
        };
        let reply = handle_request(&mut inner, &request);
        drop(inner);
        self.outbox.push_back(reply.encode());
        Ok(())
    }

    fn recv(&mut self, timeout: Duration) -> Result<Option<[u8; FRAME_LEN]>, TransportError> {
        // An injected push always jumps the queue ahead of a reply already computed and waiting
        // in `outbox`, matching a real device that can interleave a push with an in-flight reply.
        {
            let mut inner = lock(&self.inner);
            if inner.closed {
                return Err(TransportError::Disconnected);
            }
            if let Some(push) = inner.pending_pushes.pop_front() {
                return Ok(Some(push.encode()));
            }
        }
        if let Some(bytes) = self.outbox.pop_front() {
            return Ok(Some(bytes));
        }
        std::thread::sleep(timeout.min(IDLE_STEP));
        Ok(None)
    }

    fn description(&self) -> String {
        "Hyperpace simulator".to_owned()
    }
}

/// A cheap-to-clone controller for a [`SimTransport`]'s shared state.
///
/// Lets a test change what the simulator reports, mark a command unsupported, inject an
/// unsolicited push, or close the connection, all while an owner thread already runs against the
/// paired [`SimTransport`] on another thread.
#[derive(Clone)]
pub struct SimController {
    inner: Arc<Mutex<Inner>>,
}

impl SimController {
    /// Set the identity the next handshake reply carries.
    pub fn set_identity(&self, cid: u8, mid: u8, link_byte: u8) {
        let mut inner = lock(&self.inner);
        inner.cid = cid;
        inner.mid = mid;
        inner.link_byte = link_byte;
    }

    /// Set whether `DeviceOnLine` reports the device online.
    pub fn set_online(&self, online: bool) {
        lock(&self.inner).online = online;
    }

    /// Set the value `BatteryLevel` reports.
    pub fn set_battery(&self, percent: u8, charging: bool, millivolts: u16) {
        lock(&self.inner).battery = (percent, charging, millivolts);
    }

    /// Set the value `ReadVersionID` reports.
    pub fn set_version(&self, major: u8, minor: u8) {
        lock(&self.inner).version = (major, minor);
    }

    /// Force a command to answer with status 1 (unsupported), or clear that.
    pub fn set_unsupported(&self, command: u8, unsupported: bool) {
        let mut inner = lock(&self.inner);
        if unsupported {
            inner.unsupported.insert(command);
        } else {
            inner.unsupported.remove(&command);
        }
    }

    /// Queue an unsolicited push frame; it is delivered on the next `recv`, ahead of any reply
    /// already queued for an in-flight request.
    pub fn push(&self, frame: Frame) {
        lock(&self.inner).pending_pushes.push_back(frame);
    }

    /// The raw byte the simulator's own flash holds at `address`.
    #[must_use]
    pub fn flash_at(&self, address: u16) -> u8 {
        lock(&self.inner)
            .flash
            .get(usize::from(address))
            .copied()
            .unwrap_or(0xff)
    }

    /// Close the simulated connection; every further `send` or `recv` reports
    /// [`TransportError::Disconnected`], exactly like an unplugged device.
    pub fn close(&self) {
        lock(&self.inner).closed = true;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use hyperpace_protocol::request;
    use proptest::prelude::*;

    #[test]
    fn handshake_reports_the_configured_identity() {
        let (mut transport, _controller) = SimTransport::new(62, 1, 4);
        transport
            .send(&request::handshake([0; 4]).encode())
            .unwrap();
        let reply = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert_eq!(reply.command, 1);
        assert_eq!(reply.payload[4], 62);
        assert_eq!(reply.payload[5], 1);
        assert_eq!(reply.payload[6], 4);
    }

    #[test]
    fn battery_reply_understates_its_own_length() {
        let (mut transport, controller) = SimTransport::new(62, 1, 0);
        controller.set_battery(55, true, 0x0e74);
        transport.send(&Frame::command(4).encode()).unwrap();
        let reply = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert_eq!(
            reply.length, 1,
            "the length byte must understate the real 4-byte payload"
        );
        assert_eq!(reply.payload[0], 55);
        assert_eq!(reply.payload[1], 1);
        assert_eq!(
            u16::from_be_bytes([reply.payload[2], reply.payload[3]]),
            0x0e74
        );
    }

    #[test]
    fn an_unmodeled_command_answers_unsupported() {
        let (mut transport, _controller) = SimTransport::new(62, 1, 0);
        transport.send(&Frame::command(13).encode()).unwrap();
        let reply = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert_eq!(reply.status, 1);
    }

    #[test]
    fn a_command_can_be_forced_unsupported() {
        let (mut transport, controller) = SimTransport::new(62, 1, 0);
        controller.set_unsupported(4, true);
        transport.send(&Frame::command(4).encode()).unwrap();
        let reply = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert_eq!(reply.status, 1);

        controller.set_unsupported(4, false);
        transport.send(&Frame::command(4).encode()).unwrap();
        let reply = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert_eq!(reply.status, 0);
    }

    #[test]
    fn write_then_read_round_trips_through_the_simulated_flash() {
        let (mut transport, _controller) = SimTransport::new(62, 1, 0);
        let write = request::write_flash(20, &[1, 2, 3, 4]).unwrap();
        transport.send(&write.encode()).unwrap();
        let _ack = transport.recv(Duration::ZERO).unwrap().unwrap();

        transport
            .send(&request::read_flash(20, 4).encode())
            .unwrap();
        let reply = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert_eq!(&reply.payload[..4], &[1, 2, 3, 4]);
    }

    #[test]
    fn an_unsolicited_push_is_delivered_without_a_request() {
        let (mut transport, controller) = SimTransport::new(62, 1, 0);
        let mut push = Frame::command(10);
        push.payload[0] = 0x40;
        controller.push(push);

        let reply =
            Frame::decode(&transport.recv(Duration::from_millis(1)).unwrap().unwrap()).unwrap();
        assert_eq!(reply.command, 10);
        assert_eq!(reply.payload[0], 0x40);
    }

    #[test]
    fn a_push_is_delivered_ahead_of_a_pending_reply() {
        let (mut transport, controller) = SimTransport::new(62, 1, 0);
        controller.push(Frame::command(10));
        transport.send(&Frame::command(3).encode()).unwrap();

        let first = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert_eq!(first.command, 10);
        let second = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert_eq!(second.command, 3);
    }

    #[test]
    fn closing_reports_disconnected_on_both_ends() {
        let (mut transport, controller) = SimTransport::new(62, 1, 0);
        controller.close();
        assert_eq!(
            transport.send(&Frame::command(3).encode()),
            Err(TransportError::Disconnected)
        );
        assert_eq!(
            transport.recv(Duration::from_millis(1)),
            Err(TransportError::Disconnected)
        );
    }

    #[test]
    fn an_out_of_range_read_returns_erased_bytes_without_panicking() {
        let (mut transport, _controller) = SimTransport::new(62, 1, 0);
        let mut request = request::read_flash(0, 10);
        request.address = u16::MAX - 2;
        transport.send(&request.encode()).unwrap();
        let reply = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
        assert!(reply.payload.iter().all(|byte| *byte == 0xff));
    }

    proptest! {
        #[test]
        fn write_read_round_trips_for_any_in_bounds_chunk(
            address in 0u16..16000,
            data in prop::collection::vec(any::<u8>(), 1..=PAYLOAD_LEN),
        ) {
            let (mut transport, _controller) = SimTransport::new(62, 1, 0);
            let write = request::write_flash(address, &data).unwrap();
            transport.send(&write.encode()).unwrap();
            transport.recv(Duration::ZERO).unwrap();

            let len = u8::try_from(data.len()).unwrap();
            transport.send(&request::read_flash(address, len).encode()).unwrap();
            let reply = Frame::decode(&transport.recv(Duration::ZERO).unwrap().unwrap()).unwrap();
            prop_assert_eq!(&reply.payload[..data.len()], data.as_slice());
        }
    }
}
