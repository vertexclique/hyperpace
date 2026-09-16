//! Connects to the attached device read-only and prints what it reports: identity, then the
//! battery readings as they arrive.
//!
//! The counterpart to `probe`: that one proves the right collection is found, this one proves the
//! device answers. [`hyperpace_device::Access::ReadOnly`] is not a convention here, it is enforced
//! by the owner thread, which refuses a write before it reaches the transport, so this cannot
//! change anything on the device.
//!
//! Run with `cargo run -p hyperpace-device --example read_state`.

use std::time::Duration;

use hyperpace_device::{Access, DeviceEvent, HidTransport};
use hyperpace_protocol::Transport;

fn main() {
    let transport = match HidTransport::open_first() {
        Ok(transport) => transport,
        Err(error) => {
            eprintln!("could not open the device: {error}");
            eprintln!("run the `probe` example to see which collections are attached.");
            std::process::exit(1);
        }
    };
    println!("opened: {}", transport.description());

    let handle = match hyperpace_device::spawn(Box::new(transport), Access::ReadOnly) {
        Ok(handle) => handle,
        Err(error) => {
            eprintln!("could not start the device owner thread: {error}");
            std::process::exit(1);
        }
    };

    let events = handle.events();
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while std::time::Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        match events.recv_timeout(remaining) {
            Ok(DeviceEvent::Connected(identity)) => println!(
                "connected: cid {} mid {} link {:?} (up to {} Hz)",
                identity.cid,
                identity.mid,
                identity.link,
                identity.link.max_polling_hz()
            ),
            Ok(DeviceEvent::Battery(battery)) => println!(
                "battery: {}% {} ({} mV)",
                battery.percent,
                if battery.charging {
                    "charging"
                } else {
                    "discharging"
                },
                battery.millivolts
            ),
            Ok(DeviceEvent::Changed(changed)) => println!("pushed a change: {changed:?}"),
            Ok(DeviceEvent::Offline) => println!("offline: asleep or out of range"),
            Ok(DeviceEvent::Disconnected) => {
                println!("disconnected");
                return;
            }
            Err(_) => break,
        }
    }
    println!("done");
}
