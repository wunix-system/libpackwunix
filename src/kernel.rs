//! Модуль управления ядрами в каталоге `/boot` и загрузчиком Limine.
//!
//! Реализует схему безопасного A/B обновления с автоматическим
//! резервированием предыдущего рабочего ядра в limine.conf.

extern crate alloc;
use alloc::format;
use alloc::string::String;

use crate::version::Version;

/// Запись ядра для загрузчика
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelEntry {
    pub version: Version,
    pub kernel_path: String,
    pub cmdline: String,
}

/// Генерирует содержимое файла `/boot/limine.conf` с текущим и резервным ядрами
pub fn generate_limine_conf(
    current: &KernelEntry,
    fallback: Option<&KernelEntry>,
    timeout_seconds: u32,
) -> String {
    let mut out = String::new();

    out.push_str(&format!("timeout: {}\n", timeout_seconds));
    out.push_str("default_entry: 1\n");
    out.push_str("serial: yes\n\n");

    // Основное (текущее) ядро
    out.push_str(&format!("/Wunix OS (Current: v{})\n", current.version));
    out.push_str("    protocol: limine\n");
    out.push_str(&format!("    kernel_path: {}\n", current.kernel_path));
    out.push_str(&format!("    cmdline: {}\n\n", current.cmdline));

    // Резервное ядро (если установлено)
    if let Some(fb) = fallback {
        out.push_str(&format!("/Wunix OS (Fallback: v{})\n", fb.version));
        out.push_str("    protocol: limine\n");
        out.push_str(&format!("    kernel_path: {}\n", fb.kernel_path));
        out.push_str(&format!("    cmdline: {}\n\n", fb.cmdline));
    }

    out
}
