//! Deterministic 40 × 40 RGBA menu tiles.
//!
//! Tiles follow the camera's own menu icons: a dark square with a gray frame
//! and anti-aliased text. The native list icons have a 38 x 36 frame, and the
//! icons of the ADJ toolbar a 38 x 38 frame. Effect icons can instead be gray film covers
//! (`render_box`, `render_cine`) that span the tile width. Alpha is binary;
//! text edges blend in RGB.

use ab_glyph::{point, Font, FontRef, PxScale, Rect, ScaleFont};
use anyhow::{bail, ensure, Context, Result};

/// Inter Medium 4.1, SIL Open Font License 1.1 (`assets/Inter-LICENSE.txt`).
const FONT: &[u8] = include_bytes!("../assets/Inter-Medium.ttf");
/// Effect icon labels use capitals and digits, as the native text tiles do.
const LABEL_CHARACTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
/// The largest label size. Its capitals are about 10 pixels high, close to the
/// 11 pixels of the native toolbar text, such as "Focus". A label that does
/// not fit `LABEL_WIDTH` at this size shrinks.
const LABEL_SIZE: f32 = 18.0;
/// Text keeps at least 4 pixels from each side of the 36-pixel frame inside.
const LABEL_WIDTH: f32 = 28.0;

fn dot(rgba: &mut [u8], x: usize, y: usize, shade: u8) {
    let offset = (y * 40 + x) * 4;
    rgba[offset..offset + 4].copy_from_slice(&[shade, shade, shade, 255]);
}

/// A dark square with a gray frame on columns `left..=right` and rows
/// `top..=bottom`. The rest of the tile stays transparent.
fn frame(left: usize, top: usize, right: usize, bottom: usize) -> Vec<u8> {
    let mut rgba = vec![0; 40 * 40 * 4];
    for y in top..=bottom {
        for x in left..=right {
            let edge = x == left || x == right || y == top || y == bottom;
            dot(&mut rgba, x, y, if edge { 128 } else { 51 });
        }
    }
    rgba
}

/// The frame of a list row, as the native list icons have it.
fn background() -> Vec<u8> {
    frame(1, 2, 38, 37)
}

/// The frame of the ADJ toolbar, as the native toolbar icons have it.
fn toolbar_background() -> Vec<u8> {
    frame(1, 1, 38, 38)
}

/// Blend `ink` over an opaque tile pixel by glyph coverage.
fn blend(rgba: &mut [u8], x: i32, y: i32, ink: u8, coverage: f32) {
    if !(0..40).contains(&x) || !(0..40).contains(&y) {
        return;
    }
    let offset = (y as usize * 40 + x as usize) * 4;
    if rgba[offset + 3] != 255 {
        return;
    }
    let coverage = coverage.clamp(0.0, 1.0);
    for channel in &mut rgba[offset..offset + 3] {
        let old = f32::from(*channel);
        *channel = (old + (f32::from(ink) - old) * coverage).round() as u8;
    }
}

/// Draw `label` centered horizontally, with its capitals centered on `middle`.
/// The size shrinks from `size` until the label fits within `width` pixels.
fn text(rgba: &mut [u8], label: &str, size: f32, width: f32, middle: f32, ink: u8) {
    let font = FontRef::try_from_slice(FONT).expect("embedded font is valid");
    let mut scale = size;
    loop {
        let scaled = font.as_scaled(PxScale::from(scale));
        let mut caret = 0.0;
        let mut previous = None;
        let mut glyphs = Vec::new();
        for character in label.chars() {
            let id = scaled.glyph_id(character);
            if let Some(previous) = previous {
                caret += scaled.kern(previous, id);
            }
            glyphs.push(id.with_scale_and_position(scale, point(caret, 0.0)));
            caret += scaled.h_advance(id);
            previous = Some(id);
        }
        let outlines: Vec<_> = glyphs
            .into_iter()
            .filter_map(|glyph| font.outline_glyph(glyph))
            .collect();
        let bounds = outlines
            .iter()
            .map(|outline| outline.px_bounds())
            .reduce(|a, b| Rect {
                min: point(a.min.x.min(b.min.x), a.min.y.min(b.min.y)),
                max: point(a.max.x.max(b.max.x), a.max.y.max(b.max.y)),
            });
        let Some(bounds) = bounds else { return };
        if bounds.width() > width && scale > 6.0 {
            scale -= 0.5;
            continue;
        }
        let capital = font
            .outline_glyph(scaled.glyph_id('H').with_scale(scale))
            .map_or(scale * 0.73, |outline| outline.px_bounds().height());
        // Whole-pixel offsets keep vertical and horizontal stems sharp.
        let dx = (20.0 - (bounds.min.x + bounds.max.x) / 2.0).round() as i32;
        let dy = (middle + capital / 2.0).round() as i32;
        for outline in outlines {
            let origin = outline.px_bounds().min;
            outline.draw(|x, y, coverage| {
                blend(
                    rgba,
                    origin.x as i32 + x as i32 + dx,
                    origin.y as i32 + y as i32 + dy,
                    ink,
                    coverage,
                )
            });
        }
        return;
    }
}

pub fn render_tile(label: &str, shape: &str) -> Result<Vec<u8>> {
    ensure!(
        (1..=4).contains(&label.len()) && label.bytes().all(|c| LABEL_CHARACTERS.contains(&c)),
        "Expected one to four supported label characters"
    );
    let mut rgba = background();
    let ink = match shape {
        "filter" => {
            // A white disc of radius 13.5, anti-aliased with 4 x 4 samples.
            for y in 5..35 {
                for x in 5..35 {
                    let inside = (0..16)
                        .filter(|sample| {
                            let sx = x as f32 + (sample % 4) as f32 / 4.0 + 0.125 - 20.0;
                            let sy = y as f32 + (sample / 4) as f32 / 4.0 + 0.125 - 20.0;
                            sx * sx + sy * sy <= 13.5 * 13.5
                        })
                        .count();
                    blend(&mut rgba, x, y, 255, inside as f32 / 16.0);
                }
            }
            30
        }
        "film" => {
            for top in [6, 31] {
                for left in [6, 12, 18, 24, 30] {
                    for y in top..top + 3 {
                        for x in left..left + 3 {
                            dot(&mut rgba, x, y, 230);
                        }
                    }
                }
            }
            255
        }
        "plain" => 255,
        _ => bail!("Expected film, filter, or plain tile shape"),
    };
    // Text inside the disc keeps clear of its curved edge.
    let (size, width) = if shape == "filter" {
        (17.0, 19.0)
    } else {
        (LABEL_SIZE, LABEL_WIDTH)
    };
    text(&mut rgba, label, size, width, 20.0, ink);
    Ok(rgba)
}

/// Inter Black 4.1, from the same release and license as `FONT`. Only the
/// cover icons use it.
const COVER_FONT: &[u8] = include_bytes!("../assets/Inter-Black.ttf");
/// Cover shades: the light bands, the dark text on them, and the body.
const LIGHT: u8 = 214;
const DARK: u8 = 40;
const BODY: u8 = 51;
/// The cover rows, with a 3-pixel transparent margin above and below. The cover spans the full tile width, so that a name
/// such as KODACHROME fits on one line.
const COVER: i32 = 3;
const COVER_END: i32 = 37;
const COVER_WIDTH: i32 = 40;
/// A cover text may compress to this horizontal scale before it is rejected.
const MIN_COMPRESSION: f32 = 0.75;

fn fill(rgba: &mut [u8], top: i32, bottom: i32, shade: u8) {
    for y in top..bottom {
        for x in 0..COVER_WIDTH {
            dot(rgba, x as usize, y as usize, shade);
        }
    }
}

/// Draw `label` in Inter Black, centered on x = 20, with `capital` pixels of
/// capital height on the `baseline` row. The text compresses horizontally to
/// fit `width` pixels. Returns `None`, and draws nothing, if the text would
/// need a horizontal scale below `min_compression`.
#[allow(clippy::too_many_arguments)]
fn cover_text(
    rgba: &mut [u8],
    label: &str,
    capital: f32,
    baseline: i32,
    tracking: f32,
    width: f32,
    min_compression: f32,
    ink: u8,
) -> Option<()> {
    let font = FontRef::try_from_slice(COVER_FONT).expect("embedded font is valid");
    let unit = PxScale::from(100.0);
    let size = 100.0 * capital
        / font
            .outline_glyph(font.glyph_id('H').with_scale(unit))
            .expect("the font has H")
            .px_bounds()
            .height();
    let mut compression = 1.0f32;
    loop {
        let scale = PxScale {
            x: size * compression,
            y: size,
        };
        let scaled = font.as_scaled(scale);
        let mut caret = 0.0;
        let outlines: Vec<_> = label
            .chars()
            .filter_map(|character| {
                let id = scaled.glyph_id(character);
                let glyph = id.with_scale_and_position(scale, point(caret, 0.0));
                caret += scaled.h_advance(id) + tracking * size * compression;
                font.outline_glyph(glyph)
            })
            .collect();
        let left = outlines
            .iter()
            .map(|o| o.px_bounds().min.x)
            .reduce(f32::min)?;
        let right = outlines
            .iter()
            .map(|o| o.px_bounds().max.x)
            .reduce(f32::max)?;
        let actual = right - left;
        if actual > width {
            compression *= width / actual * 0.999;
            if compression < min_compression {
                return None;
            }
            continue;
        }
        // Whole-pixel offsets keep the stems sharp, as in `text`.
        let dx = (20.0 - actual / 2.0 - left).round() as i32;
        for outline in outlines {
            let origin = outline.px_bounds().min;
            outline.draw(|x, y, coverage| {
                blend(
                    rgba,
                    origin.x as i32 + x as i32 + dx,
                    origin.y as i32 + y as i32 + baseline,
                    ink,
                    coverage,
                )
            });
        }
        return Some(());
    }
}

/// Five-pixel capitals for cover text too small for an outline font.
/// Each string is one row, and `#` is ink.
fn pixel_glyph(character: char) -> Option<[&'static str; 5]> {
    Some(match character {
        'A' => [".#.", "#.#", "###", "#.#", "#.#"],
        'B' => ["##.", "#.#", "##.", "#.#", "##."],
        'C' => [".##", "#..", "#..", "#..", ".##"],
        'D' => ["##.", "#.#", "#.#", "#.#", "##."],
        'E' => ["###", "#..", "##.", "#..", "###"],
        'F' => ["###", "#..", "##.", "#..", "#.."],
        'G' => [".##", "#..", "#.#", "#.#", ".##"],
        'H' => ["#.#", "#.#", "###", "#.#", "#.#"],
        'I' => ["#", "#", "#", "#", "#"],
        'J' => ["..#", "..#", "..#", "#.#", ".#."],
        'K' => ["#.#", "#.#", "##.", "#.#", "#.#"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'M' => ["#...#", "##.##", "#.#.#", "#...#", "#...#"],
        'N' => ["#..#", "##.#", "#.##", "#..#", "#..#"],
        'O' => [".#.", "#.#", "#.#", "#.#", ".#."],
        'P' => ["##.", "#.#", "##.", "#..", "#.."],
        'Q' => [".#.", "#.#", "#.#", "##.", ".##"],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'S' => [".##", "#..", ".#.", "..#", "##."],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        'V' => ["#.#", "#.#", "#.#", "#.#", ".#."],
        'W' => ["#...#", "#...#", "#.#.#", "##.##", "#...#"],
        'X' => ["#.#", "#.#", ".#.", "#.#", "#.#"],
        'Y' => ["#.#", "#.#", ".#.", ".#.", ".#."],
        'Z' => ["###", "..#", ".#.", "#..", "###"],
        '0' => ["###", "#.#", "#.#", "#.#", "###"],
        '1' => [".#", "##", ".#", ".#", ".#"],
        '2' => ["##.", "..#", ".#.", "#..", "###"],
        '3' => ["##.", "..#", ".#.", "..#", "##."],
        '4' => ["#.#", "#.#", "###", "..#", "..#"],
        '5' => ["###", "#..", "##.", "..#", "##."],
        '6' => [".##", "#..", "###", "#.#", "###"],
        '7' => ["###", "..#", ".#.", ".#.", ".#."],
        '8' => ["###", "#.#", "###", "#.#", "###"],
        '9' => ["###", "#.#", "###", "..#", "##."],
        '-' => ["..", "..", "##", "..", ".."],
        '+' => ["...", ".#.", "###", ".#.", "..."],
        ' ' => [".", ".", ".", ".", "."],
        _ => return None,
    })
}

/// Draw `label` in the pixel font, centered on x = 20, with its top row at `top`.
/// Narrow forms of the five-pixel `M` and `W`, for a text that does not fit
/// with the regular forms. They read less clearly, so only such a text uses them.
fn narrow_glyph(character: char) -> Option<[&'static str; 5]> {
    match character {
        'M' => Some(["#.#", "###", "###", "#.#", "#.#"]),
        'W' => Some(["#.#", "#.#", "###", "###", "#.#"]),
        other => pixel_glyph(other),
    }
}

/// The width of glyphs with one blank column between them.
fn pixel_width(glyphs: &[[&str; 5]]) -> i32 {
    glyphs
        .iter()
        .map(|rows| rows[0].len() as i32 + 1)
        .sum::<i32>()
        - 1
}

fn pixel_text(rgba: &mut [u8], label: &str, top: i32, ink: u8) -> Result<()> {
    let glyphs_of = |glyph: fn(char) -> Option<[&'static str; 5]>| {
        label
            .chars()
            .map(glyph)
            .collect::<Option<Vec<_>>>()
            .context("Expected capitals, digits, spaces, '-', or '+' in the cover text")
    };
    let mut glyphs = glyphs_of(pixel_glyph)?;
    if pixel_width(&glyphs) > COVER_WIDTH {
        glyphs = glyphs_of(narrow_glyph)?;
    }
    let width = pixel_width(&glyphs);
    ensure!(
        width <= COVER_WIDTH,
        "Cover text {label:?} is {width} pixels wide, more than {COVER_WIDTH}"
    );
    let mut x = 20 - (width + 1) / 2;
    for rows in glyphs {
        for (dy, row) in rows.iter().enumerate() {
            for (dx, bit) in row.bytes().enumerate() {
                if bit == b'#' {
                    blend(rgba, x + dx as i32, top + dy as i32, ink, 1.0);
                }
            }
        }
        x += rows[0].len() as i32 + 1;
    }
    Ok(())
}

fn check_cover_label(label: &str, what: &str) -> Result<()> {
    ensure!(
        (1..=12).contains(&label.len()) && label.bytes().all(|c| c.is_ascii_graphic() || c == b' '),
        "Expected 1 to 12 printable ASCII characters in the cover {what}"
    );
    Ok(())
}

/// The bottom of a cover.
#[derive(Clone, Copy)]
enum Bottom {
    /// The dark body continues to the bottom row.
    None,
    /// A light band, as on professional film boxes.
    Band,
    /// Three light stripes, as on motion picture film labels.
    Stripes,
}

/// A gray cover: a light band with the brand, then a dark body with the film
/// name and a large number. The brand uses Inter Black when it fits without
/// compression, and the pixel font otherwise. The name always uses the pixel
/// font.
fn cover(brand: &str, name: &str, number: &str, bottom: Bottom) -> Result<Vec<u8>> {
    check_cover_label(brand, "brand")?;
    check_cover_label(name, "name")?;
    check_cover_label(number, "number")?;
    let mut rgba = vec![0; 40 * 40 * 4];
    fill(&mut rgba, COVER, COVER_END, BODY);
    fill(&mut rgba, COVER, 12, LIGHT);
    if cover_text(&mut rgba, brand, 6.0, 10, -0.04, 28.0, 1.0, DARK).is_none() {
        pixel_text(&mut rgba, brand, 5, DARK)?;
    }
    // The number sits one row lower without a bottom band, and one row
    // higher above the stripes.
    let (name_top, baseline) = match bottom {
        Bottom::None => (15, 33),
        Bottom::Band => {
            fill(&mut rgba, 34, COVER_END, LIGHT);
            (14, 32)
        }
        Bottom::Stripes => {
            for row in [32, 34, 36] {
                fill(&mut rgba, row, row + 1, LIGHT);
            }
            (14, 31)
        }
    };
    pixel_text(&mut rgba, name, name_top, 255)?;
    cover_text(
        &mut rgba,
        number,
        11.0,
        baseline,
        -0.04,
        34.0,
        MIN_COMPRESSION,
        255,
    )
    .with_context(|| format!("Cover number {number:?} does not fit"))?;
    Ok(rgba)
}

/// A still film box. Professional films have a light band at the bottom.
pub fn render_box(brand: &str, name: &str, number: &str, band: bool) -> Result<Vec<u8>> {
    cover(
        brand,
        name,
        number,
        if band { Bottom::Band } else { Bottom::None },
    )
}

/// A motion picture film label, with three light stripes at the bottom.
pub fn render_cine(brand: &str, name: &str, number: &str) -> Result<Vec<u8>> {
    cover(brand, name, number, Bottom::Stripes)
}

pub fn off_pixels() -> Vec<u8> {
    render_tile("OFF", "plain").expect("constant tile is valid")
}

/// A panoramic frame inside the same tile used by the other module entries.
pub fn framing_pixels() -> Vec<u8> {
    let mut rgba = background();
    for y in 12..28 {
        for x in 7..33 {
            let edge = x == 7 || x == 32 || y == 12 || y == 27;
            dot(&mut rgba, x, y, if edge { 255 } else { 20 });
        }
    }
    rgba
}

/// A frame with the proportions of `width` x `height`, centered above its label.
fn ratio_pixels(label: &str, width: usize, height: usize) -> Vec<u8> {
    let mut rgba = background();
    let (left, top) = ((40 - width) / 2, 6 + (19 - height) / 2);
    for y in top..top + height {
        for x in left..left + width {
            let edge = x == left || x == left + width - 1 || y == top || y == top + height - 1;
            dot(&mut rgba, x, y, if edge { 255 } else { 20 });
        }
    }
    text(&mut rgba, label, 11.0, LABEL_WIDTH, 30.5, 255);
    rgba
}

/// Framing artwork in runtime order: the module tile, then the XPan (65:24),
/// 16:9, and 4:3 ratios.
pub fn framing_tiles() -> Vec<Vec<u8>> {
    vec![
        framing_pixels(),
        ratio_pixels("XPan", 30, 11),
        ratio_pixels("16:9", 30, 17),
        ratio_pixels("4:3", 24, 18),
    ]
}

/// Composition guide styles, in runtime order after Off.
#[derive(Clone, Copy)]
enum Guide {
    Thirds,
    Center,
    Golden,
    Diagonals,
    ThirdsCenter,
}

/// A frame of about 3:2 with its guide lines, centered horizontally. Lines are
/// gray, so they stay distinct from the frame.
fn guide_frame(rgba: &mut [u8], top: usize, guide: Guide) {
    let (width, height) = (28, 18);
    let left = (40 - width) / 2;
    let (right, bottom) = (left + width - 1, top + height - 1);
    for y in top..=bottom {
        for x in left..=right {
            let edge = x == left || x == right || y == top || y == bottom;
            dot(rgba, x, y, if edge { 255 } else { 20 });
        }
    }
    // Symmetric line offsets from the left and top edges.
    let grid = |rgba: &mut [u8], column: usize, row: usize| {
        for x in [left + column, right - column] {
            (top + 1..bottom).for_each(|y| dot(rgba, x, y, 170));
        }
        for y in [top + row, bottom - row] {
            (left + 1..right).for_each(|x| dot(rgba, x, y, 170));
        }
    };
    // Even dimensions center the cross between two pixel rows and columns.
    // `reach` is the arm length from the center pair.
    let cross = |rgba: &mut [u8], reach: (usize, usize)| {
        for offset in [0, 1] {
            (left + 13 - reach.0..=left + 14 + reach.0)
                .for_each(|x| dot(rgba, x, top + 8 + offset, 170));
            (top + 8 - reach.1..=top + 9 + reach.1)
                .for_each(|y| dot(rgba, left + 13 + offset, y, 170));
        }
    };
    match guide {
        Guide::Thirds => grid(rgba, 9, 6),
        Guide::Golden => grid(rgba, 10, 7),
        Guide::Center => cross(rgba, (4, 4)),
        // A short cross inside the center cell of the grid.
        Guide::ThirdsCenter => {
            grid(rgba, 9, 6);
            cross(rgba, (2, 1));
        }
        Guide::Diagonals => {
            for x in 1..width - 1 {
                // Round half up, as the runtime's Bresenham lines do.
                let y = (2 * x * (height - 1) + width - 1) / (2 * (width - 1));
                dot(rgba, left + x, top + y, 170);
                dot(rgba, right - x, top + y, 170);
            }
        }
    }
}

/// A guide frame centered above its label.
fn guide_pixels(label: &str, guide: Guide) -> Vec<u8> {
    let mut rgba = background();
    guide_frame(&mut rgba, 6, guide);
    text(&mut rgba, label, 11.0, LABEL_WIDTH, 30.5, 255);
    rgba
}

/// Guides artwork in runtime order: the module tile, then thirds, center,
/// golden ratio, diagonals, and thirds with center.
pub fn guides_tiles() -> Vec<Vec<u8>> {
    let mut module = background();
    guide_frame(&mut module, 11, Guide::Thirds);
    vec![
        module,
        guide_pixels("3x3", Guide::Thirds),
        guide_pixels("Ctr", Guide::Center),
        guide_pixels("Phi", Guide::Golden),
        guide_pixels("Diag", Guide::Diagonals),
        guide_pixels("3x3+", Guide::ThirdsCenter),
    ]
}

/// Date stamp artwork in runtime order: the module tile, then the 'YY M D,
/// 'YY M D H:MM, M D 'YY, D M 'YY, and D M 'YY H:MM styles, each named by a
/// sample. The last tile has two lines: the day-first order and the time.
pub fn stamp_tiles() -> Vec<Vec<u8>> {
    let mut tiles: Vec<_> = ["DATE", "'26", "9:05", "M D", "D M"]
        .iter()
        .map(|label| {
            let mut rgba = background();
            text(&mut rgba, label, LABEL_SIZE, LABEL_WIDTH, 20.0, 255);
            rgba
        })
        .collect();
    let mut rgba = background();
    text(&mut rgba, "D M", 14.0, LABEL_WIDTH, 13.5, 255);
    text(&mut rgba, "9:05", 14.0, LABEL_WIDTH, 26.5, 255);
    tiles.push(rgba);
    tiles
}

/// Light speckles in rows `top..bottom`, one in `period` pixels on average.
/// A fixed hash places them, so the artwork stays deterministic.
fn speckles(rgba: &mut [u8], top: usize, bottom: usize, period: u32) {
    for y in top..bottom {
        for x in 6..34 {
            let hash = (x as u32)
                .wrapping_mul(0x9e37_79b1)
                .wrapping_add((y as u32).wrapping_mul(0x85eb_ca77))
                .rotate_left(13)
                .wrapping_mul(0xc2b2_ae3d);
            if (hash >> 16).is_multiple_of(period) {
                dot(rgba, x, y, 220);
            }
        }
    }
}

/// Grain artwork in runtime order: the module tile, then the weak, medium,
/// and strong levels, with more speckles at each level.
pub fn grain_tiles() -> Vec<Vec<u8>> {
    let mut module = background();
    speckles(&mut module, 6, 34, 4);
    let mut tiles = vec![module];
    for (label, period) in [("Weak", 9), ("Med", 5), ("Strong", 3)] {
        let mut rgba = background();
        speckles(&mut rgba, 6, 24, period);
        text(&mut rgba, label, 11.0, LABEL_WIDTH, 30.5, 255);
        tiles.push(rgba);
    }
    tiles
}

/// A bright core with a soft halo of dots around it: the halo widens and
/// brightens with `strength` (0..3).
fn halo(rgba: &mut [u8], cx: usize, cy: usize, strength: u32) {
    let radius = 3 + strength * 3;
    for dy in -(radius as i32)..=(radius as i32) {
        for dx in -(radius as i32)..=(radius as i32) {
            let d2 = (dx * dx + dy * dy) as u32;
            let r2 = radius * radius;
            if d2 > r2 {
                continue;
            }
            let alpha = if d2 <= 2 {
                255
            } else {
                220 - (160 * d2 / r2.max(1)) as u8
            };
            let (x, y) = ((cx as i32 + dx) as usize, (cy as i32 + dy) as usize);
            if x < 40 && y < 40 {
                dot(rgba, x, y, alpha);
            }
        }
    }
}

/// Diffusion artwork in runtime order: the module tile, then the weak,
/// medium, and strong levels, with a wider halo at each level.
pub fn diffusion_tiles() -> Vec<Vec<u8>> {
    let mut module = background();
    halo(&mut module, 20, 17, 2);
    let mut tiles = vec![module];
    for (label, strength) in [("Weak", 1u32), ("Med", 2), ("Strong", 3)] {
        let mut rgba = background();
        halo(&mut rgba, 20, 14, strength);
        text(&mut rgba, label, 11.0, LABEL_WIDTH, 30.5, 255);
        tiles.push(rgba);
    }
    tiles
}

/// The GRKit entry of the ADJ toolbar.
pub fn plugins_pixels() -> Vec<u8> {
    let mut rgba = toolbar_background();
    text(&mut rgba, "GRK", LABEL_SIZE, LABEL_WIDTH, 20.0, 255);
    rgba
}

/// The About row, which gives the version. Its own tile keeps a second GRK
/// tile out of the GRKit root.
pub fn version_pixels() -> Vec<u8> {
    let mut rgba = background();
    text(&mut rgba, "VER", LABEL_SIZE, LABEL_WIDTH, 20.0, 255);
    rgba
}
