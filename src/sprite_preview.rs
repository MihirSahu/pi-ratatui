use std::{fs, path::Path};

use color_eyre::eyre::Result;
use image::{ImageBuffer, Rgba, RgbaImage};

use crate::clawd;

const CELL_WIDTH: u32 = 9;
const CELL_HEIGHT: u32 = 18;
const GAP: u32 = 18;
const PADDING: u32 = 18;
const BACKGROUND: Rgba<u8> = Rgba([24, 24, 24, 255]);
const BODY: Rgba<u8> = Rgba([216, 122, 88, 255]);
const LAPTOP: Rgba<u8> = Rgba([84, 88, 96, 255]);
const SCREEN: Rgba<u8> = Rgba([54, 148, 218, 255]);
const CODE: Rgba<u8> = Rgba([154, 230, 180, 255]);
const SUIT: Rgba<u8> = Rgba([28, 32, 40, 255]);
const SHIRT: Rgba<u8> = Rgba([238, 238, 228, 255]);

pub fn export_all(out_dir: impl AsRef<Path>) -> Result<()> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    export_sheet(
        out_dir.join("walk.png"),
        clawd::FRAMES.as_slice(),
        clawd::WIDTH,
        clawd::HEIGHT,
    )?;
    export_sheet(
        out_dir.join("coding.png"),
        clawd::CODING_FRAMES.as_slice(),
        clawd::CODING_WIDTH,
        clawd::CODING_HEIGHT,
    )?;
    export_sheet(
        out_dir.join("suit.png"),
        clawd::SUIT_FRAMES.as_slice(),
        clawd::SUIT_WIDTH,
        clawd::SUIT_HEIGHT,
    )?;

    Ok(())
}

fn export_sheet(path: impl AsRef<Path>, frames: &[&str], width: u16, height: u16) -> Result<()> {
    let frame_width = u32::from(width) * CELL_WIDTH;
    let frame_height = u32::from(height) * CELL_HEIGHT;
    let image_width =
        PADDING * 2 + frame_width * frames.len() as u32 + GAP * (frames.len() as u32 - 1);
    let image_height = PADDING * 2 + frame_height;
    let mut image = ImageBuffer::from_pixel(image_width, image_height, BACKGROUND);

    for (index, frame) in frames.iter().enumerate() {
        let x = PADDING + index as u32 * (frame_width + GAP);
        draw_frame(&mut image, frame, x, PADDING);
    }

    image.save(path)?;

    Ok(())
}

fn draw_frame(image: &mut RgbaImage, frame: &str, x: u32, y: u32) {
    for (row, pixels) in frame.lines().enumerate() {
        for (col, pixel) in pixels.chars().enumerate() {
            let Some(color) = color_for(pixel) else {
                continue;
            };

            fill_cell(
                image,
                x + col as u32 * CELL_WIDTH,
                y + row as u32 * CELL_HEIGHT,
                color,
            );
        }
    }
}

fn fill_cell(image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    for cell_y in y..y + CELL_HEIGHT {
        for cell_x in x..x + CELL_WIDTH {
            image.put_pixel(cell_x, cell_y, color);
        }
    }
}

fn color_for(pixel: char) -> Option<Rgba<u8>> {
    match pixel {
        '#' | '!' => Some(BODY),
        '@' => Some(LAPTOP),
        '+' => Some(SCREEN),
        '*' => Some(CODE),
        'S' => Some(SUIT),
        '%' => Some(SHIRT),
        _ => None,
    }
}
