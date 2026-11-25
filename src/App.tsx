import { useEffect, useState } from "react";
import { KeySettings } from "./features/onboarding/KeySettings";
import { desktop, type CredentialStatus } from "./lib/desktop";
import { Workspace } from "./features/workspace/Workspace";
import icon from "./assets/picaso.svg";

export function App() {
  const [status, setStatus] = useState<CredentialStatus>();
  const [settings, setSettings] = useState(false);
  const [error, setError] = useState("");
  function load() {
    setError("");
    desktop.status().then(next => { setStatus(next); setSettings(!next.selected); }).catch(e => setError(String(e)));
  }
  useEffect(load, []);
  return <>
    <header className="appbar"><div className="brand"><img src={icon} alt="" width="28" height="28" />Picaso</div>
      {status && !settings && <button onClick={() => setSettings(true)}>Key settings</button>}
    </header>
    {!status ? <main><p role="status">{error || "Opening Picaso…"}</p>{error && <button onClick={load}>Try again</button>}</main> :
      <>
        {settings && <KeySettings status={status} onChange={setStatus} onClose={() => setSettings(false)} />}
        <div hidden={settings}><Workspace status={status} openSettings={() => setSettings(true)} /></div>
      </>}
  </>;
}
