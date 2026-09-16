//! Reads the receiver's own indicator light, read-only, and prints what it reports.
//!
//! The receiver answers this itself (`GetDongleLight`, command 25), so it works while the mouse
//! sleeps. Opened with [`Access::ReadOnly`], which the owner thread enforces, so this cannot change
//! the light or anything else.
//!
//! Run with `cargo run -p hyperpace-device --example receiver_light`.

use std::time::Duration;

use hyperpace_device::{Access, HidTransport};
use hyperpace_protocol::{Command, Status, response};

fn main() {
    let transport = match HidTransport::open_first() {
        Ok(transport) => transport,
        Err(error) => {
            eprintln!("could not open the device: {error}");
            std::process::exit(1);
        }
    };
    let handle = match hyperpace_device::spawn(Box::new(transport), Access::ReadOnly) {
        Ok(handle) => handle,
        Err(error) => {
            eprintln!("could not start the owner thread: {error}");
            std::process::exit(1);
        }
    };

    match handle.request(
        Command::GetReceiverLight.request(),
        Duration::from_millis(1500),
    ) {
        Ok(reply) if reply.status() == Status::Unsupported => {
            println!("the receiver reports its light as unsupported (status 1)");
        }
        Ok(reply) => {
            println!("raw payload: {:02x?}", reply.payload);
            match response::receiver_light(&reply) {
                Ok(light) => println!(
                    "mode {} color {:?} speed {} brightness {} time {}",
                    light.mode, light.color, light.speed, light.brightness, light.time
                ),
                Err(error) => println!("could not decode the reply: {error}"),
            }
        }
        Err(error) => println!("no reply: {error}"),
    }
}
