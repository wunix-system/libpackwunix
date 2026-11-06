#![cfg_attr(not(feature = "std"), no_std)]

//! # libpackwunix
//!
//! Единая библиотека управления пакетами операционной системы Wunix.
//! Используется подсистемами:
//! - **WSP (Wunix System Package)**: управление ядром, wunix-base и системными компонентами.
//! - **WPM (Wunix Package Manager)**: управление пользовательскими приложениями и репозиториями.
//!
//! Библиотека компилируется как в среде bare-metal Wunix (`no_std` + `alloc`),
//! так и в стандартной среде Linux/Unix (`std`) для утилит хоста.

extern crate alloc;

pub mod daemon;
pub mod format;
pub mod kernel;
pub mod manifest;
pub mod sources;
pub mod version;
