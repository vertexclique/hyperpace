//! Renders the tray icon: the battery percentage in large figures on the Hyperpace mark.
//!
//! The mark is the logo's frame (`art/hyperpace.svg`): a dark void with its corners cut and neon
//! corner brackets at the top left and bottom right. On it, the percentage fills as much of the
//! icon as it can, in a status colour, because at tray size the figures are the only part anyone
//! can read. An earlier version drew the logo's mouse shell as a gauge with the figures inside it;
//! the operator could not read them at tray size, so the figures won.
//!
//! The shapes are redrawn as pixels from the logo's own coordinates rather than rasterized from the
//! SVG: the grid, traces and glow turn to mush at tray size, and rasterizing would mean a renderer
//! dependency for one small square.
//!
//! `docs/plans/hyperpace.md`: "the tray icon is redrawn only when the displayed bucket changes,
//! because on this platform every icon update writes a file and crosses D-Bus." This module is the
//! pure half of that: it builds the RGBA buffer a caller hands to
//! [`tauri::image::Image::new_owned`]; nothing here touches Tauri, a file or D-Bus, so it is fully
//! unit-tested without a tray or a display.
//!
//! There is no system font dependency: the digits are a small hand-drawn 3x5 bitmap face, scaled
//! up, the least code that reads at tray size.
//!
//! Every pixel coordinate in this module is bounded by [`ICON_SIZE`] end to end, far under every
//! integer and float type involved here, so the conversions the geometry needs are allowed at the
//! module level rather than routed through `try_from` at each call site.
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

/// Icon width and height in pixels. Square, and large enough that the shell and its digits keep
/// their shape when a desktop panel scales the icon down.
pub const ICON_SIZE: u32 = 64;

/// The logo's coordinate space: every shape below is written in it and scaled to [`ICON_SIZE`].
const LOGO_UNITS: f32 = 512.0;

/// The logo's background void.
const VOID: [u8; 4] = [5, 9, 20, 255];
/// The logo's neon cyan: the corner brackets.
const CYAN: [u8; 4] = [0, 240, 255, 255];
/// The figures when there is no reading to show.
const UNKNOWN_GREY: [u8; 4] = [120, 120, 120, 255];
/// The charging bolt.
const BOLT: [u8; 4] = [255, 214, 0, 255];

/// The chamfer the logo cuts from each corner of its frame.
const FRAME_CUT: f32 = 40.0;
/// The two neon corner brackets, as polylines.
const BRACKETS: [[(f32, f32); 3]; 2] = [
    [(24.0, 150.0), (24.0, 64.0), (104.0, 24.0)],
    [(488.0, 362.0), (488.0, 448.0), (408.0, 488.0)],
];
/// The charging bolt, small, in the frame's top right corner.
const BOLT_SHAPE: [(f32, f32); 6] = [
    (452.0, 30.0),
    (410.0, 112.0),
    (440.0, 112.0),
    (424.0, 176.0),
    (480.0, 84.0),
    (448.0, 84.0),
];

/// Pixel scale of one font cell for a one or two digit reading (each glyph is drawn [`DIGIT_ROWS`]
/// cells tall): two glyphs at this scale span most of the icon's width.
const GLYPH_SCALE: u32 = 8;
/// Pixel scale for a three digit reading, which is only ever "100".
const GLYPH_SCALE_NARROW: u32 = 5;
/// Columns in one glyph cell.
const DIGIT_COLS: u32 = 3;
/// Rows in one glyph cell.
const DIGIT_ROWS: u32 = 5;
/// Gap, in pixels, between adjacent glyphs.
const GLYPH_GAP: u32 = 4;

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

/// The colour the figures are drawn in for a charge level: the logo's cyan when comfortable, amber
/// when getting low, red at or below the low-battery threshold. Bright enough to read on the void.
fn figure_color(percent: u8) -> [u8; 4] {
    match percent {
        0..=15 => [255, 72, 88, 255],
        16..=40 => [255, 190, 50, 255],
        _ => [0, 240, 255, 255],
    }
}

/// The square RGBA buffer being drawn into, so no drawing function has to thread the buffer and its
/// size separately.
struct Canvas<'a> {
    rgba: &'a mut [u8],
    size: u32,
}

impl Canvas<'_> {
    /// Set one pixel, if it lies inside the buffer; silently clipped otherwise, so a shape placed
    /// near an edge can never index out of bounds.
    fn set(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x as u32 >= self.size || y as u32 >= self.size {
            return;
        }
        let offset = (y as u32 * self.size + x as u32) as usize * 4;
        self.rgba[offset..offset + 4].copy_from_slice(&color);
    }

    /// The logo-unit to pixel scale.
    fn scale(&self) -> f32 {
        self.size as f32 / LOGO_UNITS
    }
}

/// Whether the pixel centre `(px, py)` lies inside `polygon` (logo units scaled by `scale`), by the
/// even-odd rule.
fn inside(polygon: &[(f32, f32)], scale: f32, px: f32, py: f32) -> bool {
    let mut hit = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        let (x1, y1) = (current.0 * scale, current.1 * scale);
        let (x0, y0) = (previous.0 * scale, previous.1 * scale);
        if (y1 > py) != (y0 > py) && px < (x0 - x1) * (py - y1) / (y0 - y1) + x1 {
            hit = !hit;
        }
        previous = current;
    }
    hit
}

/// Whether `(x, y)` lies inside the chamfered frame, the logo's outer silhouette.
fn inside_frame(size: u32, x: u32, y: u32, cut: u32) -> bool {
    let from_right = size - 1 - x;
    let from_bottom = size - 1 - y;
    x + y >= cut
        && from_right + y >= cut
        && x + from_bottom >= cut
        && from_right + from_bottom >= cut
}

/// Fill the frame with the logo's void. Outside it the corners stay transparent, so the icon's
/// silhouette is the logo's chamfered square.
fn fill_frame(canvas: &mut Canvas) {
    let size = canvas.size;
    let cut = (FRAME_CUT * canvas.scale()).round() as u32;
    for y in 0..size {
        for x in 0..size {
            if inside_frame(size, x, y, cut) {
                canvas.set(x as i32, y as i32, VOID);
            }
        }
    }
}

/// Stroke a polyline with square pens `thickness` pixels wide, sampling each segment densely enough
/// that no gap opens at any angle.
fn stroke(canvas: &mut Canvas, points: &[(f32, f32)], thickness: i32, color: [u8; 4]) {
    let scale = canvas.scale();
    for pair in points.windows(2) {
        let (x0, y0) = (pair[0].0 * scale, pair[0].1 * scale);
        let (x1, y1) = (pair[1].0 * scale, pair[1].1 * scale);
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as i32 * 2;
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let cx = (x0 + (x1 - x0) * t).floor() as i32;
            let cy = (y0 + (y1 - y0) * t).floor() as i32;
            for dy in 0..thickness {
                for dx in 0..thickness {
                    canvas.set(cx + dx - thickness / 2, cy + dy - thickness / 2, color);
                }
            }
        }
    }
}

/// Fill a polygon (logo units) with `color`.
fn fill_polygon(canvas: &mut Canvas, polygon: &[(f32, f32)], color: [u8; 4]) {
    let size = canvas.size;
    let scale = canvas.scale();
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            if inside(polygon, scale, x as f32 + 0.5, y as f32 + 0.5) {
                canvas.set(x, y, color);
            }
        }
    }
}

/// Draw one glyph with its top-left cell at (`x0`, `y0`), each cell expanded to `scale` pixels.
fn draw_glyph(canvas: &mut Canvas, x0: i32, y0: i32, glyph: GlyphRows, scale: u32, color: [u8; 4]) {
    let scale = scale as i32;
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..DIGIT_COLS {
            if bits & (1 << (DIGIT_COLS - 1 - col)) == 0 {
                continue;
            }
            let cx = x0 + col as i32 * scale;
            let cy = y0 + row as i32 * scale;
            for dy in 0..scale {
                for dx in 0..scale {
                    canvas.set(cx + dx, cy + dy, color);
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
    let glyph_width = DIGIT_COLS * glyph_scale(digits);
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

/// Render the tray icon for `percent` (`None` when no reading has arrived yet), marking the device
/// as charging when `charging` is set.
///
/// A `percent` over 100 is clamped, since the device protocol never reports one but a caller should
/// still get a legible icon rather than a panic or a cut-off render.
#[must_use]
pub fn render_battery_icon(percent: Option<u8>, charging: bool) -> TrayIconImage {
    let size = ICON_SIZE;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let mut canvas = Canvas {
        rgba: &mut rgba,
        size,
    };
    let percent = percent.map(|percent| percent.min(100));
    let pen = (size / 32).max(2) as i32;

    fill_frame(&mut canvas);
    for bracket in &BRACKETS {
        stroke(&mut canvas, bracket, pen, CYAN);
    }

    let (glyphs, color): (Vec<usize>, [u8; 4]) = match percent {
        Some(percent) => (digits_of(percent), figure_color(percent)),
        None => (vec![DASH_INDEX], UNKNOWN_GREY),
    };
    let scale = glyph_scale(glyphs.len());
    let start_x = (size as i32 - digits_width(glyphs.len()) as i32) / 2;
    let start_y = (size as i32 - (DIGIT_ROWS * scale) as i32) / 2;
    let advance = (DIGIT_COLS * scale + GLYPH_GAP) as i32;
    for (index, &digit) in glyphs.iter().enumerate() {
        let x = start_x + index as i32 * advance;
        draw_glyph(&mut canvas, x, start_y, DIGIT_FONT[digit], scale, color);
    }

    if charging {
        // A one pixel void keyline first, so the bolt stays distinct where it meets a figure.
        let unit = LOGO_UNITS / size as f32;
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            let shifted: Vec<(f32, f32)> = BOLT_SHAPE
                .iter()
                .map(|&(x, y)| (x + dx * unit, y + dy * unit))
                .collect();
            fill_polygon(&mut canvas, &shifted, VOID);
        }
        fill_polygon(&mut canvas, &BOLT_SHAPE, BOLT);
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

    fn count(image: &TrayIconImage, color: [u8; 4]) -> usize {
        image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| **pixel == color)
            .count()
    }

    #[test]
    fn the_rendered_buffer_matches_its_declared_dimensions() {
        let image = render_battery_icon(Some(73), false);
        assert_eq!(image.width, ICON_SIZE);
        assert_eq!(image.height, ICON_SIZE);
        assert_eq!(image.rgba.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
    }

    #[test]
    fn every_pixel_is_either_fully_transparent_or_fully_opaque() {
        // A half-drawn alpha value would look wrong once a panel scales the icon.
        let image = render_battery_icon(Some(50), false);
        for chunk in image.rgba.as_chunks::<4>().0 {
            assert!(chunk[3] == 0 || chunk[3] == 255);
        }
    }

    #[test]
    fn the_cut_corners_are_transparent_like_the_logo() {
        let image = render_battery_icon(Some(50), false);
        assert_eq!(pixel_at(&image, 0, 0)[3], 0);
        assert_eq!(pixel_at(&image, ICON_SIZE - 1, ICON_SIZE - 1)[3], 0);
    }

    #[test]
    fn the_background_is_the_logo_void() {
        let image = render_battery_icon(Some(50), false);
        assert_eq!(pixel_at(&image, ICON_SIZE / 2, 3), VOID);
    }

    #[test]
    fn the_figures_are_large_enough_to_read_at_tray_size() {
        // Two digits at this scale are 40 pixels tall on a 64 pixel icon: most of its height.
        assert_eq!(DIGIT_ROWS * GLYPH_SCALE, 40);
        assert!(digits_width(2) >= ICON_SIZE * 3 / 4);
        assert!(digits_width(3) < ICON_SIZE);
    }

    #[test]
    fn the_figures_carry_the_charge_colour() {
        assert!(count(&render_battery_icon(Some(90), false), figure_color(90)) > 200);
        assert!(count(&render_battery_icon(Some(30), false), figure_color(30)) > 200);
        assert!(count(&render_battery_icon(Some(8), false), figure_color(8)) > 100);
        assert_ne!(figure_color(90), figure_color(30));
        assert_ne!(figure_color(30), figure_color(8));
    }

    #[test]
    fn no_reading_shows_a_grey_dash_rather_than_a_number() {
        let image = render_battery_icon(None, false);
        assert!(count(&image, UNKNOWN_GREY) > 0);
        // The comfortable-charge colour is the brackets' cyan, so with no figures drawn in it the
        // icon holds exactly as much cyan as one whose figures are red.
        assert_eq!(
            count(&image, CYAN),
            count(&render_battery_icon(Some(8), false), CYAN)
        );
    }

    #[test]
    fn charging_adds_a_mark_that_a_non_charging_icon_lacks() {
        let charging = render_battery_icon(Some(50), true);
        let idle = render_battery_icon(Some(50), false);
        assert_ne!(charging.rgba, idle.rgba);
    }

    #[test]
    fn a_value_over_one_hundred_is_clamped_instead_of_overflowing_the_layout() {
        assert_eq!(
            render_battery_icon(Some(255), false),
            render_battery_icon(Some(100), false)
        );
    }

    #[test]
    fn digits_of_has_no_leading_zero_and_handles_zero_itself() {
        assert_eq!(digits_of(0), vec![0]);
        assert_eq!(digits_of(7), vec![7]);
        assert_eq!(digits_of(42), vec![4, 2]);
        assert_eq!(digits_of(100), vec![1, 0, 0]);
    }
}
