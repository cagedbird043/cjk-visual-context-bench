use image::{GrayImage, Luma};
use rusttype::{Font, Scale, point};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

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

#[derive(Debug, Serialize)]
pub struct ArchiveRenderManifest {
    pub name: String,
    pub source: String,
    pub compact_chars: usize,
    pub frame_size: u32,
    pub content_width: u32,
    pub content_height: u32,
    pub frame_count: usize,
    pub chars_per_frame: f64,
    pub font_path: String,
    pub font_size: f32,
    pub threshold: f32,
    pub line_spacing: f32,
    pub margin: f32,
    pub frames: Vec<ArchiveFrame>,
}

#[derive(Debug, Serialize)]
pub struct ArchiveFrame {
    pub path: String,
    pub chars: usize,
}

pub fn run_render(args: &[String]) -> Result<(), Box<dyn Error>> {
    let text_path = required_arg(args, "--text")?;
    let output = required_arg(args, "--out")?;
    let text = compact_text(&fs::read_to_string(text_path)?);
    let variant = RenderVariant {
        name: "manual".to_string(),
        font_path: arg_value(args, "--font")
            .unwrap_or("fonts/zpix.ttf")
            .to_string(),
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
pub fn run_render_archive(args: &[String]) -> Result<(), Box<dyn Error>> {
    let text_path = required_arg(args, "--text")?;
    let out_dir = required_arg(args, "--out-dir")?;
    let name = arg_value(args, "--name").unwrap_or("archive").to_string();
    let frame_size = parse_arg(args, "--frame-size", 1568.0)? as u32;
    let content_width = parse_arg(args, "--content-width", frame_size as f32)? as u32;
    let content_height = parse_arg(args, "--content-height", frame_size as f32)? as u32;
    let text = compact_archive_text(&fs::read_to_string(text_path)?);
    let variant = RenderVariant {
        name: name.clone(),
        font_path: arg_value(args, "--font")
            .unwrap_or("fonts/zpix.ttf")
            .to_string(),
        font_size: parse_arg(args, "--font-size", 12.0)?,
        threshold: parse_arg(args, "--threshold", 0.3)?,
        line_spacing: parse_arg(args, "--line-spacing", 0.0)?,
        max_width: frame_size as f32,
        margin: parse_arg(args, "--margin", 8.0)?,
    };
    let manifest = render_archive_frames(
        out_dir,
        text_path,
        &text,
        frame_size,
        content_width,
        content_height,
        &variant,
    )?;
    println!(
        "rendered archive {} frames={} chars={} chars/frame={:.1}",
        manifest.name, manifest.frame_count, manifest.compact_chars, manifest.chars_per_frame
    );
    Ok(())
}

pub fn render_archive_frames(
    out_dir: &str,
    source: &str,
    text: &str,
    frame_size: u32,
    content_width: u32,
    content_height: u32,
    variant: &RenderVariant,
) -> Result<ArchiveRenderManifest, Box<dyn Error>> {
    fs::create_dir_all(out_dir)?;
    let font_data = fs::read(&variant.font_path)?;
    let font = Font::try_from_bytes(&font_data).ok_or("could not parse font")?;
    let scale = Scale::uniform(variant.font_size);
    let v_metrics = font.v_metrics(scale);
    let line_height = (v_metrics.ascent - v_metrics.descent).ceil() + variant.line_spacing;
    let right_limit =
        (variant.margin + content_width as f32).min(frame_size as f32 - variant.margin);
    let bottom_limit =
        (variant.margin + content_height as f32).min(frame_size as f32 - variant.margin);
    let compact_chars = text.chars().count();
    let mut frames = Vec::new();
    let mut image = GrayImage::from_pixel(frame_size, frame_size, Luma([255]));
    let mut current_x = variant.margin;
    let mut current_y = v_metrics.ascent + variant.margin;
    let mut frame_chars = 0usize;
    let mut frame_index = 0usize;

    for c in text.chars() {
        let scaled_glyph = font.glyph(c).scaled(scale);
        let advance_width = scaled_glyph.h_metrics().advance_width;
        if current_x + advance_width > right_limit {
            current_x = variant.margin;
            current_y += line_height;
        }
        if current_y + v_metrics.descent + variant.margin > bottom_limit && frame_chars > 0 {
            save_archive_frame(out_dir, frame_index, image, frame_chars, &mut frames)?;
            frame_index += 1;
            image = GrayImage::from_pixel(frame_size, frame_size, Luma([255]));
            current_x = variant.margin;
            current_y = v_metrics.ascent + variant.margin;
            frame_chars = 0;
        }
        let glyph = scaled_glyph.positioned(point(current_x, current_y));
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
        current_x += advance_width;
        frame_chars += 1;
    }
    if frame_chars > 0 || frames.is_empty() {
        save_archive_frame(out_dir, frame_index, image, frame_chars, &mut frames)?;
    }
    let frame_count = frames.len();
    let chars_per_frame = if frame_count == 0 {
        0.0
    } else {
        compact_chars as f64 / frame_count as f64
    };
    let manifest = ArchiveRenderManifest {
        name: variant.name.clone(),
        source: source.to_string(),
        compact_chars,
        frame_size,
        content_width,
        content_height,
        frame_count,
        chars_per_frame,
        font_path: variant.font_path.clone(),
        font_size: variant.font_size,
        threshold: variant.threshold,
        line_spacing: variant.line_spacing,
        margin: variant.margin,
        frames,
    };
    fs::write(
        Path::new(out_dir).join("manifest.json"),
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    )?;
    Ok(manifest)
}

fn save_archive_frame(
    out_dir: &str,
    frame_index: usize,
    image: GrayImage,
    chars: usize,
    frames: &mut Vec<ArchiveFrame>,
) -> Result<(), Box<dyn Error>> {
    let filename = format!("frame-{frame_index:04}.png");
    let path = PathBuf::from(out_dir).join(&filename);
    image.save(&path)?;
    frames.push(ArchiveFrame {
        path: filename,
        chars,
    });
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

    if let Some(parent) = Path::new(output).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    image.save(output)?;
    Ok(RenderedImage {
        width: img_width,
        height: img_height,
    })
}

pub fn default_font_path() -> String {
    "fonts/zpix.ttf".to_string()
}

pub fn default_margin() -> f32 {
    15.0
}

fn compact_text(text: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for line in text.lines() {
        let trimmed = line.trim().trim_start_matches('#').trim_start();
        if trimmed.is_empty() {
            continue;
        }
        for c in trimmed.chars() {
            if c.is_whitespace() {
                pending_space = true;
                continue;
            }
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        }
        pending_space = true;
    }
    out
}

fn compact_archive_text(text: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(c);
    }
    out
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
