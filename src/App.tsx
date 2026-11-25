import { useEffect, useState } from "react";
import { KeySettings } from "./features/onboarding/KeySettings";
import { desktop, type CredentialStatus } from "./lib/desktop";
import { Workspace } from "./features/workspace/Workspace";
import { AppBar } from "./components/AppBar";

export function App() {
  const [status, setStatus] = useState<CredentialStatus>();
  const [settings, setSettings] = useState(false);
  const [error, setError] = useState("");
  function load() {
    setError("");
    desktop.status().then(next => { setStatus(next); setSettings(!next.selected && !next.preferenceError); }).catch(e => setError(String(e)));
  }
  useEffect(load, []);
  function openSettings() {
    setSettings(true);
    desktop.status().then(setStatus).catch(e => setError(String(e)));
  }
  return <>
    {(!status || settings) && <AppBar/>}
    {!status ? <main><p role="status">{error || "Opening Picaso…"}</p>{error && <button onClick={load}>Try again</button>}</main> :
      <>
        {settings && <KeySettings status={status} onChange={setStatus} onClose={() => setSettings(false)} />}
        <div hidden={settings}><Workspace status={status} openSettings={openSettings} /></div>
      </>}
  </>;
}
