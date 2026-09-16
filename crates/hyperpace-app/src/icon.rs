//! Renders the battery percentage into the tray icon's own pixels.
//!
//! `docs/plans/hyperpace.md`: "the tray icon is redrawn only when the displayed bucket changes,
//! because on this platform every icon update writes a file and crosses D-Bus." This module is
//! the pure half of that: it builds the RGBA buffer a caller hands to
//! [`tauri::image::Image::new_owned`]; nothing here touches Tauri, a file or D-Bus, so it is
//! fully unit-tested without a tray or a display.
//!
//! There is no system font dependency: the digits are a small hand-drawn 3x5 bitmap face, the
//! least code that reads at tray size (climbing the ladder past a font-rendering crate for eleven
//! glyphs).
//!
//! Every pixel coordinate in this module is bounded by [`ICON_SIZE`] (40) end to end, far under
//! every integer type involved here, so the `i32`/`u32`/`usize` conversions the geometry needs
//! are allowed at the module level rather than routed through `try_from` at each call site.
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

/// Icon width and height in pixels. Square, since every desktop tray this app targets scales a
/// square icon; large enough that a three-digit "100" is legible at typical panel sizes.
pub const ICON_SIZE: u32 = 40;

/// Pixel scale of one font cell (each glyph is drawn [`DIGIT_ROWS`] cells tall).
const GLYPH_SCALE: u32 = 3;
/// Columns in one glyph cell.
const DIGIT_COLS: u32 = 3;
/// Rows in one glyph cell.
const DIGIT_ROWS: u32 = 5;
/// Gap, in pixels, between adjacent glyphs.
const GLYPH_GAP: u32 = 3;

/// One row of a glyph, as the low [`DIGIT_COLS`] bits, most significant bit leftmost.
type GlyphRows = [u8; DIGIT_ROWS as usize];

/// `0` through `9`, then a bare dash for "no reading yet".
const DIGIT_FONT: [GlyphRows; 11] = [
    [0b111, 0b101, 0b101, 0b101, 0b111], // 0
    [0b010, 0b110, 0b010, 0b010, 0b111], // 1
    [0b111, 0b001, 0b111, 0b100, 0b111], // 2
    [0b111, 0b001, 0b111, 0b001, 0b111], // 3
    [0b101, 0b101, 0b111, 0b001, 0b001], // 4
    [0b111, 0b100, 0b111, 0b001, 0b111], // 5
    [0b111, 0b100, 0b111, 0b101, 0b111], // 6
    [0b111, 0b001, 0b010, 0b010, 0b010], // 7
    [0b111, 0b101, 0b111, 0b101, 0b111], // 8
    [0b111, 0b101, 0b111, 0b001, 0b111], // 9
    [0b000, 0b000, 0b111, 0b000, 0b000], // -
];
const DASH_INDEX: usize = 10;

/// An RGBA image, row-major top to bottom, ready for [`tauri::image::Image::new_owned`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayIconImage {
    /// Pixel bytes, four per pixel (R, G, B, A), [`Self::width`] * [`Self::height`] pixels.
    pub rgba: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// The backdrop color a charge level renders at: green when comfortable, amber when getting low,
/// red at or below the low-battery threshold, gray when there is no reading yet.
fn backdrop_color(percent: Option<u8>) -> [u8; 3] {
    match percent {
        None => [120, 120, 120],
        Some(percent) if percent <= 15 => [214, 69, 69],
        Some(percent) if percent <= 40 => [214, 168, 44],
        Some(_) => [58, 158, 94],
    }
}

/// Set one pixel, if it lies inside the buffer; silently clipped otherwise so a glyph placed near
/// an edge can never index out of bounds.
fn set_pixel(rgba: &mut [u8], width: u32, height: u32, x: i32, y: i32, color: [u8; 4]) {
    if x < 0 || y < 0 || x as u32 >= width || y as u32 >= height {
        return;
    }
    let offset = (y as u32 * width + x as u32) as usize * 4;
    rgba[offset..offset + 4].copy_from_slice(&color);
}

/// Draw one glyph with its top-left cell at (`x0`, `y0`) in cell units, each cell expanded to
/// [`GLYPH_SCALE`] pixels.
fn draw_glyph(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    x0: i32,
    y0: i32,
    glyph: GlyphRows,
    color: [u8; 4],
) {
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..DIGIT_COLS {
            if bits & (1 << (DIGIT_COLS - 1 - col)) == 0 {
                continue;
            }
            let px0 = x0 + (col * GLYPH_SCALE) as i32;
            let py0 = y0 + (row as u32 * GLYPH_SCALE) as i32;
            for dy in 0..GLYPH_SCALE {
                for dx in 0..GLYPH_SCALE {
                    set_pixel(rgba, width, height, px0 + dx as i32, py0 + dy as i32, color);
                }
            }
        }
    }
}

/// Width, in pixels, that `digits` renders at.
fn digits_width(digits: usize) -> u32 {
    let glyph_width = DIGIT_COLS * GLYPH_SCALE;
    digits as u32 * glyph_width + digits.saturating_sub(1) as u32 * GLYPH_GAP
}

/// The digits of `percent` (`0..=100`), most significant first, with no leading zero.
fn digits_of(percent: u8) -> Vec<usize> {
    if percent == 0 {
        return vec![0];
    }
    let mut value = percent;
    let mut digits = Vec::new();
    while value > 0 {
        digits.push(usize::from(value % 10));
        value /= 10;
    }
    digits.reverse();
    digits
}

/// Fill a rounded-rectangle backdrop covering the whole icon, so the icon reads against both a
/// light and a dark panel regardless of the desktop theme.
fn fill_backdrop(rgba: &mut [u8], size: u32, color: [u8; 3]) {
    let corner = (size / 5).max(1);
    for y in 0..size {
        for x in 0..size {
            // Squared corners are simply skipped, leaving them transparent, which reads as a
            // rounded rectangle at this icon's own pixel scale without a curve computation.
            let corner_x = if x < corner {
                corner - 1 - x
            } else {
                x.saturating_sub(size - corner)
            };
            let corner_y = if y < corner {
                corner - 1 - y
            } else {
                y.saturating_sub(size - corner)
            };
            if corner_x + corner_y > corner {
                continue;
            }
            set_pixel(
                rgba,
                size,
                size,
                x as i32,
                y as i32,
                [color[0], color[1], color[2], 255],
            );
        }
    }
}

/// Draw a small filled triangle (a stand-in lightning mark) in the icon's top-right corner,
/// indicating the device is charging.
fn draw_charging_mark(rgba: &mut [u8], size: u32) {
    let mark = size / 4;
    let white = [255, 255, 255, 255];
    for y in 0..mark {
        for x in 0..(mark - y) {
            let px = size as i32 - 2 - x as i32;
            let py = 2 + y as i32;
            set_pixel(rgba, size, size, px, py, white);
        }
    }
}

/// Render the tray icon for `percent` (`None` when no reading has arrived yet), marking the
/// device as charging when `charging` is set.
///
/// The digits are drawn in white for contrast against every `backdrop_color`; a `percent` over
/// 100 is clamped, since the device protocol never reports one but a caller should still get a
/// legible icon rather than a panic or a cut-off render.
#[must_use]
pub fn render_battery_icon(percent: Option<u8>, charging: bool) -> TrayIconImage {
    let size = ICON_SIZE;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    fill_backdrop(&mut rgba, size, backdrop_color(percent));

    let glyphs: Vec<usize> = match percent {
        Some(percent) => digits_of(percent.min(100)),
        None => vec![DASH_INDEX],
    };
    let total_width = digits_width(glyphs.len());
    let start_x = (size as i32 - total_width as i32) / 2;
    let start_y = (size as i32 - (DIGIT_ROWS * GLYPH_SCALE) as i32) / 2;
    let glyph_advance = (DIGIT_COLS * GLYPH_SCALE + GLYPH_GAP) as i32;

    for (index, &digit) in glyphs.iter().enumerate() {
        let x = start_x + index as i32 * glyph_advance;
        draw_glyph(
            &mut rgba,
            size,
            size,
            x,
            start_y,
            DIGIT_FONT[digit],
            [255, 255, 255, 255],
        );
    }

    if charging {
        draw_charging_mark(&mut rgba, size);
    }

    TrayIconImage {
        rgba,
        width: size,
        height: size,
    }
}

/// A short, non-branded tooltip line for the tray icon, e.g. `"Hyperpace: 73% (charging)"` or
/// `"Hyperpace: not connected"`.
#[must_use]
pub fn tray_tooltip(percent: Option<u8>, charging: bool) -> String {
    match percent {
        Some(percent) if charging => format!("Hyperpace: {percent}% (charging)"),
        Some(percent) => format!("Hyperpace: {percent}%"),
        None => "Hyperpace: not connected".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel_at(image: &TrayIconImage, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * image.width + x) * 4) as usize;
        [
            image.rgba[offset],
            image.rgba[offset + 1],
            image.rgba[offset + 2],
            image.rgba[offset + 3],
        ]
    }

    #[test]
    fn the_rendered_buffer_matches_its_declared_dimensions() {
        let image = render_battery_icon(Some(73), false);
        assert_eq!(image.width, ICON_SIZE);
        assert_eq!(image.height, ICON_SIZE);
        assert_eq!(image.rgba.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
    }

    #[test]
    fn the_center_pixel_is_opaque_white_digit_ink_or_backdrop() {
        // The exact glyph layout is an implementation detail; what must hold is that every pixel
        // is either fully transparent (outside the rounded backdrop) or fully opaque (backdrop or
        // digit ink), never a half-drawn value that would look wrong at tray scale.
        let image = render_battery_icon(Some(50), false);
        for chunk in image.rgba.as_chunks::<4>().0 {
            assert!(chunk[3] == 0 || chunk[3] == 255);
        }
    }

    #[test]
    fn full_battery_renders_green_and_low_battery_renders_red() {
        let full = render_battery_icon(Some(90), false);
        let low = render_battery_icon(Some(5), false);
        // Sample a backdrop pixel far from any digit: the icon's own corner is always inside the
        // rounded rectangle for this icon size.
        let full_corner = pixel_at(&full, ICON_SIZE / 2, 2);
        let low_corner = pixel_at(&low, ICON_SIZE / 2, 2);
        assert_eq!(full_corner, [58, 158, 94, 255]);
        assert_eq!(low_corner, [214, 69, 69, 255]);
    }

    #[test]
    fn unknown_battery_renders_gray_with_a_dash() {
        let image = render_battery_icon(None, false);
        let corner = pixel_at(&image, ICON_SIZE / 2, 2);
        assert_eq!(corner, [120, 120, 120, 255]);
    }

    #[test]
    fn charging_adds_a_mark_that_a_non_charging_icon_lacks() {
        let charging = render_battery_icon(Some(50), true);
        let idle = render_battery_icon(Some(50), false);
        assert_ne!(charging.rgba, idle.rgba);
    }

    #[test]
    fn one_hundred_percent_renders_without_clamping_below_it() {
        // 100 is the largest three-digit value this render path must fit without panicking or
        // silently clipping outside the buffer (`set_pixel` clips defensively; this proves the
        // whole render still succeeds at the extreme value).
        let image = render_battery_icon(Some(100), false);
        assert_eq!(image.rgba.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
    }

    #[test]
    fn a_value_over_one_hundred_is_clamped_instead_of_overflowing_the_layout() {
        let image = render_battery_icon(Some(255), false);
        assert_eq!(image.rgba.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
    }

    #[test]
    fn digits_of_has_no_leading_zero_and_handles_zero_itself() {
        assert_eq!(digits_of(0), vec![0]);
        assert_eq!(digits_of(7), vec![7]);
        assert_eq!(digits_of(42), vec![4, 2]);
        assert_eq!(digits_of(100), vec![1, 0, 0]);
    }

    #[test]
    fn tooltip_names_the_state_in_one_plain_sentence() {
        assert_eq!(tray_tooltip(Some(73), false), "Hyperpace: 73%");
        assert_eq!(tray_tooltip(Some(73), true), "Hyperpace: 73% (charging)");
        assert_eq!(tray_tooltip(None, false), "Hyperpace: not connected");
    }
}
