use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, State};

use crate::{application::dto::request::memorydto::MemoryDTO, state::appstate::AppState};

#[tauri::command]
pub async fn load_memory(
    state: State<'_, AppState>,
    path: String,
) -> Result<Option<MemoryDTO>, String> {
    state.memory_use_case.load_memory(&path)
}

#[tauri::command]
pub async fn save_memory(
    state: State<'_, AppState>,
    path: String,
    memory: MemoryDTO,
) -> Result<(), String> {
    state.memory_use_case.save_memory(&path, &memory)
}

#[tauri::command]
pub async fn default_memory_path(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let install_dir: Option<PathBuf> = if cfg!(windows) {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
    } else {
        None
    };

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve the app data folder: {e}"))?;

    state
        .memory_use_case
        .default_memory_path(install_dir.as_deref(), &app_data_dir)
}
