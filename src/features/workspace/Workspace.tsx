import { useEffect, useLayoutEffect, useRef, useState, type FormEvent } from "react";
import { ImageSettings, optionsError } from "./ImageSettings";
import { Label } from "../../components/ui/label";
import { SettingsIcon } from "../../components/SettingsIcon";
import { AppBar } from "../../components/AppBar";
import { desktop, imageURL, type CredentialStatus, type ImageOptions, type Job, type Model } from "../../lib/desktop";
import { Gallery, type GalleryImage } from "./Gallery";
import { ImagePreview } from "./ImagePreview";


type Draft = { prompt: string; modelId?: string; options?: ImageOptions; custom?: boolean };
const emptyDraft: Draft = { prompt: "" };

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

export function Workspace({ status, openSettings }: { status: CredentialStatus; openSettings: (trigger: HTMLButtonElement) => void }) {
  const [history, setHistory] = useState(false);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [catalogError,setCatalogError] = useState("");
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
  const model = draft.modelId ? models.find(m=>m.id===draft.modelId) : models[0];
  const options = draft.options ?? model?.defaults;
  const validation = model && options ? optionsError(model,options,!!reference) : undefined;
  const available = model?.provider === "fixture" || !!(status.selected && status[status.selected]);
  const canSubmit = model && options && !validation && available && draft.prompt.trim() && (!reference || model?.canEdit) && !busy;

  useEffect(() => {
    let active = true;
    setLoading(true);
    setLoadError("");
    desktop.history(!history).then(results => {
      if (active) setJobs(results);
    }).catch(e => { if (active) setLoadError(String(e)); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [history, revision]);

  useEffect(() => {
    let active=true;
    desktop.models().then(result=>{if(active){setModels(result);setCatalogError("");}})
      .catch(e=>{if(active){setCatalogError(String(e));setModels([]);}});
    return ()=>{active=false;};
  }, [revision]);

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
    if (!model || !options || !canSubmit || inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await desktop.submit({ requestId: crypto.randomUUID(), model: model.id, prompt: draft.prompt, inputId: reference?.id, options });
      setDrafts(prev => {
        const next = { ...prev };
        if (reference) delete next[reference.id];
        else next.new = { ...draft, prompt: "", modelId:model.id, options };
        return next;
      });
      setReference(undefined);
      setHistory(false);
      setNotice("Saved to your gallery.");
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
      <button id="workspace-settings" className="settings-trigger" aria-label="Settings" title="Settings" onClick={e => openSettings(e.currentTarget)}><SettingsIcon/></button>
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
                <button type="submit" className="primary submit" disabled={!canSubmit}>{busy ? "Creating…" : reference ? (options && options.n>1 ? `Apply edit · ${options.n} images` : "Apply edit") : (options && options.n>1 ? `Generate ${options.n} images` : "Generate image")}<span aria-hidden="true">↑</span></button>
              </div>
            </div>
            {model && !available && <p className="feedback">Choose a key to create images. <button type="button" className="link" onClick={e => openSettings(e.currentTarget)}>Settings</button></p>}
            {reference && model && !model.canEdit && <p role="alert">This model cannot edit a reference. Remove it to generate a new image.</p>}
            {error && <p role="alert" className="feedback">{error}</p>}
            {notice && <p role="status" className="feedback">{notice}</p>}
          </section>
          {catalogError && <p role="alert" className="panel-section">{catalogError} <button type="button" onClick={()=>refresh(n=>n+1)}>Reload models</button></p>}
          {!catalogError && !model && draft.modelId && models.length>0 && <p role="alert" className="panel-section">The selected model is unavailable. <button type="button" onClick={()=>updateDraft({modelId:models[0].id,options:models[0].defaults,custom:false})}>Use {models[0].displayName}</button></p>}
          {model && options && <ImageSettings models={models} model={model} options={options} custom={draft.custom ?? !model.sizes.includes(options.size)}
            editing={!!reference} disabled={busy} issue={validation}
            selectModel={next=>updateDraft({modelId:next.id,options:next.defaults,custom:!next.sizes.includes(next.defaults.size)})}
            update={(value,custom)=>updateDraft({options:value,modelId:model.id,custom:custom ?? draft.custom})}/>}
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
