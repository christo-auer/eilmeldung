use std::borrow::Cow;

// use crate::prelude::*;
use crate::prelude::*;
use image::{DynamicImage, Pixel, Rgb, RgbaImage};
use ratatui::style::Color;

// adapted from https://github.com/ratatui/ratzilla/
/// Converts an indexed color (0-255) to an RGB value.
fn indexed_color_to_rgb(index: u8) -> Rgb<u8> {
    match index {
        // Basic 16 colors (0-15)
        0..=15 => {
            const BASIC_COLORS: [Rgb<u8>; 16] = [
                Rgb::<u8>([0x00, 0x00, 0x00]), // 0: black
                Rgb::<u8>([0xCD, 0x00, 0x00]), // 1: red
                Rgb::<u8>([0x00, 0xCD, 0x00]), // 2: green
                Rgb::<u8>([0xCD, 0xCD, 0x00]), // 3: yellow
                Rgb::<u8>([0x00, 0x00, 0xEE]), // 4: blue
                Rgb::<u8>([0xCD, 0x00, 0xCD]), // 5: magenta
                Rgb::<u8>([0x00, 0xCD, 0xCD]), // 6: cyan
                Rgb::<u8>([0xE5, 0xE5, 0xE5]), // 7: white
                Rgb::<u8>([0x7F, 0x7F, 0x7F]), // 8: bright Black
                Rgb::<u8>([0xFF, 0x00, 0x00]), // 9: bright Red
                Rgb::<u8>([0x00, 0xFF, 0x00]), // 10: bright Green
                Rgb::<u8>([0xFF, 0xFF, 0x00]), // 11: bright Yellow
                Rgb::<u8>([0x5C, 0x5C, 0xFF]), // 12: bright Blue
                Rgb::<u8>([0xFF, 0x00, 0xFF]), // 13: bright Magenta
                Rgb::<u8>([0x00, 0xFF, 0xFF]), // 14: bright Cyan
                Rgb::<u8>([0xFF, 0xFF, 0xFF]), // 15: bright White
            ];
            BASIC_COLORS[index as usize]
        }

        // 216-color cube (16-231)
        16..=231 => {
            let cube_index = index - 16;
            let r = cube_index / 36;
            let g = (cube_index % 36) / 6;
            let b = cube_index % 6;

            // Convert 0-5 range to 0-255 RGB
            // Values: 0 -> 0, 1 -> 95, 2 -> 135, 3 -> 175, 4 -> 215, 5 -> 255
            let to_rgb = |n: u8| -> u8 { if n == 0 { 0 } else { 55 + 40 * n } };

            Rgb::<u8>([to_rgb(r), to_rgb(g), to_rgb(b)])
        }

        // 24 grayscale colors (232-255)
        232..=255 => {
            let gray_index = index - 232;
            // linear interpolation from 8 to 238
            let gray = 8 + gray_index * 10;
            Rgb::<u8>([gray, gray, gray])
        }
    }
}
fn color_to_rgb(color: Color) -> Rgb<u8> {
    use Color as C;
    match color {
        C::Reset | C::Black => Rgb::<u8>([0, 0, 0]),
        C::Red => Rgb::<u8>([128, 0, 0]),
        C::Green => Rgb::<u8>([0, 128, 0]),
        C::Yellow => Rgb::<u8>([128, 128, 0]),
        C::Blue => Rgb::<u8>([0, 0, 128]),
        C::Magenta => Rgb::<u8>([128, 0, 128]),
        C::Cyan => Rgb::<u8>([0, 128, 128]),
        C::Gray => Rgb::<u8>([128, 128, 128]),
        C::DarkGray => Rgb::<u8>([64, 64, 64]),
        C::LightRed => Rgb::<u8>([255, 0, 0]),
        C::LightGreen => Rgb::<u8>([0, 255, 0]),
        C::LightYellow => Rgb::<u8>([255, 255, 0]),
        C::LightBlue => Rgb::<u8>([0, 0, 255]),
        C::LightMagenta => Rgb::<u8>([255, 0, 255]),
        C::LightCyan => Rgb::<u8>([0, 255, 255]),
        C::White => Rgb::<u8>([255, 255, 255]),
        C::Rgb(r, g, b) => Rgb::<u8>([r, g, b]),
        C::Indexed(index) => indexed_color_to_rgb(index),
    }
}

pub fn round_corners(img: &mut RgbaImage, radius: u32) {
    let (width_f, height_f) = (img.width() as f32, img.height() as f32);
    round_corners_with_shift(img, 0.0, 0.0, width_f, height_f, radius as f32);
}

pub fn round_corners_with_shift(
    img: &mut RgbaImage,
    shift_x: f32,
    shift_y: f32,
    width_f: f32,
    height_f: f32,
    radius_f: f32,
) {
    let radius_f = radius_f.min(width_f / 2.0).min(height_f / 2.0).max(0.0);

    let (left, top) = (shift_x, shift_y);
    let (right, bottom) = (left + width_f, top + height_f);

    for (x, y, p) in img.enumerate_pixels_mut() {
        let (middle_x, middle_y) = (x as f32 + 0.5, y as f32 + 0.5);
        let (cx, cy) = (
            middle_x.clamp(left + radius_f, right - radius_f),
            middle_y.clamp(top + radius_f, bottom - radius_f),
        );

        let distance = ((middle_x - cx).powi(2) + (middle_y - cy).powi(2)).sqrt();
        let alpha = (radius_f - distance + 0.5).clamp(0.0, 1.0);

        p.0[3] = (p.0[3] as f32 * alpha) as u8;
    }
}

pub fn add_drop_shadow(
    img: &DynamicImage,
    color: Color,
    shadow_width: u32,
    border_radius: u32,
) -> DynamicImage {
    let (width, height) = (img.width() + shadow_width, img.height() + shadow_width);

    let mut border_color = color_to_rgb(color).to_rgba();
    border_color.0[3] = 255;

    let mut background = RgbaImage::from_pixel(width, height, border_color);
    round_corners_with_shift(
        &mut background,
        shadow_width as f32,
        shadow_width as f32,
        img.width() as f32,
        img.height() as f32,
        border_radius as f32,
    );

    let mut rgba_image = img.to_rgba8();

    round_corners(&mut rgba_image, border_radius);

    image::imageops::overlay(&mut background, &rgba_image, 0, 0);

    DynamicImage::ImageRgba8(background)
}

pub fn apply_image_enhancements<'a>(
    img: &'a DynamicImage,
    picker: &ratatui_image::picker::Picker,
    config: &Config,
) -> Cow<'a, DynamicImage> {
    if config.article_image_drop_shadow || config.article_images_rounded_corners {
        Cow::Owned(add_drop_shadow(
            img,
            config.theme.color_palette().image_drop_shadow,
            to_drop_shadow_width(config, picker),
            to_corner_radius(config, picker),
        ))
    } else {
        Cow::Borrowed(img)
    }
}

pub fn to_drop_shadow_width(config: &Config, picker: &ratatui_image::picker::Picker) -> u32 {
    if config.article_image_drop_shadow {
        (picker.font_size().width / 3) as u32
    } else {
        0
    }
}

pub fn to_corner_radius(config: &Config, picker: &ratatui_image::picker::Picker) -> u32 {
    if config.article_images_rounded_corners {
        (picker.font_size().width / 3) as u32
    } else {
        0
    }
}
