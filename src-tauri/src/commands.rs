use crate::{
    jobs::Submission,
    library::{Job, Library},
    providers::Model,
};
use tauri::State;

#[tauri::command]
pub fn history(library: State<Library>) -> Result<Vec<Job>, String> {
    library.history().map_err(|e| e.to_string())
}
#[tauri::command]
pub fn current_model() -> Model {
    #[cfg(feature = "fixture-provider")]
    {
        use crate::providers::Provider;
        crate::providers::Fixture::default().model()
    }
    #[cfg(not(feature = "fixture-provider"))]
    {
        Model {
            provider: "openai",
            id: "gpt-image-2",
            can_edit: true,
        }
    }
}
#[tauri::command]
pub async fn submit_job(library: State<'_, Library>, draft: Submission) -> Result<String, String> {
    #[cfg(feature = "fixture-provider")]
    {
        let library = library.inner().clone();
        tauri::async_runtime::spawn_blocking(move || {
            crate::jobs::submit(&library, &crate::providers::Fixture::default(), &draft)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
    #[cfg(not(feature = "fixture-provider"))]
    {
        let _ = (library, draft);
        Err("Image generation is available in the test-provider build.".into())
    }
}
#[tauri::command]
pub async fn export_image(library: State<'_, Library>, id: String) -> Result<bool, String> {
    let id = uuid::Uuid::parse_str(&id)
        .map_err(|_| "Choose an image from History.")?
        .to_string();
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_file_name(format!("Picaso-{id}.png"))
        .add_filter("PNG image", &["png"])
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
