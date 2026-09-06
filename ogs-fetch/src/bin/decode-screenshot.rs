use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::fs;
use std::io::{self, BufWriter, Read};
use std::path::Path;

const EXPECTED_FORMAT: &str = "INKPLATE_1BIT_LSB_FIRST_BLACK_1";

struct Screenshot {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

struct PngImage {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

fn parse_screenshot(text: &str) -> Result<Screenshot, String> {
    let mut in_screenshot = false;
    let mut in_data = false;
    let mut width = None;
    let mut height = None;
    let mut byte_count = None;
    let mut format = None;
    let mut data = String::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();

        if line == "SCREENSHOT_BEGIN" {
            in_screenshot = true;
            continue;
        }

        if line == "SCREENSHOT_END" {
            break;
        }

        if !in_screenshot {
            continue;
        }

        if line == "DATA_BEGIN" {
            in_data = true;
            continue;
        }

        if line == "DATA_END" {
            in_data = false;
            continue;
        }

        if in_data {
            data.push_str(line);
            continue;
        }

        if let Some(value) = line.strip_prefix("WIDTH ") {
            width = Some(parse_usize(value, "WIDTH")?);
        } else if let Some(value) = line.strip_prefix("HEIGHT ") {
            height = Some(parse_usize(value, "HEIGHT")?);
        } else if let Some(value) = line.strip_prefix("BYTES ") {
            byte_count = Some(parse_usize(value, "BYTES")?);
        } else if let Some(value) = line.strip_prefix("FORMAT ") {
            format = Some(value.to_string());
        }
    }

    let width = width.ok_or("missing WIDTH line")?;
    let height = height.ok_or("missing HEIGHT line")?;
    let byte_count = byte_count.ok_or("missing BYTES line")?;
    let format = format.ok_or("missing FORMAT line")?;

    if format != EXPECTED_FORMAT {
        return Err(format!("unsupported FORMAT: {format}"));
    }

    if data.is_empty() {
        return Err("missing base64 data".to_string());
    }

    let decoded = STANDARD
        .decode(data.as_bytes())
        .map_err(|err| format!("invalid base64 data: {err}"))?;

    if decoded.len() != byte_count {
        return Err(format!(
            "expected {byte_count} decoded bytes, got {}",
            decoded.len()
        ));
    }

    let expected_bytes = width.div_ceil(8) * height;
    if decoded.len() != expected_bytes {
        return Err(format!(
            "{width}x{height} requires {expected_bytes} bytes, got {}",
            decoded.len()
        ));
    }

    Ok(Screenshot {
        width,
        height,
        data: decoded,
    })
}

fn parse_usize(value: &str, name: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|err| format!("invalid {name} value {value:?}: {err}"))
}

fn inkplate_to_grayscale_pixels(screenshot: &Screenshot) -> PngImage {
    let row_bytes = screenshot.width.div_ceil(8);
    let mut output = Vec::with_capacity(screenshot.width * screenshot.height);

    for output_row in 0..screenshot.width {
        for output_col in 0..screenshot.height {
            let raw_row = output_col;
            let raw_col = screenshot.width - output_row - 1;
            let raw_byte = screenshot.data[(raw_row * row_bytes) + (raw_col / 8)];
            let is_black = raw_byte & (1 << (raw_col % 8)) != 0;

            output.push(if is_black { 0 } else { 255 });
        }
    }

    PngImage {
        width: screenshot.height,
        height: screenshot.width,
        pixels: output,
    }
}

fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|err| format!("failed to read stdin: {err}"))?;
        return Ok(input);
    }

    fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))
}

fn write_png(path: &str, screenshot: &Screenshot) -> Result<(), String> {
    let image = inkplate_to_grayscale_pixels(screenshot);
    let file = fs::File::create(Path::new(path))
        .map_err(|err| format!("failed to create {path}: {err}"))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, image.width as u32, image.height as u32);

    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut png_writer = encoder
        .write_header()
        .map_err(|err| format!("failed to write PNG header: {err}"))?;
    png_writer
        .write_image_data(&image.pixels)
        .map_err(|err| format!("failed to write PNG data: {err}"))
}

fn run() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() != 3 {
        return Err(format!(
            "usage: {} <serial-log|-> <output.png>",
            args.first()
                .map(String::as_str)
                .unwrap_or("decode-screenshot")
        ));
    }

    let input = read_input(&args[1])?;
    let screenshot = parse_screenshot(&input)?;
    write_png(&args[2], &screenshot)
}

fn main() {
    if let Err(err) = run() {
        eprintln!("decode-screenshot: {err}");
        std::process::exit(1);
    }
}
