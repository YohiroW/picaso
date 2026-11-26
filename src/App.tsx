import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { KeySettings } from "./features/onboarding/KeySettings";
import { desktop, type CredentialStatus } from "./lib/desktop";
import { Workspace } from "./features/workspace/Workspace";
import { AppBar } from "./components/AppBar";

export function App() {
  const [status, setStatus] = useState<CredentialStatus>();
  const [settings, setSettings] = useState(false);
  const [error, setError] = useState("");
  const returnTarget = useRef<HTMLButtonElement | null>(null);
  const workspaceScroll = useRef(0);
  const returning = useRef(false);
  const refreshId = useRef(0);

  function load() {
    setError("");
    desktop.status().then(next => {
      setStatus(next);
      const unsupportedSource = next.selected === "keychain" && next.platform !== "macos";
      setSettings(unsupportedSource || (!next.selected && !next.preferenceError));
    }).catch(e => setError(String(e)));
  }
  useEffect(load, []);
  function openSettings(trigger: HTMLButtonElement) {
    returnTarget.current = trigger;
    workspaceScroll.current = window.scrollY;
    setSettings(true);
    setError("");
    const id = ++refreshId.current;
    desktop.status().then(next => {
      if (id === refreshId.current) setStatus(next);
    }).catch(e => { if (id === refreshId.current) setError(String(e)); });
  }
  function closeSettings() {
    refreshId.current++;
    returning.current = true;
    setSettings(false);
  }
  useLayoutEffect(() => {
    if (settings && window.scrollY) window.scrollTo(0, 0);
    if (!settings && returning.current) {
      returning.current = false;
      const trigger = returnTarget.current;
      const target = trigger?.isConnected ? trigger : document.getElementById("workspace-settings");
      target?.focus({ preventScroll: true });
      if (window.scrollY !== workspaceScroll.current) window.scrollTo(0, workspaceScroll.current);
    }
  }, [settings]);
  return <>
    {!status && <AppBar/>}
    {!status ? <main><p role="status">{error || "Opening Picaso…"}</p>{error && <button onClick={load}>Try again</button>}</main> :
      <>
        {settings && <KeySettings status={status} refreshError={error} onChange={next => { refreshId.current++; setStatus(next); setError(""); }} onClose={closeSettings} />}
        <div hidden={settings}><Workspace status={status} openSettings={openSettings} /></div>
      </>}
  </>;
}
