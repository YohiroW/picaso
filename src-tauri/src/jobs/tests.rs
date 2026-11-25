use super::*;
use crate::providers::Fixture;

#[test]
fn repeated_submission_is_stored_once_and_export_survives_restart() {
    let temp = tempfile::tempdir().unwrap();
    let lib = Library::open(temp.path()).unwrap();
    let fixture = Fixture::default();
    let draft = Submission::new("Folder icon");
    let id = submit(&lib, &fixture, &draft).unwrap();
    assert_eq!(submit(&lib, &fixture, &draft).unwrap(), id);
    assert_eq!(fixture.calls.get(), 1);
    drop(lib);
    let reopened = Library::open(temp.path()).unwrap();
    let jobs = reopened.history().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].prompt, "Folder icon");
    assert_eq!(jobs[0].provider, "fixture");
    assert_eq!(jobs[0].status, "succeeded");
    let export_dir = tempfile::tempdir().unwrap();
    let path = export_dir.path().join("export.png");
    reopened.export(&jobs[0].images[0], &path).unwrap();
    assert_eq!(
        std::fs::read(path).unwrap(),
        include_bytes!("../../icons/icon.png")
    );
}

#[test]
fn write_failure_leaves_a_failed_task_without_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let lib = Library::open(temp.path()).unwrap();
    std::fs::remove_dir(temp.path().join("originals")).unwrap();
    let fixture = Fixture::default();
    assert!(submit(&lib, &fixture, &Submission::new("An icon")).is_err());
    let job = lib.history().unwrap().pop().unwrap();
    assert_eq!(job.status, "failed");
    assert!(job.images.is_empty());
}

#[test]
fn editing_the_second_original_preserves_the_first_job_and_allows_branching() {
    let folder = tempfile::tempdir().unwrap();
    let library = Library::open(folder.path()).unwrap();
    let provider = Fixture::default();
    let parent = submit(&library, &provider, &Submission::new("Two icon studies")).unwrap();
    let images = library.history().unwrap().remove(0).images;
    assert_eq!(images.len(), 2);
    assert_ne!(
        library.original(&images[0]).unwrap(),
        library.original(&images[1]).unwrap()
    );
    for text in ["Make it warmer", "Try a rounded form"] {
        let mut draft = Submission::new(text);
        draft.input_id = Some(images[1].clone());
        submit(&library, &provider, &draft).unwrap();
        let job = library.history().unwrap().remove(0);
        assert_eq!(job.input_id.as_ref(), Some(&images[1]));
        // The test provider echoes the exact original supplied to an edit.
        assert_eq!(
            library.original(&job.images[0]).unwrap(),
            library.original(&images[1]).unwrap()
        );
    }
    assert_eq!(provider.calls.get(), 3);
    let original = library.history().unwrap().pop().unwrap();
    assert_eq!(original.id, parent);
    assert_eq!(original.images, images);
    std::fs::remove_file(
        folder
            .path()
            .join("originals")
            .join(format!("{}.png", images[1])),
    )
    .unwrap();
    let mut missing = Submission::new("Repair");
    missing.input_id = Some(images[1].clone());
    assert!(submit(&library, &provider, &missing).is_err());
    assert_eq!(provider.calls.get(), 3);
    assert_eq!(library.history().unwrap().len(), 3);
}

#[test]
fn unsupported_model_rejects_edits_before_calling_the_provider() {
    use crate::providers::{Failure, Model, Output};
    struct GenerateOnly;
    impl Provider for GenerateOnly {
        fn model(&self) -> Model {
            Model {
                provider: "fixture",
                id: "generate-only",
                can_edit: false,
            }
        }
        fn render(&self, _: &str, _: Option<&[u8]>) -> Result<Output, Failure> {
            panic!("unsupported edit reached the provider")
        }
    }
    let root = tempfile::tempdir().unwrap();
    let lib = Library::open(root.path()).unwrap();
    let mut draft = Submission::new("Keep the source");
    draft.input_id = Some(Uuid::new_v4().to_string());
    let error = submit(&lib, &GenerateOnly, &draft).unwrap_err();
    assert!(error.to_string().contains("cannot edit"));
    assert!(lib.history().unwrap().is_empty());
}
