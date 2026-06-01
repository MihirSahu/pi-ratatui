use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use sysinfo::{Components, Disks, Networks};

mod clawd;
mod sprite_preview;

const TICK_RATE: Duration = Duration::from_millis(140);
const STATS_RATE: Duration = Duration::from_secs(1);
const STATS_HEIGHT: u16 = 6;
const STATS_MIN_WIDTH: u16 = 44;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    if std::env::args().any(|arg| arg == "--preview-sprites") {
        sprite_preview::export_all("target/sprite-previews")?;
        println!("Sprite previews written to target/sprite-previews/");
        return Ok(());
    }

    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::default();

    loop {
        terminal.draw(|frame| render(frame, &app))?;

        if event::poll(TICK_RATE)? {
            if app.handle_event(event::read()?) {
                return Ok(());
            }
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
    let content = centered_scene_area(area, clawd::WIDTH, clawd::HEIGHT);
    let track_width = content.width.max(clawd::WIDTH);
    let track_area = Rect {
        x: content.x,
        y: content.y,
        width: track_width.min(area.width),
        height: clawd::HEIGHT.min(area.height),
    };
    let mascot_area = Rect {
        x: track_area.x + bounded_axis(app.dashboard_x, track_area.width, clawd::WIDTH),
        y: track_area.y,
        width: clawd::WIDTH.min(track_area.width),
        height: clawd::HEIGHT.min(track_area.height),
    };
    let stats_y = content.y + clawd::HEIGHT + 1;
    let stats_area = Rect {
        x: content.x,
        y: stats_y,
        width: content.width,
        height: STATS_HEIGHT.min(area.bottom().saturating_sub(stats_y)),
    };

    frame.render_widget(clawd::Clawd::new(app.frame), mascot_area);
    frame.render_widget(StatsPanel::new(&app.stats), stats_area);
}

fn render_coding(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let content = centered_scene_area(area, clawd::CODING_WIDTH, clawd::CODING_HEIGHT);
    let mascot_area = Rect {
        x: content.x + content.width.saturating_sub(clawd::CODING_WIDTH) / 2,
        y: content.y,
        width: clawd::CODING_WIDTH.min(content.width),
        height: clawd::CODING_HEIGHT.min(content.height),
    };
    let stats_y = content.y + clawd::CODING_HEIGHT + 1;
    let stats_area = Rect {
        x: content.x,
        y: stats_y,
        width: content.width,
        height: STATS_HEIGHT.min(area.bottom().saturating_sub(stats_y)),
    };

    frame.render_widget(clawd::CodingClawd::new(app.coding_frame), mascot_area);
    frame.render_widget(StatsPanel::new(&app.stats), stats_area);
}

fn render_suit(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let content = centered_scene_area(area, clawd::SUIT_WIDTH, clawd::SUIT_HEIGHT);
    let mascot_area = Rect {
        x: content.x + content.width.saturating_sub(clawd::SUIT_WIDTH) / 2,
        y: content.y,
        width: clawd::SUIT_WIDTH.min(content.width),
        height: clawd::SUIT_HEIGHT.min(content.height),
    };
    let stats_y = content.y + clawd::SUIT_HEIGHT + 1;
    let stats_area = Rect {
        x: content.x,
        y: stats_y,
        width: content.width,
        height: STATS_HEIGHT.min(area.bottom().saturating_sub(stats_y)),
    };

    frame.render_widget(clawd::SuitClawd::new(app.suit_frame), mascot_area);
    frame.render_widget(StatsPanel::new(&app.stats), stats_area);
}

fn render_roam(frame: &mut Frame, app: &App) {
    let mascot_area = app.roam_area(frame.area());
    frame.render_widget(clawd::Clawd::new(app.frame), mascot_area);
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
}

#[derive(Clone, Copy)]
enum ViewMode {
    Dashboard,
    Coding,
    Suit,
    Roam,
}

impl Default for App {
    fn default() -> Self {
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
        }
    }
}

impl App {
    fn tick(&mut self) {
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
        let x = bounded_axis(self.x, terminal.width, clawd::WIDTH);
        let y = bounded_axis(self.y, terminal.height, clawd::HEIGHT);

        Rect {
            x: terminal.x + x,
            y: terminal.y + y,
            width: clawd::WIDTH.min(terminal.width),
            height: clawd::HEIGHT.min(terminal.height),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct StatsSnapshot {
    temperature_c: Option<f32>,
    network_in_per_sec: f64,
    network_out_per_sec: f64,
    storage: Option<StorageSnapshot>,
}

#[derive(Clone, Copy)]
struct StorageSnapshot {
    available: u64,
    total: u64,
}

struct Metrics {
    networks: Networks,
    disks: Disks,
    components: Components,
}

impl Metrics {
    fn new() -> Self {
        let mut networks = Networks::new_with_refreshed_list();
        let mut disks = Disks::new_with_refreshed_list();
        let mut components = Components::new_with_refreshed_list();

        networks.refresh(true);
        disks.refresh(true);
        components.refresh(true);

        Self {
            networks,
            disks,
            components,
        }
    }

    fn refresh(&mut self, elapsed: Duration) -> StatsSnapshot {
        self.networks.refresh(true);
        self.disks.refresh(true);
        self.components.refresh(true);

        let seconds = elapsed.as_secs_f64().max(0.001);
        let (received, transmitted) = network_bytes(&self.networks);

        StatsSnapshot {
            temperature_c: component_temperature(&self.components).or_else(pi_temperature),
            network_in_per_sec: received as f64 / seconds,
            network_out_per_sec: transmitted as f64 / seconds,
            storage: storage_snapshot(&self.disks),
        }
    }
}

fn network_bytes(networks: &Networks) -> (u64, u64) {
    let mut received = 0;
    let mut transmitted = 0;
    let mut found_non_loopback = false;

    for (name, data) in networks.iter() {
        if name.starts_with("lo") {
            continue;
        }

        found_non_loopback = true;
        received += data.received();
        transmitted += data.transmitted();
    }

    if found_non_loopback {
        return (received, transmitted);
    }

    networks
        .iter()
        .fold((0, 0), |(received, transmitted), (_, data)| {
            (received + data.received(), transmitted + data.transmitted())
        })
}

fn component_temperature(components: &Components) -> Option<f32> {
    components
        .list()
        .iter()
        .filter_map(|component| component.temperature())
        .filter(|temperature| temperature.is_finite())
        .max_by(|a, b| a.total_cmp(b))
}

fn pi_temperature() -> Option<f32> {
    let raw = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").ok()?;
    let milli_celsius = raw.trim().parse::<f32>().ok()?;

    Some(milli_celsius / 1000.0)
}

fn storage_snapshot(disks: &Disks) -> Option<StorageSnapshot> {
    let current_dir = std::env::current_dir().ok();
    let disk = disks
        .list()
        .iter()
        .max_by_key(|disk| {
            current_dir
                .as_ref()
                .filter(|path| path.starts_with(disk.mount_point()))
                .map(|_| disk.mount_point().components().count())
                .unwrap_or_else(|| usize::from(disk.mount_point() == Path::new("/")))
        })
        .or_else(|| disks.list().first())?;

    Some(StorageSnapshot {
        available: disk.available_space(),
        total: disk.total_space(),
    })
}

struct StatsPanel<'a> {
    stats: &'a StatsSnapshot,
}

impl<'a> StatsPanel<'a> {
    fn new(stats: &'a StatsSnapshot) -> Self {
        Self { stats }
    }
}

impl Widget for StatsPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label_style = Style::default().fg(Color::Rgb(216, 122, 88));
        let value_style = Style::default().fg(Color::White);
        let muted_style = Style::default().fg(Color::DarkGray);
        let panel_style = Style::default().fg(Color::Gray);
        let storage = self
            .stats
            .storage
            .map(format_storage)
            .unwrap_or_else(|| "unavailable".to_owned());

        let lines = vec![
            Line::from(Span::styled("RASPBERRY PI", label_style)),
            Line::from(Span::styled("------------", muted_style)),
            Line::from(vec![
                Span::styled("TEMP  ", label_style),
                Span::styled(format_temperature(self.stats.temperature_c), value_style),
            ]),
            Line::from(vec![
                Span::styled("NET   ", label_style),
                Span::styled(
                    format!(
                        "IN {}  OUT {}",
                        format_rate(self.stats.network_in_per_sec),
                        format_rate(self.stats.network_out_per_sec)
                    ),
                    value_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("DISK  ", label_style),
                Span::styled(storage, value_style),
            ]),
        ];

        Paragraph::new(lines).style(panel_style).render(area, buf);
    }
}

fn format_temperature(temperature: Option<f32>) -> String {
    temperature
        .map(|temperature| format!("{temperature:.1} C"))
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn format_storage(storage: StorageSnapshot) -> String {
    format!(
        "{} free / {}",
        format_bytes(storage.available as f64),
        format_bytes(storage.total as f64)
    )
}

fn format_rate(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

fn format_bytes(bytes: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes.max(0.0);
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{value:.0} {}", UNITS[unit])
    } else if value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
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
