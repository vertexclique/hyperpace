//! Lists this product's HID collections as hidapi reports them, without opening any of them.
//!
//! A diagnostic for the one question a user of a device app cannot answer alone: "the mouse is
//! plugged in, so why does the app say it cannot find it?" Enumeration is read-only; nothing here
//! opens, reads from or writes to the device.
//!
//! Run with `cargo run -p hyperpace-device --example probe`.

fn main() {
    let api = match hidapi::HidApi::new() {
        Ok(api) => api,
        Err(error) => {
            eprintln!("could not reach the HID subsystem: {error}");
            std::process::exit(1);
        }
    };

    let mut found = 0_usize;
    for info in api.device_list() {
        if info.vendor_id() != 0x3554 {
            continue;
        }
        found += 1;
        println!(
            "{:04x}:{:04x}  usage page {:#06x}  usage {:#06x}  interface {}  path {}",
            info.vendor_id(),
            info.product_id(),
            info.usage_page(),
            info.usage(),
            info.interface_number(),
            info.path().to_string_lossy(),
        );
    }
    println!("\n{found} collection(s) attached for vendor 0x3554");
}
