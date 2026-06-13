use std::{
    fs,
    net::{IpAddr, SocketAddr, UdpSocket},
    path::Path,
    process::Command,
    time::Duration,
};

use sysinfo::{Components, Disks, Networks, System};

#[derive(Clone)]
pub(crate) struct StatsSnapshot {
    pub(crate) host_name: String,
    pub(crate) temperature_c: Option<f32>,
    pub(crate) cpu_usage: f32,
    pub(crate) cpu_count: usize,
    pub(crate) memory: Option<MemorySnapshot>,
    pub(crate) load_average: LoadSnapshot,
    pub(crate) uptime: Duration,
    pub(crate) network_in_per_sec: f64,
    pub(crate) network_out_per_sec: f64,
    pub(crate) local_ip: Option<IpAddr>,
    pub(crate) storage: Option<StorageSnapshot>,
    pub(crate) runtime: RuntimeSnapshot,
    pub(crate) pi_power: PiPowerStatus,
    pub(crate) health: HealthSnapshot,
}

#[derive(Clone, Copy)]
pub(crate) struct StorageSnapshot {
    pub(crate) available: u64,
    pub(crate) total: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct MemorySnapshot {
    pub(crate) available: u64,
    pub(crate) used: u64,
    pub(crate) total: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct LoadSnapshot {
    pub(crate) one: f64,
    pub(crate) five: f64,
    pub(crate) fifteen: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct RuntimeSnapshot {
    pub(crate) launched_by_systemd: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum PiPowerStatus {
    Unknown,
    Ok,
    Flags(u32),
}

#[derive(Clone, Copy)]
pub(crate) struct HealthSnapshot {
    pub(crate) level: HealthLevel,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum HealthLevel {
    Ok,
    Warn,
    Alert,
}

pub(crate) struct Metrics {
    system: System,
    networks: Networks,
    disks: Disks,
    components: Components,
    host_name: String,
    local_ip: Option<IpAddr>,
    runtime: RuntimeSnapshot,
    pi_power: PiPowerStatus,
    pi_power_supported: bool,
}

impl Metrics {
    pub(crate) fn new() -> Self {
        let mut system = System::new_all();
        let mut networks = Networks::new_with_refreshed_list();
        let mut disks = Disks::new_with_refreshed_list();
        let mut components = Components::new_with_refreshed_list();

        system.refresh_cpu_all();
        system.refresh_memory();
        networks.refresh(true);
        disks.refresh(true);
        components.refresh(true);

        let pi_power = pi_power_status();

        Self {
            system,
            networks,
            disks,
            components,
            host_name: System::host_name().unwrap_or_else(|| "THIS HOST".to_owned()),
            local_ip: local_ip_address(),
            runtime: runtime_snapshot(),
            pi_power,
            pi_power_supported: !matches!(pi_power, PiPowerStatus::Unknown),
        }
    }

    pub(crate) fn refresh(&mut self, elapsed: Duration) -> StatsSnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.networks.refresh(true);
        self.disks.refresh(true);
        self.components.refresh(true);

        let seconds = elapsed.as_secs_f64().max(0.001);
        let (received, transmitted) = network_bytes(&self.networks);
        let load = System::load_average();

        self.local_ip = local_ip_address().or(self.local_ip);

        let memory = (self.system.total_memory() > 0).then(|| MemorySnapshot {
            available: self.system.available_memory(),
            used: self.system.used_memory(),
            total: self.system.total_memory(),
        });

        let storage = storage_snapshot(&self.disks);
        let temperature_c = component_temperature(&self.components).or_else(pi_temperature);
        if self.pi_power_supported {
            self.pi_power = pi_power_status();
        }
        let cpu_usage = self.system.global_cpu_usage();

        let mut snapshot = StatsSnapshot {
            host_name: self.host_name.clone(),
            temperature_c,
            cpu_usage,
            cpu_count: self.system.cpus().len(),
            memory,
            load_average: LoadSnapshot {
                one: load.one,
                five: load.five,
                fifteen: load.fifteen,
            },
            uptime: Duration::from_secs(System::uptime()),
            network_in_per_sec: received as f64 / seconds,
            network_out_per_sec: transmitted as f64 / seconds,
            local_ip: self.local_ip,
            storage,
            runtime: self.runtime,
            pi_power: self.pi_power,
            health: HealthSnapshot {
                level: HealthLevel::Ok,
                reason: "all clear",
            },
        };

        snapshot.health = health_snapshot(&snapshot);
        snapshot
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

fn local_ip_address() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect(SocketAddr::from(([8, 8, 8, 8], 80))).ok()?;

    let ip = socket.local_addr().ok()?.ip();

    (!ip.is_loopback()).then_some(ip)
}

fn runtime_snapshot() -> RuntimeSnapshot {
    RuntimeSnapshot {
        launched_by_systemd: std::env::var_os("INVOCATION_ID").is_some()
            || std::env::var_os("JOURNAL_STREAM").is_some(),
    }
}

fn pi_power_status() -> PiPowerStatus {
    let raw = read_pi_throttled_file()
        .or_else(vcgencmd_throttled)
        .and_then(|raw| parse_throttled_flags(&raw));

    match raw {
        Some(0) => PiPowerStatus::Ok,
        Some(flags) => PiPowerStatus::Flags(flags),
        None => PiPowerStatus::Unknown,
    }
}

fn read_pi_throttled_file() -> Option<String> {
    const PATHS: [&str; 2] = [
        "/sys/devices/platform/soc/soc:firmware/get_throttled",
        "/sys/devices/platform/raspberrypi-firmware/get_throttled",
    ];

    PATHS.iter().find_map(|path| fs::read_to_string(path).ok())
}

fn vcgencmd_throttled() -> Option<String> {
    let output = Command::new("vcgencmd")
        .arg("get_throttled")
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_throttled_flags(raw: &str) -> Option<u32> {
    let value = raw
        .trim()
        .split(['=', ' ', '\n', '\t'])
        .rev()
        .find(|part| !part.is_empty())?;

    value
        .strip_prefix("0x")
        .map(|hex| u32::from_str_radix(hex, 16))
        .unwrap_or_else(|| value.parse())
        .ok()
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

fn health_snapshot(stats: &StatsSnapshot) -> HealthSnapshot {
    let mut health = HealthSnapshot {
        level: HealthLevel::Ok,
        reason: "all clear",
    };

    raise_health(
        &mut health,
        temperature_level(stats.temperature_c),
        "heat",
        "hot",
    );
    raise_health(
        &mut health,
        percent_level(stats.cpu_usage, 70.0, 85.0),
        "cpu busy",
        "cpu pegged",
    );

    if let Some(memory) = stats.memory {
        raise_health(
            &mut health,
            percent_level(memory_used_percent(memory), 75.0, 90.0),
            "ram busy",
            "ram tight",
        );
    }

    if let Some(storage) = stats.storage {
        raise_health(&mut health, disk_level(storage), "disk low", "disk tight");
    }

    match stats.pi_power {
        PiPowerStatus::Flags(flags) if flags & 0x0f != 0 => {
            raise_health(&mut health, HealthLevel::Alert, "pi power", "pi power")
        }
        PiPowerStatus::Flags(_) => {
            raise_health(&mut health, HealthLevel::Warn, "pi power", "pi power")
        }
        PiPowerStatus::Unknown | PiPowerStatus::Ok => {}
    }

    health
}

fn raise_health(
    health: &mut HealthSnapshot,
    level: HealthLevel,
    warn_reason: &'static str,
    alert_reason: &'static str,
) {
    let reason = match level {
        HealthLevel::Ok => return,
        HealthLevel::Warn => warn_reason,
        HealthLevel::Alert => alert_reason,
    };

    if level > health.level {
        health.level = level;
        health.reason = reason;
    }
}

pub(crate) fn temperature_level(temperature: Option<f32>) -> HealthLevel {
    match temperature {
        Some(temperature) if temperature >= 75.0 => HealthLevel::Alert,
        Some(temperature) if temperature >= 65.0 => HealthLevel::Warn,
        _ => HealthLevel::Ok,
    }
}

pub(crate) fn percent_level(value: f32, warn: f32, alert: f32) -> HealthLevel {
    if value >= alert {
        HealthLevel::Alert
    } else if value >= warn {
        HealthLevel::Warn
    } else {
        HealthLevel::Ok
    }
}

pub(crate) fn disk_level(storage: StorageSnapshot) -> HealthLevel {
    if storage.total == 0 {
        return HealthLevel::Ok;
    }

    let free_ratio = storage.available as f64 / storage.total as f64;

    if free_ratio <= 0.10 {
        HealthLevel::Alert
    } else if free_ratio <= 0.20 {
        HealthLevel::Warn
    } else {
        HealthLevel::Ok
    }
}

pub(crate) fn memory_used_percent(memory: MemorySnapshot) -> f32 {
    if memory.total == 0 {
        0.0
    } else {
        memory.used as f32 / memory.total as f32 * 100.0
    }
}

pub(crate) fn load_level(stats: &StatsSnapshot) -> HealthLevel {
    if stats.cpu_count == 0 {
        return HealthLevel::Ok;
    }

    let load_per_cpu = stats.load_average.one / stats.cpu_count as f64;

    if load_per_cpu >= 1.5 {
        HealthLevel::Alert
    } else if load_per_cpu >= 1.0 {
        HealthLevel::Warn
    } else {
        HealthLevel::Ok
    }
}

pub(crate) fn pi_power_level(status: PiPowerStatus) -> HealthLevel {
    match status {
        PiPowerStatus::Unknown | PiPowerStatus::Ok => HealthLevel::Ok,
        PiPowerStatus::Flags(flags) if flags & 0x0f != 0 => HealthLevel::Alert,
        PiPowerStatus::Flags(_) => HealthLevel::Warn,
    }
}
