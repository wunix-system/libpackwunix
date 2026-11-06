//! Модуль семантического версионирования пакетов Wunix (SemVer).
//!
//! Обеспечивает строгое сравнение версий x.y.z и сопоставление
//! с диапазонами зависимостей (например, "libwunix >= 0.2.0").

extern crate alloc;
use alloc::string::String;
use core::cmp::Ordering;
use core::fmt;

/// Семантическая версия пакета: major.minor.patch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Парсинг строки версии вида "0.4.2"
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        let s = s.trim();
        let mut parts = s.split('.');

        let major_str = parts.next().ok_or("Отсутствует major-номер версии")?;
        let minor_str = parts.next().unwrap_or("0");
        let patch_str = parts.next().unwrap_or("0");

        if parts.next().is_some() {
            return Err("Версия содержит более 3 сегментов");
        }

        let major = major_str.parse::<u32>().map_err(|_| "Некорректный major номер")?;
        let minor = minor_str.parse::<u32>().map_err(|_| "Некорректный minor номер")?;
        let patch = patch_str.parse::<u32>().map_err(|_| "Некорректный patch номер")?;

        Ok(Self { major, minor, patch })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
    }
}

/// Оператор сравнения версий
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompareOp {
    #[default]
    Any,
    Eq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

/// Требование к версии пакета
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VersionReq {
    pub op: CompareOp,
    pub version: Version,
}

impl VersionReq {
    pub const fn any() -> Self {
        Self {
            op: CompareOp::Any,
            version: Version::new(0, 0, 0),
        }
    }

    /// Проверяет, удовлетворяет ли конкретная версия данному требованию
    pub fn matches(&self, candidate: &Version) -> bool {
        match self.op {
            CompareOp::Any => true,
            CompareOp::Eq => candidate == &self.version,
            CompareOp::Gt => candidate > &self.version,
            CompareOp::GtEq => candidate >= &self.version,
            CompareOp::Lt => candidate < &self.version,
            CompareOp::LtEq => candidate <= &self.version,
        }
    }
}

/// Требование зависимости: имя пакета + ограничение версии
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyReq {
    pub name: String,
    pub req: VersionReq,
}

impl DependencyReq {
    /// Парсит строку вида "libwunix >= 0.2.0" или просто "nano"
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        let s = s.trim();
        if s.is_empty() {
            return Err("Пустая строка зависимости");
        }

        // Проверяем наличие операторов сравнения
        let (name, op, v_str) = if let Some(idx) = s.find(">=") {
            (&s[..idx], CompareOp::GtEq, &s[idx + 2..])
        } else if let Some(idx) = s.find("<=") {
            (&s[..idx], CompareOp::LtEq, &s[idx + 2..])
        } else if let Some(idx) = s.find("==") {
            (&s[..idx], CompareOp::Eq, &s[idx + 2..])
        } else if let Some(idx) = s.find('=') {
            (&s[..idx], CompareOp::Eq, &s[idx + 1..])
        } else if let Some(idx) = s.find('>') {
            (&s[..idx], CompareOp::Gt, &s[idx + 1..])
        } else if let Some(idx) = s.find('<') {
            (&s[..idx], CompareOp::Lt, &s[idx + 1..])
        } else {
            return Ok(Self {
                name: String::from(s.trim()),
                req: VersionReq::any(),
            });
        };

        let name = name.trim();
        if name.is_empty() {
            return Err("Имя зависимости не может быть пустым");
        }

        let version = Version::parse(v_str.trim())?;

        Ok(Self {
            name: String::from(name),
            req: VersionReq { op, version },
        })
    }
}
