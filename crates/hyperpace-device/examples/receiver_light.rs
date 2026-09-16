//! Reads the receiver's own indicator light, read-only, and prints what it reports.
//!
//! The receiver answers this itself (`GetDongleLight`, command 25), so it works while the mouse
//! sleeps. Opened with [`Access::ReadOnly`], which the owner thread enforces, so this cannot change
//! the light or anything else.
//!
//! Run with `cargo run -p hyperpace-device --example receiver_light` to read the light, or with
//! `-- set <mode> <r> <g> <b> <speed> <brightness> <time>` to set it and read it back. Setting
//! writes to the receiver only; it never touches the mouse.

use std::time::Duration;

use hyperpace_device::{Access, HidTransport};
use hyperpace_protocol::request;
use hyperpace_protocol::settings::ReceiverLight;
use hyperpace_protocol::{Command, Status, response};

fn main() {
    let transport = match HidTransport::open_first() {
        Ok(transport) => transport,
        Err(error) => {
            eprintln!("could not open the device: {error}");
            std::process::exit(1);
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let setting = if args.first().map(String::as_str) == Some("set") {
        let numbers: Vec<u8> = args[1..].iter().filter_map(|a| a.parse().ok()).collect();
        if numbers.len() != 7 {
            eprintln!("set needs: <mode> <r> <g> <b> <speed> <brightness> <time>");
            std::process::exit(1);
        }
        Some(ReceiverLight {
            mode: numbers[0],
            color: (numbers[1], numbers[2], numbers[3]),
            speed: numbers[4],
            brightness: numbers[5],
            time: numbers[6],
        })
    } else {
        None
    };
    let access = if setting.is_some() {
        Access::ReadWrite
    } else {
        Access::ReadOnly
    };

    let handle = match hyperpace_device::spawn(Box::new(transport), access) {
        Ok(handle) => handle,
        Err(error) => {
            eprintln!("could not start the owner thread: {error}");
            std::process::exit(1);
        }
    };

    if let Some(light) = setting {
        match handle.request(request::receiver_light(&light), Duration::from_millis(1500)) {
            Ok(reply) if reply.status() == Status::Unsupported => {
                println!("the receiver refused the write as unsupported (status 1)");
            }
            Ok(_) => println!("wrote {light:?}"),
            Err(error) => println!("the write failed: {error}"),
        }
    }

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
