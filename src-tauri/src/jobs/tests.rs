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
