fn main() {
    let manifest = tauri_build::AppManifest::new().commands(&[
        "installation_info",
        "credential_status",
        "select_source",
        "save_key",
        "remove_key",
        "open_key_portal",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("failed to build Picaso");
}
