//! Модуль бинарного формата контейнера пакетов Wunix (`WSP1`).
//!
//! Формат представляет собой 128-байтный заголовок прямого доступа,
//! за которым следуют секция цифровой подписи Ed25519, манифест
//! и сжатый пейлоад файлов (data.tar.gz).

extern crate alloc;
use alloc::vec::Vec;

use crate::manifest::{PackageManifest, PackageType};

/// Магическая 4-байтная сигнатура контейнера: 'W', 'S', 'P', '1'
pub const WSP_MAGIC: [u8; 4] = *b"WSP1";

/// Стандартное расширение пакетов Wunix Package (.wp)
pub const WP_EXTENSION: &str = ".wp";

/// Устаревшее расширение пакетов (для обратной совместимости)
pub const WSP_LEGACY_EXTENSION: &str = ".wsp";

/// Размер бинарного заголовка контейнера (128 байт для выравнивания и резерва)
pub const WSP_HEADER_SIZE: usize = 128;

/// Бинарный заголовок контейнера WSP/WPM
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WspHeader {
    pub magic: [u8; 4],
    pub format_version: u16,
    pub package_type: u8, // 0 = System (WSP), 1 = User (WPM)
    pub flags: u8,
    pub sig_offset: u32,
    pub sig_len: u32,
    pub meta_offset: u32,
    pub meta_len: u32,
    pub payload_offset: u64,
    pub payload_len: u64,
    pub payload_sha256: [u8; 32],
}

impl WspHeader {
    /// Сериализует заголовок в ровно 128 байт в порядке Little-Endian
    pub fn to_bytes(&self) -> [u8; WSP_HEADER_SIZE] {
        let mut b = [0u8; WSP_HEADER_SIZE];

        b[0..4].copy_from_slice(&self.magic);
        b[4..6].copy_from_slice(&self.format_version.to_le_bytes());
        b[6] = self.package_type;
        b[7] = self.flags;

        b[8..12].copy_from_slice(&self.sig_offset.to_le_bytes());
        b[12..16].copy_from_slice(&self.sig_len.to_le_bytes());

        b[16..20].copy_from_slice(&self.meta_offset.to_le_bytes());
        b[20..24].copy_from_slice(&self.meta_len.to_le_bytes());

        b[24..32].copy_from_slice(&self.payload_offset.to_le_bytes());
        b[32..40].copy_from_slice(&self.payload_len.to_le_bytes());

        b[40..72].copy_from_slice(&self.payload_sha256);
        // Байты 72..128 зарезервированы (нули)

        b
    }

    /// Десериализует заголовок из среза байтов
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < WSP_HEADER_SIZE {
            return Err("Слишком короткий буфер для заголовка WSP");
        }

        let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
        if magic != WSP_MAGIC {
            return Err("Неверная сигнатура пакета (ожидалось 'WSP1')");
        }

        let format_version = u16::from_le_bytes([bytes[4], bytes[5]]);
        let package_type = bytes[6];
        let flags = bytes[7];

        let sig_offset = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let sig_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        let meta_offset = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let meta_len = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);

        let payload_offset = u64::from_le_bytes([
            bytes[24], bytes[25], bytes[26], bytes[27],
            bytes[28], bytes[29], bytes[30], bytes[31],
        ]);
        let payload_len = u64::from_le_bytes([
            bytes[32], bytes[33], bytes[34], bytes[35],
            bytes[36], bytes[37], bytes[38], bytes[39],
        ]);

        let mut payload_sha256 = [0u8; 32];
        payload_sha256.copy_from_slice(&bytes[40..72]);

        Ok(Self {
            magic,
            format_version,
            package_type,
            flags,
            sig_offset,
            sig_len,
            meta_offset,
            meta_len,
            payload_offset,
            payload_len,
            payload_sha256,
        })
    }
}

/// Разобранный пакет Wunix
pub struct PackageContainer<'a> {
    pub header: WspHeader,
    pub manifest: PackageManifest,
    pub signature: Option<[u8; 64]>,
    pub payload: &'a [u8],
}

impl<'a> PackageContainer<'a> {
    /// Собирает пакет в единый бинарный буфер
    pub fn build(
        manifest: &PackageManifest,
        signature: Option<&[u8; 64]>,
        payload: &[u8],
    ) -> Result<Vec<u8>, &'static str> {
        let manifest_str = manifest.to_toml_string();
        let manifest_bytes = manifest_str.as_bytes();

        let sig_len = if signature.is_some() { 64u32 } else { 0u32 };
        let sig_offset = WSP_HEADER_SIZE as u32;

        let meta_offset = sig_offset + sig_len;
        let meta_len = manifest_bytes.len() as u32;

        let payload_offset = (meta_offset + meta_len) as u64;
        let payload_len = payload.len() as u64;

        let payload_sha256 = [0u8; 32];

        let pkg_type_byte = match manifest.package_type {
            PackageType::System => 0u8,
            PackageType::User => 1u8,
        };

        let header = WspHeader {
            magic: WSP_MAGIC,
            format_version: 1,
            package_type: pkg_type_byte,
            flags: 0,
            sig_offset,
            sig_len,
            meta_offset,
            meta_len,
            payload_offset,
            payload_len,
            payload_sha256,
        };

        let total_size = payload_offset as usize + payload.len();
        let mut buf = Vec::with_capacity(total_size);

        // 1. Заголовок
        buf.extend_from_slice(&header.to_bytes());

        // 2. Подпись (если есть)
        if let Some(sig) = signature {
            buf.extend_from_slice(sig);
        }

        // 3. Манифест
        buf.extend_from_slice(manifest_bytes);

        // 4. Пейлоад
        buf.extend_from_slice(payload);

        Ok(buf)
    }

    /// Парсит контейнер из среза байтов без полного копирования данных
    pub fn parse(bytes: &'a [u8]) -> Result<Self, &'static str> {
        let header = WspHeader::from_bytes(bytes)?;

        // Извлечение подписи
        let signature = if header.sig_len == 64 {
            let start = header.sig_offset as usize;
            let end = start + 64;
            if bytes.len() < end {
                return Err("Пакет поврежден: срез подписи выходит за границы файла");
            }
            let mut sig = [0u8; 64];
            sig.copy_from_slice(&bytes[start..end]);
            Some(sig)
        } else {
            None
        };

        // Извлечение манифеста
        let meta_start = header.meta_offset as usize;
        let meta_end = meta_start + header.meta_len as usize;
        if bytes.len() < meta_end {
            return Err("Пакет поврежден: срез манифеста выходит за границы файла");
        }
        let manifest_str = core::str::from_utf8(&bytes[meta_start..meta_end])
            .map_err(|_| "Манифест пакета содержит некорректный UTF-8")?;
        let manifest = PackageManifest::parse(manifest_str)?;

        // Извлечение пейлоада
        let payload_start = header.payload_offset as usize;
        let payload_end = payload_start + header.payload_len as usize;
        if bytes.len() < payload_end {
            return Err("Пакет поврежден: срез пейлоада выходит за границы файла");
        }
        let payload = &bytes[payload_start..payload_end];

        Ok(Self {
            header,
            manifest,
            signature,
            payload,
        })
    }

    /// Проверяет, является ли пакет системным (для WSP)
    pub fn is_system_package(&self) -> bool {
        self.manifest.package_type == PackageType::System
    }

    /// Проверяет, является ли пакет прикладным (для WPM)
    pub fn is_user_package(&self) -> bool {
        self.manifest.package_type == PackageType::User
    }
}
