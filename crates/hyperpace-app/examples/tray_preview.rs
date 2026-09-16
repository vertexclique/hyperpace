//! Writes the tray icon at a few charge levels to PPM files, composited over a panel grey, so the
//! mark can be reviewed at real size without running the app.
//!
//! Run with `cargo run -p hyperpace-app --example tray_preview -- <output dir>`.

use std::io::Write;

/// The grey a desktop panel usually is, so the icon's transparent chamfered corners read in the
/// preview the way they will read in a tray.
const PANEL_GREY: u32 = 46;

/// Composite one channel of a pixel over [`PANEL_GREY`].
fn over_panel(channel: u8, alpha: u8) -> u8 {
    let alpha = u32::from(alpha);
    let blended = (u32::from(channel) * alpha + PANEL_GREY * (255 - alpha)) / 255;
    u8::try_from(blended).unwrap_or(u8::MAX)
}

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_owned());
    for (name, percent, charging) in [
        ("tray-85", Some(85u8), false),
        ("tray-12-charging", Some(12), true),
        ("tray-none", None, false),
        ("tray-100", Some(100), false),
    ] {
        let image = hyperpace_app::icon::render_battery_icon(percent, charging);
        let mut out = Vec::new();
        let _ = write!(out, "P6\n{} {}\n255\n", image.width, image.height);
        for pixel in image.rgba.chunks(4) {
            let [red, green, blue, alpha] = [pixel[0], pixel[1], pixel[2], pixel[3]];
            out.extend_from_slice(&[
                over_panel(red, alpha),
                over_panel(green, alpha),
                over_panel(blue, alpha),
            ]);
        }

        let path = format!("{dir}/{name}.ppm");
        match std::fs::write(&path, out) {
            Ok(()) => println!("{path}"),
            Err(error) => {
                eprintln!("could not write {path}: {error}");
                std::process::exit(1);
            }
        }
    }
}
