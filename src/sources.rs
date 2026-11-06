//! Модуль управления репозиториями и источниками пакетов.
//!
//! Разделяет два фундаментальных подхода:
//! 1. WSP (System Packages): адрес системного репозитория зашит в бинарный файл при компиляции.
//! 2. WPM (User Packages): динамические источники через прямой синтаксис (`Server = URL`).

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Базовый адрес системного репозитория WSP по умолчанию.
/// Может быть переопределен на этапе компиляции переменной окружения WSP_DEFAULT_REPO_URL.
pub const DEFAULT_WSP_REPO: &str = match option_env!("WSP_DEFAULT_REPO_URL") {
    Some(url) => url,
    None => "https://repo.wunix.org/system/x86_64",
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
