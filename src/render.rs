use image::{GrayImage, Luma};
use rusttype::{Font, Scale, point};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RenderVariant {
    pub name: String,
    #[serde(default = "default_font_path")]
    pub font_path: String,
    pub font_size: f32,
    pub threshold: f32,
    pub line_spacing: f32,
    pub max_width: f32,
    #[serde(default = "default_margin")]
    pub margin: f32,
}

#[derive(Debug)]
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
}

pub fn run_render(args: &[String]) -> Result<(), Box<dyn Error>> {
    let text_path = required_arg(args, "--text")?;
    let output = required_arg(args, "--out")?;
    let text = fs::read_to_string(text_path)?;
    let variant = RenderVariant {
        name: "manual".to_string(),
        font_path: arg_value(args, "--font").unwrap_or("zpix.ttf").to_string(),
        font_size: parse_arg(args, "--font-size", 12.0)?,
        threshold: parse_arg(args, "--threshold", 0.3)?,
        line_spacing: parse_arg(args, "--line-spacing", 6.0)?,
        max_width: parse_arg(args, "--max-width", 750.0)?,
        margin: parse_arg(args, "--margin", default_margin())?,
    };
    let rendered = render_text(output, &text, &variant)?;
    println!(
        "rendered {output} ({}x{}) from {text_path}",
        rendered.width, rendered.height
    );
    Ok(())
}

pub fn render_text(
    output: &str,
    text: &str,
    variant: &RenderVariant,
) -> Result<RenderedImage, Box<dyn Error>> {
    let font_data = fs::read(&variant.font_path)?;
    let font = Font::try_from_bytes(&font_data).ok_or("could not parse font")?;
    let scale = Scale::uniform(variant.font_size);
    let v_metrics = font.v_metrics(scale);
    let line_height = (v_metrics.ascent - v_metrics.descent).ceil() + variant.line_spacing;
    let mut current_x = variant.margin;
    let mut current_y = v_metrics.ascent + variant.margin;
    let mut glyphs = Vec::new();

    for c in text.chars() {
        if c == '\n' {
            current_x = variant.margin;
            current_y += line_height;
            continue;
        }

        let scaled_glyph = font.glyph(c).scaled(scale);
        let advance_width = scaled_glyph.h_metrics().advance_width;
        if current_x + advance_width > variant.max_width {
            current_x = variant.margin;
            current_y += line_height;
        }

        glyphs.push(scaled_glyph.positioned(point(current_x, current_y)));
        current_x += advance_width;
    }

    let img_width = (variant.max_width + variant.margin) as u32;
    let img_height = (current_y + v_metrics.descent).ceil() as u32 + variant.margin as u32;
    let mut image = GrayImage::from_pixel(img_width, img_height, Luma([255]));

    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, coverage| {
                let px = x as i32 + bb.min.x;
                let py = y as i32 + bb.min.y;
                if px >= 0
                    && px < image.width() as i32
                    && py >= 0
                    && py < image.height() as i32
                    && coverage > variant.threshold
                {
                    image.put_pixel(px as u32, py as u32, Luma([0]));
                }
            });
        }
    }

    image.save(output)?;
    Ok(RenderedImage {
        width: img_width,
        height: img_height,
    })
}

pub fn default_font_path() -> String {
    "zpix.ttf".to_string()
}

pub fn default_margin() -> f32 {
    15.0
}

fn required_arg<'a>(args: &'a [String], key: &str) -> Result<&'a str, Box<dyn Error>> {
    arg_value(args, key).ok_or_else(|| format!("missing required {key}").into())
}

fn arg_value<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].as_str())
}

fn parse_arg(args: &[String], key: &str, fallback: f32) -> Result<f32, Box<dyn Error>> {
    match arg_value(args, key) {
        Some(value) => Ok(value.parse()?),
        None => Ok(fallback),
    }
}
