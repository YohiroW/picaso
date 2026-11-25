import { convertFileSrc, invoke } from "@tauri-apps/api/core";

export type Source = "environment" | "keychain";
export interface CredentialStatus {
  selected: Source | null;
  environment: boolean;
  keychain: boolean;
  keychainError: boolean;
  preferenceError?: boolean;
}
export interface Model {
  provider: string; id: string; displayName: string; canEdit: boolean;
  sizes: string[]; customSize: boolean; qualities: string[]; formats: string[]; backgrounds: string[];
  maxImages: number; defaults: ImageOptions;
}
export interface Job {
  id: string; prompt: string; provider: string; model: string; images: string[];
  parameters: {size: string; quality: string; output_format: string; n?:number; background?:string};
  status: string; message: string | null; inputId: string | null; created: number;
}
export interface ImageOptions {
  size: string; quality: string; n: number; output_format: string; background: string;
}

export const imageURL = (id: string) => convertFileSrc(id, "picaso-image");
export const desktop = {
  importReference: async (file: File) => {
    if (file.size >= 50_000_000) throw new Error("Choose a reference image smaller than 50 MB.");
    return invoke<string>("import_reference", await file.arrayBuffer());
  },
  discardReference: (id: string) => invoke<void>("discard_reference", { id }),
  history: (recentOnly = false) => invoke<Job[]>("history", { recentOnly }),
  models: () => invoke<Model[]>("available_models"),
  submit: (draft: { requestId: string; model: string; prompt: string; inputId?: string; options: ImageOptions }) => invoke<string>("submit_job", { draft }),
  export: (id: string) => invoke<boolean>("export_image", { id }),
  deleteImage: (id: string) => invoke<void>("delete_image", { id }),
  status: () => invoke<CredentialStatus>("credential_status"),
  select: (source: Source) => invoke<CredentialStatus>("select_source", { source }),
  save: (key: string) => invoke<CredentialStatus>("save_key", { key }),
  remove: () => invoke<CredentialStatus>("remove_key"),
  openPortal: () => invoke<void>("open_key_portal"),
};
export const sourceName = (source: Source) => source === "environment" ? "Environment variable" : "macOS Keychain";
