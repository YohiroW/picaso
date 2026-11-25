import { useLayoutEffect, useRef, useState } from "react";
import { Label } from "../../components/ui/label";
import { desktop, sourceName, type CredentialStatus } from "../../lib/desktop";

type Page = "connection" | "saved";
const titles = { connection: "Connection", saved: "API key" };

interface Props {
  status: CredentialStatus;
  onChange: (value: CredentialStatus) => void;
  onClose: () => void;
  refreshError?: string;
}

function NavigationIcon({ page }: { page: Page }) {
  return <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor"
    strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    {page === "connection"
      ? <path d="m14 7 2-2a4.2 4.2 0 0 1 6 6l-4 4a4.2 4.2 0 0 1-6 0m-2 2-2 2a4.2 4.2 0 0 1-6-6l4-4a4.2 4.2 0 0 1 6 0m-4 5 8-8" />
      : <><circle cx="8" cy="9" r="5"/><path d="m12 12 8 8 2-2-2-2 1.5-1.5-2-2-1.5 1.5-3.5-3.5"/></>}
  </svg>;
}

export function KeySettings({ status, onChange, onClose, refreshError }: Props) {
  const [page, setPage] = useState<Page>(
    !status.selected && !status.environment && !status.keychain ? "saved" : "connection"
  );
  const [key, setKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [confirmRemove, setConfirmRemove] = useState(false);
  const heading = useRef<HTMLHeadingElement>(null);
  useLayoutEffect(() => { heading.current?.focus({ preventScroll: true }); }, [page]);

  function navigate(next: Page) {
    setError("");
    setConfirmRemove(false);
    setPage(next);
  }
  async function change(action: () => Promise<CredentialStatus>, close = true) {
    setBusy(true);
    setError("");
    try {
      const next = await action();
      setKey("");
      onChange(next);
      if (close) onClose();
    } catch (e) { setError(String(e)); }
    finally { setBusy(false); setConfirmRemove(false); }
  }

  return <main className="settings-layout">
    <aside className="settings-sidebar">
      <button type="button" className="settings-back" onClick={onClose} disabled={busy}>
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M20 12H4m6-6-6 6 6 6"/></svg>
        Back to workspace
      </button>
      <h1>Settings</h1>
      <nav aria-label="Settings">
        {(["connection", "saved"] as const).map(id => <button type="button" key={id}
          aria-current={id === page ? "page" : undefined} onClick={() => navigate(id)} disabled={busy}>
          <NavigationIcon page={id}/>{titles[id]}
        </button>)}
      </nav>
    </aside>

    <div className="settings-scroll" key={page}>
      <section className="settings-detail" aria-labelledby="settings-page-title">
        <h2 ref={heading} id="settings-page-title" tabIndex={-1}>{titles[page]}</h2>
        <p className="settings-intro">{page === "connection"
          ? "Choose which OpenAI API key Picaso uses to create images."
          : "Store an OpenAI API key in your macOS login keychain."}</p>
        {status.preferenceError && <p role="alert">Your saved key preference could not be read. Choose a source again.</p>}
        {refreshError && <p role="alert">{refreshError}</p>}

        <fieldset disabled={busy}>
          {page === "connection" ? <>
            <div className="settings-row">
              <h3>Selected source</h3>
              <span>{status.selected ? sourceName(status.selected) : "None selected"}
                {status.selected && !status[status.selected] && " (unavailable)"}</span>
            </div>
            <div className="settings-row">
              <div><h3>Environment variable</h3><p><code>OPENAI_API_KEY</code> is {status.environment ? "available" : "unavailable"} in this session.</p></div>
              {status.environment && status.selected !== "environment" &&
                <button onClick={() => change(() => desktop.select("environment"))}>Use environment key</button>}
            </div>
            <div className="settings-row">
              <div><h3>macOS Keychain</h3><p>{status.keychainError ? "Unlock your login keychain to access your saved key." : status.keychain ? "A key is saved on this Mac." : "No key saved on this Mac."}</p></div>
              <div className="settings-actions">{status.keychain && status.selected !== "keychain" &&
                <button onClick={() => change(() => desktop.select("keychain"))}>Use saved key</button>}
                <button onClick={() => navigate("saved")}>{status.keychain ? "Manage key" : "Add key…"}</button>
              </div>
            </div>
          </> : <>
            {status.keychainError && <p role="alert">Unlock your login keychain to access your saved key.</p>}
            <form className="settings-key-form" onSubmit={e => { e.preventDefault(); void change(() => desktop.save(key)); }}>
              <Label htmlFor="api-key">{status.keychain ? "Replacement API key" : "OpenAI API key"}</Label>
              <input id="api-key" type="password" autoComplete="off" spellCheck={false} value={key} onChange={e => setKey(e.target.value)}/>
              <div className="settings-form-actions">
                <button type="button" className="link" onClick={() => desktop.openPortal().catch(e => setError(String(e)))}>Get an API key ↗</button>
                <button className="primary" disabled={!key.trim()}>Save and use key</button>
              </div>
            </form>
            {status.keychain && <div className="settings-row">
              <div><h3>Remove saved key</h3><p>{confirmRemove ? "Remove this key from your login keychain?" : "Your images will remain in the library."}</p></div>
              <div className="settings-actions">{confirmRemove
                ? <><button onClick={() => setConfirmRemove(false)}>Cancel</button><button className="remove-key" onClick={() => change(desktop.remove, false)}>Confirm removal</button></>
                : <button className="remove-key" onClick={() => setConfirmRemove(true)}>Remove saved key</button>}
              </div>
            </div>}
          </>}
        </fieldset>
        {error && <p role="alert">{error}</p>}
        <p className="settings-billing">Images use your OpenAI API credits. ChatGPT subscriptions are billed separately.</p>
      </section>
    </div>
  </main>;
}
