import { invoke } from "@tauri-apps/api/core";

export interface InstallationInfo {
  version: string;
  dataDirectory: string;
}

export function getInstallationInfo() {
  return invoke<InstallationInfo>("installation_info");
}
