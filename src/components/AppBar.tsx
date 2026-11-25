import type { ReactNode } from "react";
import icon from "../../src-tauri/icons/icon.png";

export function AppBar({ children }: { children?: ReactNode }) {
  return <header className="appbar"><div className="brand"><img src={icon} alt="" width="28" height="28"/>Picaso</div>{children}</header>;
}
