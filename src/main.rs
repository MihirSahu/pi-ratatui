use std::time::Duration;

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
const TICK_RATE: Duration = Duration::from_millis(140);
const CLAWD_FRAMES: [[&str; CLAWD_HEIGHT as usize]; 4] = [
    [
        "  ############  ",
        "  ## ###### ##  ",
        "################",
        "  ############  ",
        "   # #    # #   ",
    ],
    [
        "  ############  ",
        "  ## ###### ##  ",
        "################",
        "  ############  ",
        "  #  #    #  #  ",
    ],
    [
        "  ############  ",
        "  ## ###### ##  ",
        "################",
        "  ############  ",
        "   # #    # #   ",
    ],
    [
        "  ############  ",
        "  ## ###### ##  ",
        "################",
        "  ############  ",
        "   #  #  #  #   ",
    ],
];

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::default();

    loop {
        terminal.draw(|frame| render(frame, &app))?;

        if event::poll(TICK_RATE)? {
            if should_quit(event::read()?) {
                return Ok(());
            }
        }

        app.tick();
    }
}

fn render(frame: &mut Frame, app: &App) {
    let mascot_area = app.area(frame.area());
    frame.render_widget(Clawd::new(app.frame), mascot_area);
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

struct App {
    x: i16,
    y: i16,
    dx: i16,
    dy: i16,
    frame: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            dx: 1,
            dy: 1,
            frame: 0,
        }
    }
}

impl App {
    fn tick(&mut self) {
        self.x += self.dx;
        self.y += self.dy;
        self.frame = (self.frame + 1) % CLAWD_FRAMES.len();
    }

    fn area(&self, terminal: Rect) -> Rect {
        let x = bounded_axis(self.x, terminal.width, CLAWD_WIDTH);
        let y = bounded_axis(self.y, terminal.height, CLAWD_HEIGHT);

        Rect {
            x: terminal.x + x,
            y: terminal.y + y,
            width: CLAWD_WIDTH.min(terminal.width),
            height: CLAWD_HEIGHT.min(terminal.height),
        }
    }
}

fn bounded_axis(value: i16, available: u16, size: u16) -> u16 {
    let limit = available.saturating_sub(size) as i16;

    if limit <= 0 {
        return 0;
    }

    let period = limit * 2;
    let value = value.rem_euclid(period);
    let value = if value <= limit {
        value
    } else {
        period - value
    };

    value as u16
}

struct Clawd {
    frame: usize,
}

impl Clawd {
    fn new(frame: usize) -> Self {
        Self {
            frame: frame % CLAWD_FRAMES.len(),
        }
    }
}

impl Widget for Clawd {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (row, pixels) in CLAWD_FRAMES[self.frame].iter().enumerate() {
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
