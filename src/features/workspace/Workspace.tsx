import { useEffect, useRef, useState, type FormEvent } from "react";
import { desktop, imageURL, type CredentialStatus, type Job, type Model } from "../../lib/desktop";

export function Workspace({status, openSettings}: {status: CredentialStatus; openSettings: () => void}) {
  const [items, setItems] = useState<Job[]>([]);
  const [model, setModel] = useState<Model>();
  const [selectedId, setSelectedId] = useState<string>();
  const [imageId, setImageId] = useState<string>();
  const [prompt, setPrompt] = useState("");
  const [reference, setReference] = useState<string>();
  const promptField = useRef<HTMLTextAreaElement>(null);
  const [busy, setBusy] = useState(false);
  const inFlight = useRef(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const selected = items.find(item => item.id === selectedId);
  const available = model?.provider === "fixture" || !!(status.selected && status[status.selected]);

  function select(job: Job | undefined, image?: string) {
    setSelectedId(job?.id); setImageId(image ?? job?.images[0]); setNotice("");
  }
  useEffect(() => {
    let active = true;
    Promise.all([desktop.history(), desktop.model()]).then(([jobs, model]) => {
      if (active) { setItems(jobs); setModel(model); select(jobs[0]); }
    }).catch(e => { if (active) setError(String(e)); });
    return () => { active = false; };
  }, []);
  async function submit(e: FormEvent) {
    e.preventDefault();
    if ((reference && !model?.canEdit) || !prompt.trim() || !available || inFlight.current) return;
    inFlight.current = true; setBusy(true); setError(""); setNotice("");
    try {
      const id = await desktop.submit({requestId: crypto.randomUUID(), prompt, inputId: reference});
      setPrompt(""); setReference(undefined);
      const jobs = await desktop.history();
      setItems(jobs); select(jobs.find(job => job.id === id));
    } catch (e) {
      setError(String(e));
      try { setItems(await desktop.history()); } catch { /* Keep the last visible history. */ }
    } finally { inFlight.current = false; setBusy(false); }
  }
  async function save() {
    if (!imageId) return;
    try { if (await desktop.export(imageId)) setNotice("Original saved."); }
    catch (e) { setError(String(e)); }
  }
  return <main className="studio">
    <aside className="history"><h2>History</h2>
      {items.length === 0 && <p className="quiet">Your images will appear here.</p>}
      {items.map(job => <button className="history-item" key={job.id} aria-pressed={job.id === selectedId}
        onClick={() => select(job)}>{job.images[0] && <img src={imageURL(job.images[0])} alt="" loading="lazy" />}
        <span>{job.prompt}<small>{job.status === "succeeded" ? new Date(job.created*1000).toLocaleDateString() : job.status}</small></span></button>)}
    </aside>
    <div className="desk"><section className="viewer" aria-label="Image preview">
      {imageId ? <img className="original" key={imageId} src={imageURL(imageId)} alt={selected?.prompt ?? "Selected image"} onError={() => setError("The original image is missing. Restore it or choose another image.")}/> :
        <div className="empty"><h1>{selected ? "Task details" : "Create your first image"}</h1><p>{selected?.message ?? "Describe what you want to make below."}</p></div>}
    </section>
    {selected && <div className="details"><h2>{selected.prompt}</h2><p>{selected.provider} · {selected.model}</p>
      {selected.images.length > 1 && <div className="variants">{selected.images.map((id,i) => <button key={id} aria-pressed={id === imageId} aria-label={`Select image ${i+1}`} onClick={() => select(selected,id)}><img alt="" src={imageURL(id)} /></button>)}</div>}
      {imageId && <div className="actions"><button disabled={busy} onClick={() => {
        setReference(imageId); setError(""); setNotice(""); promptField.current?.focus();
      }}>Continue editing</button><button onClick={save}>Save original…</button></div>}</div>}
    <form className="composer" onSubmit={submit}>
      {reference && <div className="reference"><img src={imageURL(reference)} alt="Selected original for editing" /><span>Reference image</span><button type="button" disabled={busy} onClick={() => setReference(undefined)}>Remove reference</button></div>}
      {reference && model && !model.canEdit && <p role="alert">This model cannot edit a reference. Remove it to generate a new image.</p>}
      <label htmlFor="prompt">{reference ? "Edit instructions" : "Image prompt"}</label>
      <textarea ref={promptField} id="prompt" onKeyDown={e => {
        if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && !e.nativeEvent.isComposing && e.nativeEvent.keyCode !== 229) {
          e.preventDefault(); e.currentTarget.form?.requestSubmit();
        }
      }} rows={3} value={prompt} disabled={busy} onChange={e => setPrompt(e.target.value)} placeholder={reference ? "What would you like to change?" : "Describe your image…"} />
      <div className="submit-row"><small>{model?.provider === "fixture" ? "Local test images" : "OpenAI · " + (model?.id ?? "…")}</small>
        <button className="primary" disabled={busy || !available || !prompt.trim() || (!!reference && !model?.canEdit)}>{busy ? "Creating…" : reference ? "Apply edit" : "Generate image"}</button></div>
      {!available && model && <p>Choose an available key to generate images. <button type="button" className="link" onClick={openSettings}>Key settings</button></p>}
    </form>
    {error && <p role="alert">{error}</p>}{notice && <p role="status">{notice}</p>}
    </div>
  </main>;
}
