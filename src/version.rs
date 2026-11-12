//! Модуль семантического версионирования пакетов Wunix (SemVer).
//!
//! Обеспечивает строгое сравнение версий x.y.z и сопоставление
//! с диапазонами зависимостей (например, "libwunix >= 0.2.0").

extern crate alloc;
use alloc::string::String;
use core::cmp::Ordering;
use core::fmt;

/// Семантическая версия пакета: major.minor[.patch][-commit]
#[derive(Debug, Clone, Copy, Default)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    tag: [u8; 16],
    tag_len: u8,
    has_patch: bool,
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.tag_slice() == other.tag_slice()
    }
}

impl Eq for Version {}

impl core::hash::Hash for Version {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.major.hash(state);
        self.minor.hash(state);
        self.patch.hash(state);
        self.tag_slice().hash(state);
    }
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            tag: [0; 16],
            tag_len: 0,
            has_patch: true,
        }
    }

    pub fn tag_slice(&self) -> &[u8] {
        &self.tag[..self.tag_len as usize]
    }

    /// Парсинг строки версии вида "0.4.2" или "0.1-070b090"
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        let s = s.trim();
        let (ver_core, tag_str) = match s.find('-') {
            Some(idx) => (&s[..idx], Some(&s[idx + 1..])),
            None => (s, None),
        };

        let mut parts = ver_core.split('.');
        let major_str = parts.next().ok_or("Отсутствует major-номер версии")?;
        let minor_str = parts.next().unwrap_or("0");
        let patch_opt = parts.next();
        let has_patch = patch_opt.is_some();
        let patch_str = patch_opt.unwrap_or("0");

        if parts.next().is_some() {
            return Err("Версия содержит более 3 сегментов");
        }

        let major = major_str.parse::<u32>().map_err(|_| "Некорректный major номер")?;
        let minor = minor_str.parse::<u32>().map_err(|_| "Некорректный minor номер")?;
        let patch = patch_str.parse::<u32>().map_err(|_| "Некорректный patch номер")?;

        let mut tag = [0u8; 16];
        let mut tag_len = 0u8;
        if let Some(t) = tag_str {
            let t = t.trim();
            if t.is_empty() {
                return Err("Пустой суффикс версии после '-'");
            }
            let bytes = t.as_bytes();
            let copy_len = bytes.len().min(16);
            tag[..copy_len].copy_from_slice(&bytes[..copy_len]);
            tag_len = copy_len as u8;
        }

        Ok(Self {
            major,
            minor,
            patch,
            tag,
            tag_len,
            has_patch,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_patch || self.tag_len == 0 {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        } else {
            write!(f, "{}.{}", self.major, self.minor)?;
        }
        if self.tag_len > 0 {
            let tag_str = core::str::from_utf8(self.tag_slice()).unwrap_or("");
            write!(f, "-{}", tag_str)?;
        }
        Ok(())
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
            .then_with(|| self.tag_slice().cmp(other.tag_slice()))
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
