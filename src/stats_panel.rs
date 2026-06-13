use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::metrics::{
    HealthLevel, HealthSnapshot, MemorySnapshot, PiPowerStatus, RuntimeSnapshot, StatsSnapshot,
    StorageSnapshot, disk_level, load_level, memory_used_percent, percent_level, pi_power_level,
    temperature_level,
};

pub(crate) const HEIGHT: u16 = 11;

pub(crate) struct StatsPanel<'a> {
    stats: &'a StatsSnapshot,
}

impl<'a> StatsPanel<'a> {
    pub(crate) fn new(stats: &'a StatsSnapshot) -> Self {
        Self { stats }
    }
}

impl Widget for StatsPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label_style = Style::default().fg(Color::Rgb(216, 122, 88));
        let muted_style = Style::default().fg(Color::DarkGray);
        let panel_style = Style::default().fg(Color::Gray);
        let lines = vec![
            Line::from(Span::styled(
                self.stats.host_name.to_uppercase(),
                label_style,
            )),
            Line::from(Span::styled("------------", muted_style)),
            two_metric_line(
                "TEMP",
                format_temperature(self.stats.temperature_c),
                value_style(temperature_level(self.stats.temperature_c)),
                "CPU",
                format!(
                    "{:.0}% ({} cores)",
                    self.stats.cpu_usage, self.stats.cpu_count
                ),
                value_style(percent_level(self.stats.cpu_usage, 70.0, 85.0)),
            ),
            metric_line(
                "RAM",
                self.stats
                    .memory
                    .map(format_memory)
                    .unwrap_or_else(|| "unavailable".to_owned()),
                self.stats
                    .memory
                    .map(|memory| {
                        value_style(percent_level(memory_used_percent(memory), 75.0, 90.0))
                    })
                    .unwrap_or_else(|| value_style(HealthLevel::Ok)),
            ),
            metric_line(
                "LOAD",
                format!(
                    "1m {:.2}  5m {:.2}  15m {:.2}",
                    self.stats.load_average.one,
                    self.stats.load_average.five,
                    self.stats.load_average.fifteen
                ),
                value_style(load_level(self.stats)),
            ),
            metric_line(
                "NET",
                format!(
                    "in {}  out {}",
                    format_rate(self.stats.network_in_per_sec),
                    format_rate(self.stats.network_out_per_sec)
                ),
                value_style(HealthLevel::Ok),
            ),
            metric_line(
                "DISK",
                self.stats
                    .storage
                    .map(format_storage)
                    .unwrap_or_else(|| "unavailable".to_owned()),
                self.stats
                    .storage
                    .map(|storage| value_style(disk_level(storage)))
                    .unwrap_or_else(|| value_style(HealthLevel::Ok)),
            ),
            metric_line(
                "IP",
                self.stats
                    .local_ip
                    .map(|ip| ip.to_string())
                    .unwrap_or_else(|| "unavailable".to_owned()),
                value_style(HealthLevel::Ok),
            ),
            two_metric_line(
                "UP",
                format_duration(self.stats.uptime),
                value_style(HealthLevel::Ok),
                "APP",
                runtime_label(self.stats.runtime).to_owned(),
                value_style(HealthLevel::Ok),
            ),
            two_metric_line(
                "POWER",
                pi_power_label(self.stats.pi_power),
                value_style(pi_power_level(self.stats.pi_power)),
                "HEALTH",
                health_label(self.stats.health),
                value_style(self.stats.health.level),
            ),
            metric_line(
                "CLAWD",
                clawd_reaction(self.stats).to_owned(),
                value_style(self.stats.health.level),
            ),
        ];
        debug_assert_eq!(lines.len(), HEIGHT as usize);

        Paragraph::new(lines).style(panel_style).render(area, buf);
    }
}

fn metric_line(label: &'static str, value: String, style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label:<7}"),
            Style::default().fg(Color::Rgb(216, 122, 88)),
        ),
        Span::styled(value, style),
    ])
}

fn two_metric_line(
    first_label: &'static str,
    first_value: String,
    first_style: Style,
    second_label: &'static str,
    second_value: String,
    second_style: Style,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{first_label:<7}"),
            Style::default().fg(Color::Rgb(216, 122, 88)),
        ),
        Span::styled(first_value, first_style),
        Span::raw("  "),
        Span::styled(
            format!("{second_label:<7}"),
            Style::default().fg(Color::Rgb(216, 122, 88)),
        ),
        Span::styled(second_value, second_style),
    ])
}

fn value_style(level: HealthLevel) -> Style {
    match level {
        HealthLevel::Ok => Style::default().fg(Color::White),
        HealthLevel::Warn => Style::default().fg(Color::Yellow),
        HealthLevel::Alert => Style::default().fg(Color::Red),
    }
}

fn format_temperature(temperature: Option<f32>) -> String {
    temperature
        .map(|temperature| format!("{temperature:.1} C"))
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn format_memory(memory: MemorySnapshot) -> String {
    format!(
        "{} free / {} total",
        format_bytes(memory.available as f64),
        format_bytes(memory.total as f64)
    )
}

fn format_storage(storage: StorageSnapshot) -> String {
    format!(
        "{} free / {} total",
        format_bytes(storage.available as f64),
        format_bytes(storage.total as f64)
    )
}

fn format_rate(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

fn format_duration(duration: Duration) -> String {
    let total_minutes = duration.as_secs() / 60;
    let days = total_minutes / (24 * 60);
    let hours = (total_minutes / 60) % 24;
    let minutes = total_minutes % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn format_bytes(bytes: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes.max(0.0);
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 || value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn runtime_label(runtime: RuntimeSnapshot) -> &'static str {
    if runtime.launched_by_systemd {
        "systemd"
    } else {
        "manual"
    }
}

fn health_label(health: HealthSnapshot) -> String {
    match health.level {
        HealthLevel::Ok => "ok".to_owned(),
        HealthLevel::Warn => format!("warn: {}", health.reason),
        HealthLevel::Alert => format!("alert: {}", health.reason),
    }
}

fn clawd_reaction(stats: &StatsSnapshot) -> &'static str {
    match stats.health.level {
        HealthLevel::Ok => "patrol ok",
        HealthLevel::Warn => "watching",
        HealthLevel::Alert => "on alert",
    }
}

fn pi_power_label(status: PiPowerStatus) -> String {
    match status {
        PiPowerStatus::Unknown => "n/a".to_owned(),
        PiPowerStatus::Ok => "ok".to_owned(),
        PiPowerStatus::Flags(flags) => {
            let mut labels = Vec::new();

            if flags & 0x01 != 0 {
                labels.push("undervoltage");
            }
            if flags & 0x02 != 0 {
                labels.push("freq capped");
            }
            if flags & 0x04 != 0 {
                labels.push("throttled");
            }
            if flags & 0x08 != 0 {
                labels.push("soft temp");
            }
            if labels.is_empty() && flags & 0x0f0000 != 0 {
                labels.push("had events");
            }

            if labels.is_empty() {
                format!("flags 0x{flags:x}")
            } else {
                labels.join(", ")
            }
        }
    }
}
