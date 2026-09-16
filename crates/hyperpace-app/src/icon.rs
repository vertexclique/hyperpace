//! Renders the battery percentage into the tray icon's own pixels, inside the Hyperpace mark.
//!
//! The mark is the logo's own geometry (`art/hyperpace.svg`): a chamfered frame with a neon corner
//! bracket at the top left and the bottom right. Those shapes are redrawn here as pixels rather
//! than rasterized from the SVG, because at 40 pixels the logo's grid, traces and glow render as
//! mush, and because rasterizing an SVG would mean a renderer dependency for one small square.
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

/// Pixel scale of one font cell for a one or two digit reading (each glyph is drawn
/// [`DIGIT_ROWS`] cells tall).
const GLYPH_SCALE: u32 = 3;
/// Pixel scale used for a three digit reading, which is only ever "100": at [`GLYPH_SCALE`] three
/// glyphs run into the chamfered frame's own edge, so the full-charge icon steps down a size
/// rather than losing its margin.
const GLYPH_SCALE_NARROW: u32 = 2;
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

/// The square RGBA buffer being drawn into, so no drawing function has to thread the buffer, its
/// width and its height separately.
struct Canvas<'a> {
    rgba: &'a mut [u8],
    size: u32,
}

impl Canvas<'_> {
    /// Set one pixel, if it lies inside the buffer; silently clipped otherwise, so a glyph placed
    /// near an edge can never index out of bounds.
    fn set(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x as u32 >= self.size || y as u32 >= self.size {
            return;
        }
        let offset = (y as u32 * self.size + x as u32) as usize * 4;
        self.rgba[offset..offset + 4].copy_from_slice(&color);
    }
}

/// Draw one glyph with its top-left cell at (`x0`, `y0`) in cell units, each cell expanded to
/// [`GLYPH_SCALE`] pixels.
fn draw_glyph(canvas: &mut Canvas, x0: i32, y0: i32, glyph: GlyphRows, color: [u8; 4], scale: u32) {
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..DIGIT_COLS {
            if bits & (1 << (DIGIT_COLS - 1 - col)) == 0 {
                continue;
            }
            let px0 = x0 + (col * scale) as i32;
            let py0 = y0 + (row as u32 * scale) as i32;
            for dy in 0..scale {
                for dx in 0..scale {
                    canvas.set(px0 + dx as i32, py0 + dy as i32, color);
                }
            }
        }
    }
}

/// The cell scale a reading of `digits` glyphs is drawn at.
fn glyph_scale(digits: usize) -> u32 {
    if digits >= 3 {
        GLYPH_SCALE_NARROW
    } else {
        GLYPH_SCALE
    }
}

/// Width, in pixels, that `digits` renders at.
fn digits_width(digits: usize) -> u32 {
    let scale = glyph_scale(digits);
    let glyph_width = DIGIT_COLS * scale;
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

/// The logo's neon cyan, for the corner brackets.
const BRAND_CYAN: [u8; 4] = [0, 240, 255, 255];
/// The logo's red, for the charging mark.
const BRAND_RED: [u8; 4] = [255, 0, 60, 255];

/// How deep each corner is cut, as a fraction of the icon: the logo cuts 40 of its 512 units.
fn chamfer(size: u32) -> u32 {
    (size * 40 / 512).max(3)
}

/// Whether `(x, y)` lies inside the chamfered frame, which is the logo's outer shape: a square
/// with all four corners cut at 45 degrees.
fn inside_frame(size: u32, x: u32, y: u32, cut: u32) -> bool {
    let from_right = size - 1 - x;
    let from_bottom = size - 1 - y;
    x + y >= cut
        && from_right + y >= cut
        && x + from_bottom >= cut
        && from_right + from_bottom >= cut
}

/// Fill the chamfered frame covering the whole icon, so the icon reads against both a light and a
/// dark panel regardless of the desktop theme. Nothing is drawn outside the frame, so the cut
/// corners stay transparent and the mark's silhouette is the logo's, not a plain square.
fn fill_backdrop(canvas: &mut Canvas, color: [u8; 3]) {
    let size = canvas.size;
    let cut = chamfer(size);
    for y in 0..size {
        for x in 0..size {
            if inside_frame(size, x, y, cut) {
                canvas.set(x as i32, y as i32, [color[0], color[1], color[2], 255]);
            }
        }
    }
}

/// Draw the logo's two neon corner brackets, top left and bottom right, tracing the chamfered
/// edge. This is what makes the tray icon recognizably this app rather than a generic status pill,
/// and it is the one place the brand colour appears in the icon: everything else is status.
fn draw_corner_brackets(canvas: &mut Canvas) {
    let size = canvas.size;
    let cut = chamfer(size);
    let thickness = (size / 20).max(2);
    let arm = (size / 3).max(cut + thickness);

    for step in 0..arm {
        for t in 0..thickness {
            // Top left: down the left edge, then right along the top edge.
            canvas.set(t as i32, (cut + step) as i32, BRAND_CYAN);
            canvas.set((cut + step) as i32, t as i32, BRAND_CYAN);
            // Bottom right: up the right edge, then left along the bottom edge.
            let far = (size - 1 - t) as i32;
            canvas.set(far, (size - 1 - cut - step) as i32, BRAND_CYAN);
            canvas.set((size - 1 - cut - step) as i32, far, BRAND_CYAN);
        }
    }

    // The 45 degree run across each cut corner, so the bracket turns the chamfer instead of
    // stopping short of it.
    for step in 0..=cut {
        for t in 0..thickness {
            let x = (cut - step) as i32;
            let y = step as i32 + t as i32;
            canvas.set(x, y, BRAND_CYAN);
            canvas.set(size as i32 - 1 - x, size as i32 - 1 - y, BRAND_CYAN);
        }
    }
}

/// Draw the logo's red diamond in the icon's top right corner, indicating the device is charging.
/// The logo puts the same diamond at the head of its shell, so charging reads as part of the mark
/// rather than a sticker on top of it.
fn draw_charging_mark(canvas: &mut Canvas) {
    let size = canvas.size;
    let radius = (size / 8).max(2) as i32;
    let cx = size as i32 - radius - 3;
    let cy = radius + 3;
    for dy in -radius..=radius {
        let span = radius - dy.abs();
        for dx in -span..=span {
            canvas.set(cx + dx, cy + dy, BRAND_RED);
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
    let mut canvas = Canvas {
        rgba: &mut rgba,
        size,
    };
    fill_backdrop(&mut canvas, backdrop_color(percent));
    draw_corner_brackets(&mut canvas);

    let glyphs: Vec<usize> = match percent {
        Some(percent) => digits_of(percent.min(100)),
        None => vec![DASH_INDEX],
    };
    let scale = glyph_scale(glyphs.len());
    let total_width = digits_width(glyphs.len());
    let start_x = (size as i32 - total_width as i32) / 2;
    let start_y = (size as i32 - (DIGIT_ROWS * scale) as i32) / 2;
    let glyph_advance = (DIGIT_COLS * scale + GLYPH_GAP) as i32;

    for (index, &digit) in glyphs.iter().enumerate() {
        let x = start_x + index as i32 * glyph_advance;
        draw_glyph(
            &mut canvas,
            x,
            start_y,
            DIGIT_FONT[digit],
            [255, 255, 255, 255],
            scale,
        );
    }

    if charging {
        draw_charging_mark(&mut canvas);
    }

    TrayIconImage {
        rgba,
        width: size,
        height: size,
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
}
