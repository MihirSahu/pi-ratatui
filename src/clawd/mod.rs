use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub const WIDTH: u16 = 16;
pub const HEIGHT: u16 = 6;
pub const CODING_WIDTH: u16 = WIDTH;
pub const CODING_HEIGHT: u16 = HEIGHT;
pub const SUIT_WIDTH: u16 = WIDTH;
pub const SUIT_HEIGHT: u16 = HEIGHT;

const BODY: Color = Color::Rgb(216, 122, 88);
const LAPTOP: Color = Color::Rgb(84, 88, 96);
const SCREEN: Color = Color::Rgb(54, 148, 218);
const CODE: Color = Color::Rgb(154, 230, 180);
const SUIT: Color = Color::Rgb(28, 32, 40);
const SHIRT: Color = Color::Rgb(238, 238, 228);
const TIE: Color = Color::Rgb(216, 122, 88);
pub(crate) const FRAMES: [&str; 4] = [
    include_str!("walk_0.txt"),
    include_str!("walk_1.txt"),
    include_str!("walk_2.txt"),
    include_str!("walk_3.txt"),
];
pub(crate) const CODING_FRAMES: [&str; 3] = [
    include_str!("coding_0.txt"),
    include_str!("coding_1.txt"),
    include_str!("coding_2.txt"),
];
pub(crate) const SUIT_FRAMES: [&str; 4] = [
    include_str!("suit_0.txt"),
    include_str!("suit_1.txt"),
    include_str!("suit_2.txt"),
    include_str!("suit_3.txt"),
];

pub fn frame_count() -> usize {
    FRAMES.len()
}

pub fn coding_frame_count() -> usize {
    CODING_FRAMES.len()
}

pub fn suit_frame_count() -> usize {
    SUIT_FRAMES.len()
}

pub struct Clawd {
    frame: usize,
    scale: u16,
}

impl Clawd {
    pub fn scaled(frame: usize, scale: u16) -> Self {
        Self {
            frame: frame % FRAMES.len(),
            scale: scale.max(1),
        }
    }
}

impl Widget for Clawd {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_pixels(FRAMES[self.frame], area, buf, self.scale);
    }
}

pub struct CodingClawd {
    frame: usize,
    scale: u16,
}

impl CodingClawd {
    pub fn scaled(frame: usize, scale: u16) -> Self {
        Self {
            frame: frame % CODING_FRAMES.len(),
            scale: scale.max(1),
        }
    }
}

impl Widget for CodingClawd {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_pixels(CODING_FRAMES[self.frame], area, buf, self.scale);
    }
}

pub struct SuitClawd {
    frame: usize,
    scale: u16,
}

impl SuitClawd {
    pub fn scaled(frame: usize, scale: u16) -> Self {
        Self {
            frame: frame % SUIT_FRAMES.len(),
            scale: scale.max(1),
        }
    }
}

impl Widget for SuitClawd {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_pixels(SUIT_FRAMES[self.frame], area, buf, self.scale);
    }
}

fn render_pixels(frame: &str, area: Rect, buf: &mut Buffer, scale: u16) {
    for (row, pixels) in frame.lines().enumerate() {
        for (col, pixel) in pixels.chars().enumerate() {
            let Some(color) = color_for(pixel) else {
                continue;
            };

            let y = area.y + row as u16 * scale;
            let x = area.x + col as u16 * scale;

            for scaled_y in y..y.saturating_add(scale) {
                for scaled_x in x..x.saturating_add(scale) {
                    if scaled_x >= area.right() || scaled_y >= area.bottom() {
                        continue;
                    }

                    buf[(scaled_x, scaled_y)]
                        .set_symbol(" ")
                        .set_style(Style::default().bg(color));
                }
            }
        }
    }
}

fn color_for(pixel: char) -> Option<Color> {
    match pixel {
        '#' => Some(BODY),
        '@' => Some(LAPTOP),
        '+' => Some(SCREEN),
        '*' => Some(CODE),
        'S' => Some(SUIT),
        '%' => Some(SHIRT),
        '!' => Some(TIE),
        _ => None,
    }
}
