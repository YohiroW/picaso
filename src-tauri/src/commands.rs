use crate::{
    catalog::Catalog,
    jobs::Submission,
    library::{Job, Library},
    providers::Model,
};
use tauri::State;

#[tauri::command]
pub async fn import_reference(
    library: State<'_, Library>,
    request: tauri::ipc::Request<'_>,
) -> Result<String, String> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err("Choose an image file to attach.".into());
    };
    let library = library.inner().clone();
    let bytes = bytes.clone();
    tauri::async_runtime::spawn_blocking(move || {
        library.import_reference(&bytes).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn discard_reference(library: State<Library>, id: uuid::Uuid) -> Result<(), String> {
    library.discard_reference(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn history(library: State<Library>, recent_only: Option<bool>) -> Result<Vec<Job>, String> {
    library
        .history(recent_only.unwrap_or(false))
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn delete_image(library: State<Library>, id: uuid::Uuid) -> Result<(), String> {
    library.delete_image(id).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn available_models(catalog: State<Catalog>) -> Result<Vec<Model>, String> {
    let models = catalog.models().map_err(|e| format!("{e:#}"))?;
    #[cfg(feature = "fixture-provider")]
    {
        let _ = models;
        Ok(vec![crate::catalog::fixture()])
    }
    #[cfg(not(feature = "fixture-provider"))]
    {
        Ok(models)
    }
}
#[tauri::command]
pub fn current_model(catalog: State<Catalog>) -> Result<Model, String> {
    available_models(catalog)?
        .into_iter()
        .next()
        .ok_or_else(|| "No configured models.".into())
}
#[tauri::command]
pub async fn submit_job(
    library: State<'_, Library>,
    state: State<'_, crate::Account>,
    catalog: State<'_, Catalog>,
    draft: Submission,
) -> Result<String, String> {
    let models = available_models(catalog)?;
    let selected = match &draft.model {
        Some(id) => models.into_iter().find(|m| m.id == *id),
        None => models.into_iter().next(),
    }
    .ok_or("The selected model is no longer configured.")?;
    let library = library.inner().clone();
    #[cfg(not(feature = "fixture-provider"))]
    let key = crate::account(&state)?.resolve()?;
    #[cfg(feature = "fixture-provider")]
    let _ = (state, selected);
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(not(feature = "fixture-provider"))]
        let provider = crate::providers::openai::OpenAi::new(key, selected)?;
        #[cfg(feature = "fixture-provider")]
        let provider = crate::providers::Fixture::default();
        crate::jobs::submit(&library, &provider, &draft).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn export_image(library: State<'_, Library>, id: String) -> Result<bool, String> {
    let id = uuid::Uuid::parse_str(&id)
        .map_err(|_| "Choose an image from History.")?
        .to_string();
    let format = library.image_format(&id).map_err(|e| e.to_string())?;
    let ext = format.extension();
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_file_name(format!("Picaso-{id}.{ext}"))
        .add_filter(format!("{} image", ext.to_uppercase()), &[ext])
        .save_file()
        .await
    else {
        return Ok(false);
    };
    library
        .export(&id, file.path())
        .map_err(|e| e.to_string())?;
    Ok(true)
}
