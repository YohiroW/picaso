use super::*;
use std::cell::RefCell;

#[derive(Default)]
struct MemoryVault(RefCell<Option<String>>);
impl Vault for &MemoryVault {
    fn get(&self) -> Result<Option<String>, String> {
        Ok(self.0.borrow().clone())
    }
    fn set(&self, value: &str) -> Result<(), String> {
        self.0.replace(Some(value.into()));
        Ok(())
    }
    fn remove(&self) -> Result<(), String> {
        self.0.replace(None);
        Ok(())
    }
}

#[test]
fn explicit_account_survives_restart_and_never_falls_back() {
    let folder = tempfile::tempdir().unwrap();
    let vault = MemoryVault::default();
    let service = Credentials::new(
        folder.path().into(),
        &vault,
        Some("environment-fixture".into()),
    );
    assert_eq!(service.status().unwrap().selected, None);
    assert!(service.resolve().is_err());
    service.select(Source::Environment).unwrap();
    assert_eq!(service.resolve().unwrap(), "environment-fixture");
    service.save("manual-fixture").unwrap();
    let restarted = Credentials::new(
        folder.path().into(),
        &vault,
        Some("environment-fixture".into()),
    );
    assert_eq!(restarted.resolve().unwrap(), "manual-fixture");
    restarted.remove().unwrap();
    assert!(restarted.resolve().is_err());
    restarted.select(Source::Environment).unwrap();
    let missing = Credentials::new(folder.path().into(), &vault, None);
    assert!(missing.resolve().is_err());
    let config = std::fs::read_to_string(folder.path().join("credential-source.json")).unwrap();
    assert!(!config.contains("fixture"));
    assert!(!serde_json::to_string(&service.status().unwrap())
        .unwrap()
        .contains("fixture"));
}

#[cfg(target_os = "macos")]
#[test]
fn mac_keychain_round_trip_for_an_isolated_account() {
    let dir = tempfile::tempdir().unwrap();
    let vault = MacKeychain(format!("test-{}", dir.path().display()));
    vault.set("fictional-picaso-test-key").unwrap();
    let read = vault.get();
    vault.remove().unwrap();
    assert_eq!(read.unwrap().as_deref(), Some("fictional-picaso-test-key"));
    assert!(vault.get().unwrap().is_none());
}
