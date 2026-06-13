use std::{
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::Rect};

use crate::{
    control::{ControlCommand, ControlScene},
    metrics::{Metrics, StatsSnapshot},
    stats_panel::StatsPanel,
};

mod clawd;
mod control;
mod metrics;
mod sprite_preview;
mod stats_panel;

const TICK_RATE: Duration = Duration::from_millis(140);
const STATS_RATE: Duration = Duration::from_secs(1);
const STATS_HEIGHT: u16 = stats_panel::HEIGHT;
const STATS_MIN_WIDTH: u16 = 62;
const STATIONARY_MAX_ART_SCALE: u16 = 2;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    if std::env::args().any(|arg| arg == "--preview-sprites") {
        sprite_preview::export_all("target/sprite-previews")?;
        println!("Sprite previews written to target/sprite-previews/");
        return Ok(());
    }

    let (control_tx, control_rx) = mpsc::channel();
    match control::start(control_tx) {
        Ok(addr) => eprintln!("Control API listening on http://{addr}"),
        Err(error) => eprintln!("Control API unavailable: {error}"),
    }

    ratatui::run(|terminal| app(terminal, control_rx))?;
    Ok(())
}

fn app(
    terminal: &mut DefaultTerminal,
    control_rx: Receiver<ControlCommand>,
) -> std::io::Result<()> {
    let mut app = App::new(control_rx);

    loop {
        terminal.draw(|frame| render(frame, &app))?;

        if event::poll(TICK_RATE)? && app.handle_event(event::read()?) {
            return Ok(());
        }

        app.tick();
    }
}

fn render(frame: &mut Frame, app: &App) {
    match app.mode {
        ViewMode::Dashboard => render_dashboard(frame, app),
        ViewMode::Coding => render_coding(frame, app),
        ViewMode::Suit => render_suit(frame, app),
        ViewMode::Roam => render_roam(frame, app),
    }
}

fn render_dashboard(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let scale = art_scale(
        area,
        clawd::WIDTH.saturating_mul(2),
        clawd::HEIGHT,
        STATS_HEIGHT + 1,
        STATIONARY_MAX_ART_SCALE,
    );
    let mascot_width = scaled(clawd::WIDTH, scale);
    let mascot_height = scaled(clawd::HEIGHT, scale);
    let content = centered_scene_area(area, mascot_width.saturating_mul(2), mascot_height);
    let track_width = content.width.max(mascot_width);
    let track_area = Rect {
        x: content.x,
        y: content.y,
        width: track_width.min(area.width),
        height: mascot_height.min(area.height),
    };
    let mascot_area = Rect {
        x: track_area.x + bounded_axis(app.dashboard_x, track_area.width, mascot_width),
        y: track_area.y,
        width: mascot_width.min(track_area.width),
        height: mascot_height.min(track_area.height),
    };
    let stats_y = content.y + mascot_height + 1;
    let stats_area = Rect {
        x: content.x,
        y: stats_y,
        width: content.width,
        height: STATS_HEIGHT.min(area.bottom().saturating_sub(stats_y)),
    };

    frame.render_widget(clawd::Clawd::scaled(app.frame, scale), mascot_area);
    frame.render_widget(StatsPanel::new(&app.stats), stats_area);
}

fn render_coding(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let scale = art_scale(
        area,
        clawd::CODING_WIDTH,
        clawd::CODING_HEIGHT,
        STATS_HEIGHT + 1,
        STATIONARY_MAX_ART_SCALE,
    );
    let mascot_width = scaled(clawd::CODING_WIDTH, scale);
    let mascot_height = scaled(clawd::CODING_HEIGHT, scale);
    let content = centered_scene_area(area, mascot_width, mascot_height);
    let mascot_area = Rect {
        x: content.x + content.width.saturating_sub(mascot_width) / 2,
        y: content.y,
        width: mascot_width.min(content.width),
        height: mascot_height.min(content.height),
    };
    let stats_y = content.y + mascot_height + 1;
    let stats_area = Rect {
        x: content.x,
        y: stats_y,
        width: content.width,
        height: STATS_HEIGHT.min(area.bottom().saturating_sub(stats_y)),
    };

    frame.render_widget(
        clawd::CodingClawd::scaled(app.coding_frame, scale),
        mascot_area,
    );
    frame.render_widget(StatsPanel::new(&app.stats), stats_area);
}

fn render_suit(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let scale = art_scale(
        area,
        clawd::SUIT_WIDTH,
        clawd::SUIT_HEIGHT,
        STATS_HEIGHT + 1,
        STATIONARY_MAX_ART_SCALE,
    );
    let mascot_width = scaled(clawd::SUIT_WIDTH, scale);
    let mascot_height = scaled(clawd::SUIT_HEIGHT, scale);
    let content = centered_scene_area(area, mascot_width, mascot_height);
    let mascot_area = Rect {
        x: content.x + content.width.saturating_sub(mascot_width) / 2,
        y: content.y,
        width: mascot_width.min(content.width),
        height: mascot_height.min(content.height),
    };
    let stats_y = content.y + mascot_height + 1;
    let stats_area = Rect {
        x: content.x,
        y: stats_y,
        width: content.width,
        height: STATS_HEIGHT.min(area.bottom().saturating_sub(stats_y)),
    };

    frame.render_widget(clawd::SuitClawd::scaled(app.suit_frame, scale), mascot_area);
    frame.render_widget(StatsPanel::new(&app.stats), stats_area);
}

fn render_roam(frame: &mut Frame, app: &App) {
    let mascot_area = app.roam_area(frame.area());
    let scale = art_scale(
        frame.area(),
        clawd::WIDTH,
        clawd::HEIGHT,
        0,
        STATIONARY_MAX_ART_SCALE,
    );
    frame.render_widget(clawd::Clawd::scaled(app.frame, scale), mascot_area);
}

fn centered_scene_area(area: Rect, mascot_width: u16, mascot_height: u16) -> Rect {
    let width = area.width.min(STATS_MIN_WIDTH.max(mascot_width));
    let height = area.height.min(mascot_height + 1 + STATS_HEIGHT);

    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn art_scale(area: Rect, width: u16, height: u16, reserved_height: u16, max_scale: u16) -> u16 {
    let available_height = area.height.saturating_sub(reserved_height);
    let width_scale = area.width / width.max(1);
    let height_scale = available_height / height.max(1);

    width_scale.min(height_scale).clamp(1, max_scale.max(1))
}

fn scaled(value: u16, scale: u16) -> u16 {
    value.saturating_mul(scale.max(1))
}

struct App {
    x: i16,
    y: i16,
    dx: i16,
    dy: i16,
    dashboard_x: i16,
    frame: usize,
    coding_frame: usize,
    suit_frame: usize,
    mode: ViewMode,
    last_stats_refresh: Instant,
    metrics: Metrics,
    stats: StatsSnapshot,
    control_rx: Receiver<ControlCommand>,
}

#[derive(Clone, Copy)]
enum ViewMode {
    Dashboard,
    Coding,
    Suit,
    Roam,
}

impl App {
    fn new(control_rx: Receiver<ControlCommand>) -> Self {
        let mut metrics = Metrics::new();
        let stats = metrics.refresh(STATS_RATE);

        Self {
            x: 0,
            y: 0,
            dx: 1,
            dy: 1,
            dashboard_x: 0,
            frame: 0,
            coding_frame: 0,
            suit_frame: 0,
            mode: ViewMode::Dashboard,
            last_stats_refresh: Instant::now(),
            metrics,
            stats,
            control_rx,
        }
    }

    fn tick(&mut self) {
        self.handle_control_commands();

        self.x += self.dx;
        self.y += self.dy;
        self.dashboard_x += 1;
        self.frame = (self.frame + 1) % clawd::frame_count();
        self.coding_frame = (self.coding_frame + 1) % clawd::coding_frame_count();
        self.suit_frame = (self.suit_frame + 1) % clawd::suit_frame_count();

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_stats_refresh);
        if elapsed >= STATS_RATE {
            self.stats = self.metrics.refresh(elapsed);
            self.last_stats_refresh = now;
        }
    }

    fn handle_control_commands(&mut self) {
        while let Ok(command) = self.control_rx.try_recv() {
            match command {
                ControlCommand::Reset => self.reset_scene(),
                ControlCommand::SetScene(scene) => self.set_scene(scene),
            }
        }
    }

    fn reset_scene(&mut self) {
        self.mode = ViewMode::Dashboard;
        self.dashboard_x = 0;
        self.x = 0;
        self.y = 0;
    }

    fn set_scene(&mut self, scene: ControlScene) {
        self.mode = match scene {
            ControlScene::Dashboard => ViewMode::Dashboard,
            ControlScene::Coding => ViewMode::Coding,
            ControlScene::Suit => ViewMode::Suit,
            ControlScene::Roam => ViewMode::Roam,
        };
    }

    fn handle_event(&mut self, event: Event) -> bool {
        let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        else {
            return false;
        };

        if matches!(code, KeyCode::Esc | KeyCode::Char('q'))
            || matches!(code, KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL))
        {
            return true;
        }

        if matches!(code, KeyCode::Tab | KeyCode::Char('r')) {
            self.mode = match self.mode {
                ViewMode::Dashboard => ViewMode::Coding,
                ViewMode::Coding => ViewMode::Suit,
                ViewMode::Suit => ViewMode::Roam,
                ViewMode::Roam => ViewMode::Dashboard,
            };
        }

        if matches!(code, KeyCode::Char('c')) {
            self.mode = match self.mode {
                ViewMode::Coding => ViewMode::Dashboard,
                _ => ViewMode::Coding,
            };
        }

        if matches!(code, KeyCode::Char('s')) {
            self.mode = match self.mode {
                ViewMode::Suit => ViewMode::Dashboard,
                _ => ViewMode::Suit,
            };
        }

        false
    }

    fn roam_area(&self, terminal: Rect) -> Rect {
        let scale = art_scale(
            terminal,
            clawd::WIDTH,
            clawd::HEIGHT,
            0,
            STATIONARY_MAX_ART_SCALE,
        );
        let width = scaled(clawd::WIDTH, scale);
        let height = scaled(clawd::HEIGHT, scale);
        let x = bounded_axis(self.x, terminal.width, width);
        let y = bounded_axis(self.y, terminal.height, height);

        Rect {
            x: terminal.x + x,
            y: terminal.y + y,
            width: width.min(terminal.width),
            height: height.min(terminal.height),
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
