#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallationInfo {
    version: String,
    data_directory: PathBuf,
}

#[tauri::command]
fn installation_info(app: tauri::AppHandle) -> Result<InstallationInfo, String> {
    let data_directory = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_directory).map_err(|e| e.to_string())?;
    Ok(InstallationInfo {
        version: app.package_info().version.to_string(),
        data_directory,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![installation_info])
        .run(tauri::generate_context!())
        .expect("failed to run Picaso");
}
