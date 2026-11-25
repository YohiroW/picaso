#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod credentials;
mod jobs;
mod library;
mod providers;
use commands::*;
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
        .register_asynchronous_uri_scheme_protocol("picaso-image", |context, request, responder| {
            let library = context
                .app_handle()
                .state::<library::Library>()
                .inner()
                .clone();
            let id = request.uri().path().trim_start_matches('/').to_string();
            tauri::async_runtime::spawn_blocking(move || {
                let response = match library.original(&id) {
                    Ok(bytes) => tauri::http::Response::builder()
                        .header("Content-Type", "image/png")
                        .body(bytes),
                    Err(_) => tauri::http::Response::builder()
                        .status(404)
                        .body(Vec::new()),
                };
                responder.respond(response.expect("static image response"));
            });
        })
        .setup(|app| {
            let root = app.path().app_local_data_dir()?;
            #[cfg(feature = "fixture-provider")]
            let root = root.join("fixture-workspace");
            app.manage(library::Library::open(&root)?);
            let account_name = if cfg!(feature = "fixture-provider") {
                "fixture-api-key"
            } else {
                "openai-api-key"
            };
            app.manage(Mutex::new(Credentials::new(
                root,
                MacKeychain(account_name.into()),
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
            open_key_portal,
            history,
            current_model,
            submit_job,
            export_image
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Picaso");
}
