use std::sync::{Arc, Mutex};

use verso::VersoviewController;

/// Locks the [`VersoviewController`] and runs `f`, mapping both lock poisoning
/// and any verso IPC error to [`tauri_runtime::Error::FailedToSendMessage`].
pub(crate) fn with_controller<T, E>(
    cell: &Arc<Mutex<VersoviewController>>,
    f: impl FnOnce(&VersoviewController) -> Result<T, E>,
) -> tauri_runtime::Result<T> {
    let guard = cell
        .lock()
        .map_err(|_| tauri_runtime::Error::FailedToSendMessage)?;
    f(&guard).map_err(|_| tauri_runtime::Error::FailedToSendMessage)
}

pub fn to_verso_theme(theme: tauri_utils::Theme) -> verso::Theme {
    match theme {
        tauri_utils::Theme::Dark => verso::Theme::Dark,
        _ => verso::Theme::Light,
    }
}

pub fn from_verso_theme(theme: verso::Theme) -> tauri_utils::Theme {
    match theme {
        verso::Theme::Dark => tauri_utils::Theme::Dark,
        verso::Theme::Light => tauri_utils::Theme::Light,
    }
}

pub fn to_tao_theme(theme: tauri_utils::Theme) -> tao::window::Theme {
    match theme {
        tauri_utils::Theme::Dark => tao::window::Theme::Dark,
        _ => tao::window::Theme::Light,
    }
}
