import { Label } from "../../components/ui/label";
import { useState } from "react";
import { desktop, sourceName, type CredentialStatus } from "../../lib/desktop";

interface Props {
  status: CredentialStatus;
  onChange: (value: CredentialStatus) => void;
  onClose: () => void;
}
export function KeySettings({ status, onChange, onClose }: Props) {
  const [key, setKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [confirmRemove, setConfirmRemove] = useState(false);
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
  return <main className="settings">
    <h1>{status.selected ? "Key settings" : "Connect to OpenAI"}</h1>
    <p>Images use your OpenAI API account’s credits. ChatGPT subscriptions are billed separately.</p>
    {status.preferenceError && <p role="alert">Your saved key preference could not be read. Choose a source again.</p>}
    {status.selected && <p className="source">Selected source · {sourceName(status.selected)}{!status[status.selected] && " (unavailable)"}</p>}
    <fieldset disabled={busy}>
      {status.environment && <section><h2>Environment variable</h2>
        <p>Picaso detected <code>OPENAI_API_KEY</code> for this session.</p>
        <button onClick={() => change(() => desktop.select("environment"))}>Use environment key</button>
      </section>}
      <section><h2>macOS Keychain</h2>
        {status.keychainError && <p role="alert">Unlock your login keychain to access your saved key.</p>}
        {status.keychain && <div className="actions">
          <button onClick={() => change(() => desktop.select("keychain"))}>Use saved key</button>
          {confirmRemove ? <><button onClick={() => change(desktop.remove, false)}>Confirm removal</button><button onClick={() => setConfirmRemove(false)}>Cancel</button></> :
            <button onClick={() => setConfirmRemove(true)}>Remove saved key</button>}
        </div>}
        <form onSubmit={e => { e.preventDefault(); void change(() => desktop.save(key)); }}>
          <Label htmlFor="api-key">{status.keychain ? "Replacement API key" : "OpenAI API key"}</Label>
          <input id="api-key" type="password" autoComplete="off" spellCheck={false} value={key} onChange={e => setKey(e.target.value)} />
          <div className="actions"><button className="primary" disabled={!key.trim()}>Save and use key</button>
            <button type="button" className="link" onClick={() => desktop.openPortal().catch(e => setError(String(e)))}>Get an API key ↗</button></div>
        </form>
      </section>
      <button className="link" onClick={onClose}>{status.selected ? "Back to workspace" : "Browse without a key"}</button>
    </fieldset>
    {error && <p role="alert">{error}</p>}
  </main>;
}
