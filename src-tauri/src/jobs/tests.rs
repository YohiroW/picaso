use super::*;
use crate::media::Original;
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
    let jobs = reopened.history(false).unwrap();
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
    let job = lib.history(true).unwrap().pop().unwrap();
    assert_eq!(job.status, "failed");
    assert!(job.images.is_empty());
}

#[test]
fn editing_the_second_original_preserves_the_first_job_and_allows_branching() {
    let folder = tempfile::tempdir().unwrap();
    let library = Library::open(folder.path()).unwrap();
    let provider = Fixture::default();
    let mut parent_request = Submission::new("Two icon studies");
    parent_request.options.n = 2;
    let parent = submit(&library, &provider, &parent_request).unwrap();
    let images = library.history(false).unwrap().remove(0).images;
    assert_eq!(images.len(), 2);
    assert_ne!(
        library.original(&images[0]).unwrap().bytes,
        library.original(&images[1]).unwrap().bytes
    );
    for text in ["Make it warmer", "Try a rounded form"] {
        let mut draft = Submission::new(text);
        draft.input_id = Some(images[1].clone());
        submit(&library, &provider, &draft).unwrap();
        let job = library.history(false).unwrap().remove(0);
        assert_eq!(job.input_id.as_ref(), Some(&images[1]));
        // The test provider echoes the exact original supplied to an edit.
        assert_eq!(
            library.original(&job.images[0]).unwrap().bytes,
            library.original(&images[1]).unwrap().bytes
        );
    }
    assert_eq!(provider.calls.get(), 3);
    let original = library.history(false).unwrap().pop().unwrap();
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
    assert_eq!(library.history(false).unwrap().len(), 3);
}

#[test]
fn deleting_an_original_preserves_siblings_edits_and_request_deduplication() {
    let root = tempfile::tempdir().unwrap();
    let library = Library::open(root.path()).unwrap();
    let provider = Fixture::default();
    let mut batch = Submission::new("Two posters");
    batch.options.n = 2;
    submit(&library, &provider, &batch).unwrap();
    let images = library.history(true).unwrap().remove(0).images;
    let id = Uuid::parse_str(&images[0]).unwrap();
    let path = root.path().join("originals").join(format!("{id}.png"));
    let mut edit = Submission::new("A warmer poster");
    edit.input_id = Some(images[0].clone());
    submit(&library, &provider, &edit).unwrap();
    let edited = library.history(false).unwrap()[0].images[0].clone();

    library.delete_image(id).unwrap();
    library.delete_image(id).unwrap();
    assert!(!path.exists());
    assert!(library.original(&images[0]).is_err());
    for recent in [false, true] {
        let jobs = library.history(recent).unwrap();
        assert_eq!(jobs[1].images, images[1..]);
        assert_eq!(jobs[0].input_id, edit.input_id);
        assert_eq!(jobs[0].images.as_slice(), std::slice::from_ref(&edited));
    }
    // Simulate an interruption between the committed deletion and file cleanup.
    std::fs::write(&path, b"pending removal").unwrap();
    drop(library);
    let library = Library::open(root.path()).unwrap();
    assert!(!path.exists());
    assert!(library.original(&edited).is_ok());
    assert!(library.original(&images[1]).is_ok());
    library
        .delete_image(Uuid::parse_str(&images[1]).unwrap())
        .unwrap();
    for recent in [false, true] {
        let jobs = library.history(recent).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, edit.request_id.to_string());
    }
    submit(&library, &provider, &batch).unwrap();
    assert_eq!(provider.calls.get(), 2);
}

#[test]
fn unsupported_model_rejects_edits_before_calling_the_provider() {
    use crate::providers::{Failure, Model, Output};
    struct GenerateOnly;
    impl Provider for GenerateOnly {
        fn model(&self) -> Model {
            Model {
                provider: "fixture".into(),
                id: "generate-only".into(),
                can_edit: false,
                ..crate::catalog::fixture()
            }
        }
        fn render(
            &self,
            _: &str,
            _: Option<&Original>,
            _: &ImageOptions,
        ) -> Result<Output, Failure> {
            panic!("unsupported edit reached the provider")
        }
    }
    let root = tempfile::tempdir().unwrap();
    let lib = Library::open(root.path()).unwrap();
    let mut draft = Submission::new("Keep the source");
    draft.input_id = Some(Uuid::new_v4().to_string());
    let error = submit(&lib, &GenerateOnly, &draft).unwrap_err();
    assert!(error.to_string().contains("cannot edit"));
    assert!(lib.history(false).unwrap().is_empty());
}

#[test]
fn reopen_recovers_interrupted_job_and_orphan_without_resending() {
    use crate::providers::{Failure, Model, Output};
    struct Interrupted;
    impl Provider for Interrupted {
        fn model(&self) -> Model {
            Fixture::default().model()
        }
        fn render(
            &self,
            _: &str,
            _: Option<&Original>,
            _: &ImageOptions,
        ) -> Result<Output, Failure> {
            panic!("simulate exit after dispatch")
        }
    }
    let root = tempfile::tempdir().unwrap();
    let lib = Library::open(root.path()).unwrap();
    let fixture = Fixture::default();
    submit(&lib, &fixture, &Submission::new("Finished icon")).unwrap();
    let old = lib.history(false).unwrap().remove(0);
    let draft = Submission::new("Interrupted request");
    assert!(std::panic::catch_unwind(|| submit(&lib, &Interrupted, &draft)).is_err());
    assert_eq!(lib.history(true).unwrap()[0].status, "running");
    let orphan = root
        .path()
        .join("originals")
        .join(format!("{}.png", Uuid::new_v4()));
    std::fs::write(&orphan, b"uncommitted file").unwrap();
    let temp = root.path().join("originals/.picaso-tmp-interrupted");
    std::fs::write(&temp, b"partial file").unwrap();
    assert!(Library::open(root.path()).is_err());
    drop(lib);
    let reopened = Library::open(root.path()).unwrap();
    let interrupted = reopened.history(true).unwrap().remove(0);
    assert_eq!(interrupted.status, "unconfirmed");
    assert!(interrupted.message.unwrap().contains("another charge"));
    assert!(!orphan.exists() && !temp.exists());
    assert_eq!(
        reopened.original(&old.images[0]).unwrap().bytes,
        include_bytes!("../../icons/icon.png")
    );
    assert!(submit(&reopened, &fixture, &draft).is_err());
    assert_eq!(fixture.calls.get(), 1);
}

#[test]
fn a_newer_database_is_left_intact() {
    let root = tempfile::tempdir().unwrap();
    let db = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    db.pragma_update(None, "user_version", 99).unwrap();
    assert!(Library::open(root.path()).is_err());
    assert_eq!(
        db.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        99
    );
}

#[test]
fn duplicate_in_flight_request_does_not_invoke_a_second_provider() {
    use crate::providers::{Failure, Model, Output};
    use std::sync::mpsc;
    struct Held {
        started: mpsc::Sender<()>,
        resume: mpsc::Receiver<()>,
    }
    impl Provider for Held {
        fn model(&self) -> Model {
            Fixture::default().model()
        }
        fn render(
            &self,
            p: &str,
            input: Option<&Original>,
            options: &ImageOptions,
        ) -> Result<Output, Failure> {
            self.started.send(()).unwrap();
            self.resume.recv().unwrap();
            Fixture::default().render(p, input, options)
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let library = Library::open(dir.path()).unwrap();
    let draft = Submission::new("One submission");
    std::thread::scope(|scope| {
        let (started, rx) = mpsc::channel();
        let (resume, wait) = mpsc::channel();
        let worker = scope.spawn(|| {
            submit(
                &library,
                &Held {
                    started,
                    resume: wait,
                },
                &draft,
            )
        });
        rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        let other = Fixture::default();
        let duplicate = submit(&library, &other, &draft);
        resume.send(()).unwrap();
        worker.join().unwrap().unwrap();
        assert!(duplicate.is_err());
        assert_eq!(other.calls.get(), 0);
    });
    assert_eq!(library.history(false).unwrap().len(), 1);
}

#[test]
fn chosen_options_are_validated_recorded_and_used_for_deduplication() {
    let root = tempfile::tempdir().unwrap();
    let lib = Library::open(root.path()).unwrap();
    let provider = Fixture::default();
    let mut draft = Submission::new("Portrait study");
    draft.options.size = "1024x1536".into();
    draft.options.quality = "medium".into();
    submit(&lib, &provider, &draft).unwrap();
    let job = lib.history(true).unwrap().pop().unwrap();
    assert_eq!(job.parameters["size"], "1024x1536");
    assert_eq!(job.parameters["quality"], "medium");
    draft.options.quality = "high".into();
    assert!(submit(&lib, &provider, &draft).is_err());
    draft.request_id = Uuid::new_v4();
    draft.options.size = "unexpected".into();
    assert!(submit(&lib, &provider, &draft).is_err());
    assert_eq!(provider.calls.get(), 1);
}

#[test]
fn every_output_format_survives_reopen_and_exports_the_original_bytes() {
    use crate::media::Format;
    let root = tempfile::tempdir().unwrap();
    let library = Library::open(root.path()).unwrap();
    let provider = Fixture::default();
    for format in [
        Format::Png,
        Format::Jpeg,
        Format::Webp,
        Format::Svg,
        Format::Gif,
    ] {
        let mut draft = Submission::new("Mixed media");
        draft.options.output_format = format;
        draft.options.n = if format == Format::Svg { 1 } else { 3 };
        submit(&library, &provider, &draft).unwrap();
    }
    drop(library);
    let library = Library::open(root.path()).unwrap();
    let dest = tempfile::tempdir().unwrap();
    for job in library.history(false).unwrap() {
        let format = Format::parse(job.parameters["output_format"].as_str().unwrap()).unwrap();
        assert_eq!(
            job.images.len(),
            job.parameters["n"].as_u64().unwrap() as usize
        );
        for id in job.images {
            let image = library.original(&id).unwrap();
            assert_eq!(image.format, format);
            format.validate(&image.bytes).unwrap();
            let path = dest.path().join(format!("{}.{}", id, format.extension()));
            library.export(&id, &path).unwrap();
            assert_eq!(std::fs::read(path).unwrap(), image.bytes);
            if !format.editable() {
                let calls = provider.calls.get();
                let mut draft = Submission::new("Change color");
                draft.input_id = Some(id);
                assert!(submit(&library, &provider, &draft).is_err());
                assert_eq!(calls, provider.calls.get());
            }
        }
    }
}

#[test]
fn invalid_combinations_never_reserve_or_dispatch_a_request() {
    use crate::media::Format;
    let root = tempfile::tempdir().unwrap();
    let lib = Library::open(root.path()).unwrap();
    let provider = Fixture::default();
    for (format, bg, n, edit) in [
        (Format::Jpeg, "transparent", 1, false),
        (Format::Svg, "auto", 2, false),
        (Format::Svg, "auto", 1, true),
        (Format::Png, "auto", 5, false),
    ] {
        let mut draft = Submission::new("Should stay in draft");
        draft.options.output_format = format;
        draft.options.background = bg.into();
        draft.options.n = n;
        if edit {
            draft.input_id = Some(Uuid::new_v4().to_string());
        }
        assert!(submit(&lib, &provider, &draft).is_err());
    }
    assert_eq!(provider.calls.get(), 0);
    assert!(lib.history(false).unwrap().is_empty());
}
#[test]
fn migrates_v1_png_without_changing_originals_or_recharging_a_retry() {
    let root = tempfile::tempdir().unwrap();
    let id = Uuid::new_v4();
    let draft = Submission::new("Old PNG");
    let db = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    db.execute_batch("CREATE TABLE jobs(id TEXT PRIMARY KEY,prompt TEXT,provider TEXT,model TEXT,status TEXT,parameters TEXT,input_id TEXT,created INTEGER,message TEXT,usage TEXT);
       CREATE TABLE images(id TEXT PRIMARY KEY,job TEXT REFERENCES jobs(id)); PRAGMA user_version=1;").unwrap();
    db.execute(
        "INSERT INTO jobs VALUES (?1,?2,'fixture','test-image','succeeded',?3,NULL,1,NULL,NULL)",
        rusqlite::params![
            draft.request_id.to_string(),
            draft.prompt,
            r#"{"size":"1024x1024","quality":"low","output_format":"png","n":1}"#
        ],
    )
    .unwrap();
    db.execute(
        "INSERT INTO images VALUES(?1,?2)",
        [id.to_string(), draft.request_id.to_string()],
    )
    .unwrap();
    drop(db);
    let dir = root.path().join("originals");
    std::fs::create_dir(&dir).unwrap();
    let png = include_bytes!("../../icons/icon.png");
    std::fs::write(dir.join(format!("{id}.png")), png).unwrap();
    let lib = Library::open(root.path()).unwrap();
    let provider = Fixture::default();
    assert_eq!(lib.original(&id.to_string()).unwrap().bytes, png);
    assert_eq!(
        submit(&lib, &provider, &draft).unwrap(),
        draft.request_id.to_string()
    );
    assert_eq!(provider.calls.get(), 0);
}
