//! Модуль конфигурации фонового демона проверки обновлений WSP (`wspd`).
//!
//! Отвечает за разбор `/etc/wsp/daemon.conf` и параметров уведомлений.

extern crate alloc;

/// Режим уведомления пользователя
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotifyMode {
    #[default]
    Terminal,
    Log,
    Both,
}

impl NotifyMode {
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        match s.trim().to_ascii_lowercase().as_str() {
            "terminal" => Ok(NotifyMode::Terminal),
            "log" => Ok(NotifyMode::Log),
            "both" => Ok(NotifyMode::Both),
            _ => Err("Неизвестный notify_mode (ожидалось: 'terminal', 'log' или 'both')"),
        }
    }
}

/// Конфигурация фонового демона wspd
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonConfig {
    pub check_interval_seconds: u64,
    pub check_on_login: bool,
    pub critical_only: bool,
    pub notify_mode: NotifyMode,
    pub preload_packages: bool,
    pub timeout_seconds: u32,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 86400, // 24 часа
            check_on_login: true,
            critical_only: false,
            notify_mode: NotifyMode::Terminal,
            preload_packages: false,
            timeout_seconds: 10,
        }
    }
}

impl DaemonConfig {
    /// Разбирает конфигурационный файл daemon.conf
    pub fn parse(content: &str) -> Result<Self, &'static str> {
        let mut config = DaemonConfig::default();
        let mut section = "";

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                section = &trimmed[1..trimmed.len() - 1].trim();
                continue;
            }

            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let raw_val = trimmed[eq_pos + 1..].trim();
                // Удаляем окружающие кавычки при наличии
                let val = if (raw_val.starts_with('"') && raw_val.ends_with('"'))
                    || (raw_val.starts_with('\'') && raw_val.ends_with('\''))
                {
                    &raw_val[1..raw_val.len() - 1]
                } else {
                    raw_val
                };

                match section {
                    "daemon" => match key {
                        "check_interval" => {
                            config.check_interval_seconds = parse_duration_to_seconds(val)?;
                        }
                        "check_on_login" => {
                            config.check_on_login = parse_bool(val)?;
                        }
                        "critical_only" => {
                            config.critical_only = parse_bool(val)?;
                        }
                        "notify_mode" => {
                            config.notify_mode = NotifyMode::parse(val)?;
                        }
                        "preload_packages" => {
                            config.preload_packages = parse_bool(val)?;
                        }
                        _ => {}
                    },
                    "network" => match key {
                        "timeout_seconds" => {
                            config.timeout_seconds = val.parse::<u32>().map_err(|_| "Некорректный timeout_seconds")?;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        Ok(config)
    }
}

/// Разбирает человекопонятные интервалы: "24h", "30m", "60s", "1d" или просто "86400"
pub fn parse_duration_to_seconds(s: &str) -> Result<u64, &'static str> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Пустая строка интервала времени");
    }

    let last_char = s.chars().last().unwrap();
    if last_char.is_ascii_digit() {
        return s.parse::<u64>().map_err(|_| "Некорректное число секунд");
    }

    let num_part = &s[..s.len() - 1].trim();
    let num = num_part.parse::<u64>().map_err(|_| "Некорректное число в интервале времени")?;

    match last_char {
        's' | 'S' => Ok(num),
        'm' | 'M' => Ok(num * 60),
        'h' | 'H' => Ok(num * 3600),
        'd' | 'D' => Ok(num * 86400),
        _ => Err("Неизвестный суффикс времени (ожидалось: s, m, h, d)"),
    }
}

fn parse_bool(s: &str) -> Result<bool, &'static str> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        _ => Err("Некорректное булево значение"),
    }
}
