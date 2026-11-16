//! Модуль управления репозиториями и источниками пакетов.
//!
//! Разделяет два фундаментальных подхода:
//! 1. WSP (System Packages): адрес системного репозитория зашит в бинарный файл при компиляции.
//! 2. WPM (User Packages): динамические источники через прямой синтаксис (`Server = URL`).

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::version::Version;

/// Базовый адрес системного репозитория WSP по умолчанию.
/// Может быть переопределен на этапе компиляции переменной окружения WSP_DEFAULT_REPO_URL.
pub const DEFAULT_WSP_REPO: &str = match option_env!("WSP_DEFAULT_REPO_URL") {
    Some(url) => url,
    None => "http://wunix.rusos.site/repo/system/x86_64",
};

/// Возвращает адрес системного репозитория WSP
pub fn get_wsp_system_repo() -> &'static str {
    DEFAULT_WSP_REPO
}

/// Секция репозитория WPM со списком серверов
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSection {
    pub name: String,
    pub servers: Vec<String>,
}

/// Разбирает конфигурационный файл источников WPM:
///
/// ```ini
/// [community]
/// Server = https://community.wunix.dev/repo/x86_64
/// ```
pub fn parse_repo_sources(content: &str) -> Result<Vec<RepoSection>, &'static str> {
    let mut repos: Vec<RepoSection> = Vec::new();
    let mut current_repo: Option<RepoSection> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Пропуск пустых строк и комментариев
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        // Новая секция репозитория: [имя]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(r) = current_repo.take() {
                repos.push(r);
            }
            let name = &trimmed[1..trimmed.len() - 1].trim();
            current_repo = Some(RepoSection {
                name: name.to_string(),
                servers: Vec::new(),
            });
            continue;
        }

        // Параметр Server = URL
        if let Some(ref mut r) = current_repo {
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let val = trimmed[eq_pos + 1..].trim();

                if key.eq_ignore_ascii_case("Server") {
                    r.servers.push(val.to_string());
                }
            }
        }
    }

    if let Some(r) = current_repo {
        repos.push(r);
    }

    Ok(repos)
}

/// Системная конфигурация WSP из /etc/wsp/wsp.conf.
/// Обратите внимание: адрес репозитория (repo_url) зашит в бинарник (DEFAULT_WSP_REPO)
/// и защищен от редактирования обычными пользователями. Загрузчик limine_conf также
/// не подлежит пользовательской настройке.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WspConfig {
    pub block_user_packages_on_critical: bool,
    pub auto_audit: bool,
    pub cache_dir: String,
    pub backup_kernel: bool,
}

impl Default for WspConfig {
    fn default() -> Self {
        Self {
            block_user_packages_on_critical: true,
            auto_audit: true,
            cache_dir: "/var/cache/wsp".to_string(),
            backup_kernel: true,
        }
    }
}

impl WspConfig {
    /// Разбирает файл /etc/wsp/wsp.conf
    pub fn parse(content: &str) -> Self {
        let mut cfg = WspConfig::default();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                continue;
            }

            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let val = trimmed[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'');

                match key {
                    "block_user_packages_on_critical" | "block_unsecure_apps" | "block_if_wsp_outdated" => {
                        cfg.block_user_packages_on_critical = val.eq_ignore_ascii_case("true") || val == "1";
                    }
                    "auto_audit" | "auto_check" => {
                        cfg.auto_audit = val.eq_ignore_ascii_case("true") || val == "1";
                    }
                    "cache_dir" => {
                        cfg.cache_dir = val.to_string();
                    }
                    "backup_kernel" => {
                        cfg.backup_kernel = val.eq_ignore_ascii_case("true") || val == "1";
                    }
                    // repo_url и limine_conf намеренно игнорируются для безопасности дистрибутива!
                    _ => {}
                }
            }
        }

        cfg
    }
}

/// Описание пакета в репозитории (индекс packages.json).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoPackage {
    pub name: String,
    pub version: Version,
    pub arch: String,
    pub tier: String,
    pub package_type: String,
    pub description: String,
    pub filename: String,
    pub size: u64,
    pub sha256: String,
}

/// Разбирает индекс пакетов репозитория packages.json
pub fn parse_repo_index(json_str: &str) -> Vec<RepoPackage> {
    let mut packages = Vec::new();
    let mut cur_name = String::new();
    let mut cur_version = Version::new(1, 0, 0);
    let mut cur_arch = "x86_64".to_string();
    let mut cur_tier = "base".to_string();
    let mut cur_type = "system".to_string();
    let mut cur_desc = String::new();
    let mut cur_filename = String::new();
    let mut cur_size = 0u64;
    let mut cur_sha256 = String::new();

    for line in json_str.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if trimmed.starts_with("\"name\":") {
            if let Some(v) = extract_json_str(trimmed) {
                cur_name = v;
            }
        } else if trimmed.starts_with("\"version\":") {
            if let Some(v) = extract_json_str(trimmed) {
                if let Ok(ver) = Version::parse(&v) {
                    cur_version = ver;
                }
            }
        } else if trimmed.starts_with("\"arch\":") {
            if let Some(v) = extract_json_str(trimmed) {
                cur_arch = v;
            }
        } else if trimmed.starts_with("\"tier\":") {
            if let Some(v) = extract_json_str(trimmed) {
                cur_tier = v;
            }
        } else if trimmed.starts_with("\"type\":") {
            if let Some(v) = extract_json_str(trimmed) {
                cur_type = v;
            }
        } else if trimmed.starts_with("\"description\":") {
            if let Some(v) = extract_json_str(trimmed) {
                cur_desc = v;
            }
        } else if trimmed.starts_with("\"filename\":") {
            if let Some(v) = extract_json_str(trimmed) {
                cur_filename = v;
            }
        } else if trimmed.starts_with("\"size\":") {
            if let Some(v) = extract_json_num(trimmed) {
                cur_size = v;
            }
        } else if trimmed.starts_with("\"sha256\":") {
            if let Some(v) = extract_json_str(trimmed) {
                cur_sha256 = v;
            }
        } else if trimmed == "}" || trimmed == "}," {
            if !cur_name.is_empty() {
                packages.push(RepoPackage {
                    name: cur_name.clone(),
                    version: cur_version,
                    arch: cur_arch.clone(),
                    tier: cur_tier.clone(),
                    package_type: cur_type.clone(),
                    description: cur_desc.clone(),
                    filename: cur_filename.clone(),
                    size: cur_size,
                    sha256: cur_sha256.clone(),
                });
                cur_name.clear();
                cur_version = Version::new(1, 0, 0);
                cur_arch = "x86_64".to_string();
                cur_tier = "base".to_string();
                cur_type = "system".to_string();
                cur_desc.clear();
                cur_filename.clear();
                cur_size = 0;
                cur_sha256.clear();
            }
        }
    }

    packages
}

fn extract_json_str(line: &str) -> Option<String> {
    let mut quotes = Vec::new();
    for (i, b) in line.bytes().enumerate() {
        if b == b'"' {
            quotes.push(i);
        }
    }
    if quotes.len() >= 4 {
        let start = quotes[2] + 1;
        let end = quotes[3];
        if end >= start {
            return Some(line[start..end].to_string());
        }
    }
    None
}

fn extract_json_num(line: &str) -> Option<u64> {
    if let Some(colon) = line.find(':') {
        let val_part = line[colon + 1..].trim().trim_end_matches(',').trim();
        return val_part.parse::<u64>().ok();
    }
    None
}
