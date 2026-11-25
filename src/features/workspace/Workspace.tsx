import { useEffect, useLayoutEffect, useRef, useState, type FormEvent } from "react";
import { Label } from "../../components/ui/label";
import { AppBar } from "../../components/AppBar";
import { desktop, imageURL, type CredentialStatus, type ImageOptions, type Job, type Model } from "../../lib/desktop";
import { Gallery, type GalleryImage } from "./Gallery";
import { ImagePreview } from "./ImagePreview";

const defaults: ImageOptions = { size: "1024x1024", quality: "low" };
type Draft = { prompt: string; options: ImageOptions };
const emptyDraft: Draft = { prompt: "", options: defaults };

function recentImages(jobs: Job[]): GalleryImage[] {
  return jobs.filter(job => job.status === "succeeded").reverse()
    .flatMap(job => job.images.map((id, index) => ({ id, index, job }))).slice(-8);
}

function History({ jobs, open }: { jobs: Job[]; open: (image: GalleryImage) => void }) {
  if (!jobs.length) return <p className="muted">No tasks yet.</p>;
  return <ol className="task-list">{jobs.map(job => <li key={job.id}>
    <div className="task-meta"><time dateTime={new Date(job.created * 1000).toISOString()}>{new Date(job.created * 1000).toLocaleString()}</time>
      <span>{job.model}</span></div>
    <p className="task-prompt">{job.prompt}</p>
    {job.status !== "succeeded" && <p className="task-status"><strong>{job.status === "unconfirmed" ? "Result unconfirmed" : job.status}</strong>{job.message && <> · {job.message}</>}</p>}
    {job.images.length > 0 && <div className="task-images">{job.images.map((id, index) =>
      <button key={id} aria-label={`View image ${index + 1}: ${job.prompt}`} onClick={() => open({id, index, job})}>
        <img alt="" src={imageURL(id)} loading="lazy" decoding="async" />
      </button>)}</div>}
  </li>)}</ol>;
}

function HistoryIcon() {
  return <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    <path d="M3 6v5h5M3.6 10a8.5 8.5 0 1 1 .9 6M12 7v5l3 2"/>
  </svg>;
}

export function Workspace({ status, openSettings }: { status: CredentialStatus; openSettings: () => void }) {
  const [history, setHistory] = useState(false);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [model, setModel] = useState<Model>();
  const [revision, refresh] = useState(0);
  const [loadError, setLoadError] = useState("");
  const [loading, setLoading] = useState(true);
  const [preview, setPreview] = useState<GalleryImage>();
  const [reference, setReference] = useState<GalleryImage>();
  const [drafts, setDrafts] = useState<Record<string, Draft>>({ new: emptyDraft });
  const [busy, setBusy] = useState(false);
  const inFlight = useRef(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [mobileEditor, setMobileEditor] = useState(false);
  const promptField = useRef<HTMLTextAreaElement>(null);
  const [focusRequest, setFocusRequest] = useState(0);
  const draftKey = reference?.id ?? "new";
  const draft = drafts[draftKey] ?? emptyDraft;
  const available = model?.provider === "fixture" || !!(status.selected && status[status.selected]);
  const canSubmit = available && draft.prompt.trim() && (!reference || model?.canEdit) && !busy;

  useEffect(() => {
    let active = true;
    setLoading(true);
    setLoadError("");
    Promise.all([desktop.history(!history), desktop.model()]).then(([results, current]) => {
      if (active) { setJobs(results); setModel(current); }
    }).catch(e => { if (active) setLoadError(String(e)); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [history, revision]);

  useLayoutEffect(() => {
    if (focusRequest) promptField.current?.focus();
  }, [focusRequest]);

  function focusEditor() {
    setMobileEditor(true);
    setFocusRequest(n => n + 1);
  }
  function updateDraft(changes: Partial<Draft>) {
    setDrafts(prev => ({ ...prev, [draftKey]: { ...(prev[draftKey] ?? emptyDraft), ...changes } }));
  }
  function newImage() {
    setReference(undefined);
    setPreview(undefined);
    setError("");
    setNotice("");
    focusEditor();
  }
  function edit(image: GalleryImage) {
    setDrafts(prev => prev[image.id] ? prev : { ...prev, [image.id]: { ...draft } });
    setReference(image);
    setPreview(undefined);
    setError("");
    setNotice("");
    focusEditor();
  }
  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!canSubmit || inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await desktop.submit({ requestId: crypto.randomUUID(), prompt: draft.prompt, inputId: reference?.id, options: draft.options });
      setDrafts(prev => {
        const next = { ...prev };
        if (reference) delete next[reference.id];
        else next.new = { prompt: "", options: draft.options };
        return next;
      });
      setReference(undefined);
      setHistory(false);
      setNotice("Image saved to your gallery.");
    } catch (e) { setError(String(e)); }
    finally {
      inFlight.current = false;
      setBusy(false);
      refresh(n => n + 1);
    }
  }

  return <>
    <AppBar><nav aria-label="Workspace">
      <button className="history-nav" aria-pressed={history} onClick={() => { setHistory(v => !v); setPreview(undefined); }}>
        <HistoryIcon />{history ? "Back to studio" : "History"}
      </button>
      <button onClick={openSettings}>Key settings</button>
    </nav></AppBar>
    <main className="studio">
      <aside className="control-panel" aria-label="Image controls">
        <button className="mobile-toggle" aria-expanded={mobileEditor} aria-controls="image-form" onClick={() => setMobileEditor(v => !v)}>Image controls <span aria-hidden="true">{mobileEditor ? "−" : "+"}</span></button>
        <form id="image-form" className={mobileEditor ? "editor expanded" : "editor"} onSubmit={submit}>
          <section className="panel-section prompt-section">
            <Label className="section-title" htmlFor="prompt">{reference ? "Edit instructions" : "Prompt"}</Label>
            <div className="prompt-box">
              {reference && <div className="reference"><img src={imageURL(reference.id)} alt="Reference image"/>
                <span>Reference image</span><button type="button" aria-label="Remove reference" disabled={busy} onClick={newImage}>×</button></div>}
              <textarea id="prompt" ref={promptField} value={draft.prompt} disabled={busy} onChange={e => updateDraft({prompt:e.target.value})}
                placeholder={reference ? "Describe what to change…" : "Describe what you want to see…"}
                onKeyDown={e => {
                  if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && !e.nativeEvent.isComposing && e.nativeEvent.keyCode !== 229) {
                    e.preventDefault();
                    e.currentTarget.form?.requestSubmit();
                  }
                }}/>
              <div className="prompt-footer"><kbd title="Command + Return">⌘ ↩</kbd>
                <button type="submit" className="primary submit" disabled={!canSubmit}>{busy ? "Creating…" : reference ? "Apply edit" : "Generate image"}<span aria-hidden="true">↑</span></button>
              </div>
            </div>
            {model && !available && <p className="feedback">Choose a key to create images. <button type="button" className="link" onClick={openSettings}>Key settings</button></p>}
            {reference && model && !model.canEdit && <p role="alert">This model cannot edit a reference. Remove it to generate a new image.</p>}
            {error && <p role="alert" className="feedback">{error}</p>}
            {notice && <p role="status" className="feedback">{notice}</p>}
          </section>
          <section className="panel-section"><h2>Model</h2>
            <div className="model-value">{model?.id ?? "Loading…"}<small>{model?.provider === "fixture" ? "Local test provider" : "OpenAI"}</small></div>
          </section>
          <section className="panel-section"><h2>Image settings</h2>
            <div className="control-row"><Label htmlFor="image-size">Size &amp; orientation</Label>
              <select id="image-size" value={draft.options.size} disabled={busy} onChange={e => updateDraft({options:{...draft.options,size:e.target.value as ImageOptions["size"]}})}>
                <option value="1024x1024">Square · 1024 × 1024</option><option value="1536x1024">Landscape · 1536 × 1024</option><option value="1024x1536">Portrait · 1024 × 1536</option>
              </select></div>
            <div className="control-row"><Label htmlFor="image-quality">Quality</Label>
              <select id="image-quality" value={draft.options.quality} disabled={busy} onChange={e => updateDraft({options:{...draft.options,quality:e.target.value as ImageOptions["quality"]}})}>
                <option value="low">Low</option><option value="medium">Medium</option><option value="high">High</option>
              </select></div>
            <div className="control-row"><span>Number of images</span><span>1</span></div>
          </section>
          <details className="panel-section advanced"><summary>Advanced</summary><div className="control-row"><span>Output format</span><span>PNG</span></div></details>
        </form>
      </aside>
      <section className="gallery-pane" aria-label={history ? "Task history" : "Recent creations"}>
        <h1>{history ? "History" : "Recent creations"}</h1>
        {loadError ? <p role="alert">{loadError} <button onClick={() => refresh(n => n + 1)}>Try again</button></p> :
          loading ? <p role="status">Loading images…</p> :
          history ? <History jobs={jobs} open={setPreview}/> :
          <Gallery images={recentImages(jobs)} open={setPreview} create={newImage} busy={busy}/>}
      </section>
    </main>
    {preview && <ImagePreview key={preview.id} image={preview} close={() => setPreview(undefined)} edit={() => edit(preview)} busy={busy}/>}
  </>;
}
