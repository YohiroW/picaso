#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod credentials;
use credentials::{Credentials, MacKeychain, Source, Status};
use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallationInfo {
    version: String,
    data_directory: std::path::PathBuf,
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

type Account = Mutex<Credentials<MacKeychain>>;
fn account(state: &Account) -> Result<std::sync::MutexGuard<'_, Credentials<MacKeychain>>, String> {
    state
        .lock()
        .map_err(|_| "Reopen Picaso to restore key settings.".into())
}
#[tauri::command]
fn credential_status(state: State<Account>) -> Result<Status, String> {
    account(&state)?.status()
}
#[tauri::command]
fn select_source(state: State<Account>, source: Source) -> Result<Status, String> {
    account(&state)?.select(source)
}
#[tauri::command]
fn save_key(state: State<Account>, key: String) -> Result<Status, String> {
    account(&state)?.save(&key)
}
#[tauri::command]
fn remove_key(state: State<Account>) -> Result<Status, String> {
    account(&state)?.remove()
}
#[tauri::command]
fn open_key_portal() -> Result<(), String> {
    if std::process::Command::new("open")
        .arg("https://platform.openai.com/api-keys")
        .status()
        .is_ok_and(|s| s.success())
    {
        Ok(())
    } else {
        Err("Open https://platform.openai.com/api-keys in your browser.".into())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let root = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&root)?;
            app.manage(Mutex::new(Credentials::new(
                root,
                MacKeychain("openai-api-key".into()),
                std::env::var("OPENAI_API_KEY").ok(),
            )));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            installation_info,
            credential_status,
            select_source,
            save_key,
            remove_key,
            open_key_portal
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Picaso");
}
