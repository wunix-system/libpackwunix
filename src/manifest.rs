//! Модуль манифеста пакета Wunix (`manifest.toml`).
//!
//! Определяет метаданные, целевую подсистему (System / User),
//! уровень критичности (Critical / Base / Desktop / Optional),
//! а также зависимости и системные триггеры.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::version::{DependencyReq, Version};

/// Тип пакета: системный (WSP) или пользовательский (WPM)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PackageType {
    /// Системный пакет ядра или базовой системы Wunix (WSP)
    #[default]
    System,
    /// Пользовательское или стороннее приложение (WPM)
    User,
}

impl PackageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageType::System => "system",
            PackageType::User => "user",
        }
    }

    pub fn parse(s: &str) -> Result<Self, &'static str> {
        match s.trim().to_ascii_lowercase().as_str() {
            "system" => Ok(PackageType::System),
            "user" => Ok(PackageType::User),
            _ => Err("Неизвестный тип пакета (ожидалось 'system' или 'user')"),
        }
    }
}

/// Уровень критичности системного пакета
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum PackageTier {
    /// Обычные необязательные пакеты (утилиты, игры)
    #[default]
    Optional,
    /// Графическое окружение (DE, композитор, GUI-приложения)
    Desktop,
    /// Базовые системные утилиты Wunix Base
    Base,
    /// Критически важные компоненты (ядро, libc, init). Блокируют установку стороннего ПО при устаревании
    Critical,
}

impl PackageTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageTier::Optional => "optional",
            PackageTier::Desktop => "desktop",
            PackageTier::Base => "base",
            PackageTier::Critical => "critical",
        }
    }

    pub fn parse(s: &str) -> Result<Self, &'static str> {
        match s.trim().to_ascii_lowercase().as_str() {
            "critical" => Ok(PackageTier::Critical),
            "base" => Ok(PackageTier::Base),
            "desktop" => Ok(PackageTier::Desktop),
            "optional" => Ok(PackageTier::Optional),
            _ => Err("Неизвестный tier (ожидалось: 'critical', 'base', 'desktop' или 'optional')"),
        }
    }
}

/// Манифест пакета Wunix
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageManifest {
    pub name: String,
    pub version: Version,
    pub arch: String,
    pub package_type: PackageType,
    pub tier: PackageTier,
    pub description: String,
    pub maintainer: String,

    pub required_deps: Vec<DependencyReq>,
    pub conflicts: Vec<String>,
    pub replaces: Vec<String>,

    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_remove: Option<String>,
    pub post_remove: Option<String>,
}

impl PackageManifest {
    /// Парсит манифест из строки формата TOML
    pub fn parse(input: &str) -> Result<Self, &'static str> {
        let mut manifest = PackageManifest::default();
        let mut current_section = "";

        // Построчный разбор с отслеживанием секций
        let mut in_array_key: Option<String> = None;
        let mut array_buffer: Vec<String> = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            // Пропуск пустых строк и комментариев
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Обработка продолжения многострочного массива
            if let Some(ref key) = in_array_key {
                if trimmed.starts_with(']') {
                    // Конец массива
                    assign_array(&mut manifest, current_section, key, &array_buffer)?;
                    in_array_key = None;
                    array_buffer.clear();
                    continue;
                }

                // Извлекаем элемент массива в кавычках
                if let Some(item) = extract_quoted_string(trimmed) {
                    array_buffer.push(item);
                }
                continue;
            }

            // Проверка заголовка секции: [package], [dependencies], [triggers]
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = &trimmed[1..trimmed.len() - 1].trim();
                continue;
            }

            // Разбор пары ключ = значение
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let val_part = trimmed[eq_pos + 1..].trim();

                // Проверка, является ли значение началом многострочного массива
                if val_part.starts_with('[') && !val_part.ends_with(']') {
                    in_array_key = Some(key.to_string());
                    array_buffer.clear();
                    // Проверяем, есть ли элементы на той же строке
                    let rest = val_part[1..].trim();
                    if let Some(item) = extract_quoted_string(rest) {
                        array_buffer.push(item);
                    }
                    continue;
                }

                // Однострочный массив []
                if val_part.starts_with('[') && val_part.ends_with(']') {
                    let inside = &val_part[1..val_part.len() - 1].trim();
                    let items = parse_inline_array(inside);
                    assign_array(&mut manifest, current_section, key, &items)?;
                    continue;
                }

                // Одиночное строковое значение
                let str_val = extract_quoted_string(val_part).unwrap_or_else(|| val_part.to_string());
                assign_scalar(&mut manifest, current_section, key, &str_val)?;
            }
        }

        // Валидация обязательных полей
        if manifest.name.is_empty() {
            return Err("Поле 'name' обязательно в секции [package]");
        }
        if manifest.arch.is_empty() {
            manifest.arch = "x86_64".to_string();
        }

        Ok(manifest)
    }

    /// Сериализует манифест в каноническую строку TOML
    pub fn to_toml_string(&self) -> String {
        let mut out = String::new();

        out.push_str("[package]\n");
        out.push_str(&format!("name = \"{}\"\n", self.name));
        out.push_str(&format!("version = \"{}\"\n", self.version));
        out.push_str(&format!("arch = \"{}\"\n", self.arch));
        out.push_str(&format!("type = \"{}\"\n", self.package_type.as_str()));
        out.push_str(&format!("tier = \"{}\"\n", self.tier.as_str()));
        out.push_str(&format!("description = \"{}\"\n", self.description));
        out.push_str(&format!("maintainer = \"{}\"\n\n", self.maintainer));

        out.push_str("[dependencies]\n");
        out.push_str("required = [\n");
        for dep in &self.required_deps {
            if dep.req.op == crate::version::CompareOp::Any {
                out.push_str(&format!("    \"{}\",\n", dep.name));
            } else {
                let op_str = match dep.req.op {
                    crate::version::CompareOp::GtEq => ">=",
                    crate::version::CompareOp::LtEq => "<=",
                    crate::version::CompareOp::Eq => "==",
                    crate::version::CompareOp::Gt => ">",
                    crate::version::CompareOp::Lt => "<",
                    _ => "",
                };
                out.push_str(&format!("    \"{} {} {}\",\n", dep.name, op_str, dep.req.version));
            }
        }
        out.push_str("]\n");

        out.push_str("conflicts = [\n");
        for c in &self.conflicts {
            out.push_str(&format!("    \"{}\",\n", c));
        }
        out.push_str("]\n");

        out.push_str("replaces = [\n");
        for r in &self.replaces {
            out.push_str(&format!("    \"{}\",\n", r));
        }
        out.push_str("]\n\n");

        out.push_str("[triggers]\n");
        if let Some(ref p) = self.pre_install {
            out.push_str(&format!("pre_install = \"{}\"\n", p));
        }
        if let Some(ref p) = self.post_install {
            out.push_str(&format!("post_install = \"{}\"\n", p));
        }
        if let Some(ref p) = self.pre_remove {
            out.push_str(&format!("pre_remove = \"{}\"\n", p));
        }
        if let Some(ref p) = self.post_remove {
            out.push_str(&format!("post_remove = \"{}\"\n", p));
        }

        out
    }
}

/// Назначение скалярного значения полю манифеста
fn assign_scalar(manifest: &mut PackageManifest, section: &str, key: &str, val: &str) -> Result<(), &'static str> {
    match section {
        "package" => match key {
            "name" => manifest.name = val.to_string(),
            "version" => manifest.version = Version::parse(val)?,
            "arch" => manifest.arch = val.to_string(),
            "type" => manifest.package_type = PackageType::parse(val)?,
            "tier" => manifest.tier = PackageTier::parse(val)?,
            "description" => manifest.description = val.to_string(),
            "maintainer" => manifest.maintainer = val.to_string(),
            _ => {}
        },
        "triggers" => match key {
            "pre_install" => manifest.pre_install = Some(val.to_string()),
            "post_install" => manifest.post_install = Some(val.to_string()),
            "pre_remove" => manifest.pre_remove = Some(val.to_string()),
            "post_remove" => manifest.post_remove = Some(val.to_string()),
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

/// Назначение массива значений полю манифеста
fn assign_array(manifest: &mut PackageManifest, section: &str, key: &str, items: &[String]) -> Result<(), &'static str> {
    if section == "dependencies" {
        match key {
            "required" => {
                for item in items {
                    manifest.required_deps.push(DependencyReq::parse(item)?);
                }
            }
            "conflicts" => {
                for item in items {
                    manifest.conflicts.push(item.clone());
                }
            }
            "replaces" => {
                for item in items {
                    manifest.replaces.push(item.clone());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Извлекает строку из кавычек: `"hello"` -> `hello`
fn extract_quoted_string(s: &str) -> Option<String> {
    let trimmed = s.trim();
    let start = trimmed.find('"')?;
    let end = trimmed[start + 1..].find('"')?;
    Some(trimmed[start + 1..start + 1 + end].to_string())
}

/// Разбирает однострочный список строк в кавычках: `"a", "b", "c"`
fn parse_inline_array(s: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut remaining = s;
    while let Some(quoted) = extract_quoted_string(remaining) {
        res.push(quoted);
        if let Some(start) = remaining.find('"') {
            if let Some(end) = remaining[start + 1..].find('"') {
                remaining = &remaining[start + 1 + end + 1..];
                continue;
            }
        }
        break;
    }
    res
}
