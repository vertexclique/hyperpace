//! Reads everything a settings screen needs from the attached device, read-only, and reports
//! exactly where it fails: the settings shadow (saved to a file), its decode, and each optional
//! query with its timing.
//!
//! The device is opened with [`Access::ReadOnly`], which the owner thread enforces by refusing any
//! write before it reaches the transport, so this cannot change anything on the device.
//!
//! Run with `cargo run -p hyperpace-device --example dump_settings -- <shadow output file>`.

use std::time::{Duration, Instant};

use hyperpace_device::{Access, DeviceEvent, HidTransport};
use hyperpace_protocol::{Command, offset, table_for};

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "shadow.bin".to_owned());
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

    let events = handle.events();
    let started = Instant::now();
    let identity = loop {
        match events.recv_timeout(Duration::from_secs(30)) {
            Ok(DeviceEvent::Connected(identity)) => break identity,
            Ok(_) => {}
            Err(_) => {
                eprintln!("the device never finished connecting");
                std::process::exit(1);
            }
        }
    };
    println!(
        "connected in {:?}: cid {} mid {} link {:?}",
        started.elapsed(),
        identity.cid,
        identity.mid,
        identity.link
    );

    let shadow = match handle.read_settings() {
        Ok(shadow) => shadow,
        Err(error) => {
            eprintln!("read_settings failed: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = std::fs::write(&out, shadow.as_bytes()) {
        eprintln!("could not save the shadow to {out}: {error}");
    } else {
        println!("shadow saved to {out} ({} bytes)", shadow.as_bytes().len());
    }
    println!(
        "polling scalar at {}: {:#04x} (complement {:#04x})",
        offset::POLLING,
        shadow.scalar(offset::POLLING),
        shadow.scalar(offset::POLLING + 1)
    );

    match table_for(identity.cid, identity.mid) {
        Some(table) => match shadow.settings(table) {
            Ok(settings) => println!("settings decode: ok\n{settings:#?}"),
            Err(error) => println!("settings decode: FAILED: {error}"),
        },
        None => println!(
            "no model table for cid {} mid {}",
            identity.cid, identity.mid
        ),
    }

    for (name, command) in [
        ("GetLongRange", Command::GetLongRange),
        ("GetProfile", Command::GetProfile),
    ] {
        let sent = Instant::now();
        match handle.request(command.request(), Duration::from_millis(1500)) {
            Ok(reply) => println!(
                "{name}: replied in {:?}, status {:?}, payload {:02x?}",
                sent.elapsed(),
                reply.status(),
                reply.payload
            ),
            Err(error) => println!("{name}: FAILED after {:?}: {error}", sent.elapsed()),
        }
    }
}
