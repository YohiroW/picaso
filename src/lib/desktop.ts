import { invoke } from "@tauri-apps/api/core";

export type Source = "environment" | "keychain";
export interface CredentialStatus {
  selected: Source | null;
  environment: boolean;
  keychain: boolean;
  keychainError: boolean;
}
export const desktop = {
  status: () => invoke<CredentialStatus>("credential_status"),
  select: (source: Source) => invoke<CredentialStatus>("select_source", { source }),
  save: (key: string) => invoke<CredentialStatus>("save_key", { key }),
  remove: () => invoke<CredentialStatus>("remove_key"),
  openPortal: () => invoke<void>("open_key_portal"),
};
export const sourceName = (source: Source) => source === "environment" ? "Environment variable" : "macOS Keychain";
