use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

const CLAWD_WIDTH: u16 = 16;
const CLAWD_HEIGHT: u16 = 5;
const CLAWD_BODY: Color = Color::Rgb(216, 122, 88);
const CLAWD_PIXELS: [&str; CLAWD_HEIGHT as usize] = [
    "  ############  ",
    "  ## ###### ##  ",
    "################",
    "  ############  ",
    "   # #    # #   ",
];

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;

        if should_quit(event::read()?) {
            return Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let mascot_area = centered_area(frame.area(), CLAWD_WIDTH, CLAWD_HEIGHT);
    frame.render_widget(Clawd, mascot_area);
}

fn should_quit(event: Event) -> bool {
    let Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        ..
    }) = event
    else {
        return false;
    };

    matches!(code, KeyCode::Esc | KeyCode::Char('q'))
        || matches!(code, KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL))
}

fn centered_area(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);

    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

struct Clawd;

impl Widget for Clawd {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (row, pixels) in CLAWD_PIXELS.iter().enumerate() {
            for (col, pixel) in pixels.chars().enumerate() {
                if pixel == ' ' {
                    continue;
                }

                let y = area.y + row as u16;
                let x = area.x + col as u16;

                if x >= area.right() || y >= area.bottom() {
                    continue;
                }

                let style = match pixel {
                    '#' => Style::default().bg(CLAWD_BODY),
                    _ => Style::default(),
                };

                buf[(x, y)].set_symbol(" ").set_style(style);
            }
        }
    }
}
