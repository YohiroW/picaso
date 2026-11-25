import { useEffect, useLayoutEffect, useRef, useState, type ClipboardEvent, type FormEvent } from "react";
import { ImageSettings, optionsError } from "./ImageSettings";
import { Label } from "../../components/ui/label";
import { SettingsIcon } from "../../components/SettingsIcon";
import { AppBar } from "../../components/AppBar";
import { desktop, imageURL, type CredentialStatus, type ImageOptions, type Job, type Model } from "../../lib/desktop";
import { Gallery, type GalleryImage } from "./Gallery";
import { ImagePreview } from "./ImagePreview";


type Reference = { id: string; name?: string };
type Draft = { prompt: string; reference?: Reference; modelId?: string; options?: ImageOptions; custom?: boolean };
const emptyDraft: Draft = { prompt: "" };

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
  return <svg width="19" height="19" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.65" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    <path d="M3 6v5h5M3.6 10a8.5 8.5 0 1 1 .9 6M12 7v5l3 2"/>
  </svg>;
}

export function Workspace({ status, openSettings }: { status: CredentialStatus; openSettings: (trigger: HTMLButtonElement) => void }) {
  const [history, setHistory] = useState(false);
  const [snapshot, setSnapshot] = useState<{ history: boolean; jobs: Job[] }>();
  const [pending, setPending] = useState<Job>();
  const [settledId, setSettledId] = useState<string>();
  const [models, setModels] = useState<Model[]>([]);
  const [catalogError,setCatalogError] = useState("");
  const [revision, refresh] = useState(0);
  const [loadError, setLoadError] = useState("");
  const [preview, setPreview] = useState<GalleryImage>();
  const [draftKey, setDraftKey] = useState("new");
  const [unavailableReference, setUnavailableReference] = useState<string>();
  const [drafts, setDrafts] = useState<Record<string, Draft>>({ new: emptyDraft });
  const previousDrafts = useRef(drafts);
  const fileInput = useRef<HTMLInputElement>(null);
  const [importing, setImporting] = useState(false);
  const [busy, setBusy] = useState(false);
  const inFlight = useRef<string | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [mobileEditor, setMobileEditor] = useState(false);
  const promptField = useRef<HTMLTextAreaElement>(null);
  const [focusRequest, setFocusRequest] = useState(0);
  const draft = drafts[draftKey] ?? emptyDraft;
  const reference = draft.reference?.id;
  const model = draft.modelId ? models.find(m=>m.id===draft.modelId) : models[0];
  const options = draft.options ?? model?.defaults;
  const validation = model && options ? optionsError(model,options,!!reference) : undefined;
  const available = model?.provider === "fixture" || !!(status.selected && status[status.selected]);
  const missingReference = !!reference && reference === unavailableReference;
  const canSubmit = model && options && !validation && available && draft.prompt.trim() && (!reference || model?.canEdit) && !missingReference && !busy && !importing;

  const storedJobs = snapshot?.jobs ?? [];
  const jobs = pending && !storedJobs.some(job => job.id === pending.id) ? [pending, ...storedJobs] : storedJobs;
  const loading = history ? !snapshot?.history : !snapshot && !pending;

  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function load() {
      try {
        const results = await desktop.history(!history);
        if (!active) return;
        setSnapshot({ history, jobs: results });
        setPending(current => current?.id === settledId ? undefined : current);
        // A reloaded webview may see a running task without owning its submit promise.
        if (results.some(job => job.status === "running" && job.id !== inFlight.current)) {
          timer = setTimeout(load, 1500);
        }
      } catch (e) {
        if (active) setLoadError(String(e));
      }
    }
    setLoadError("");
    void load();
    return () => { active = false; clearTimeout(timer); };
  }, [history, revision, settledId]);

  useEffect(() => {
    let active=true;
    desktop.models().then(result=>{if(active){setModels(result);setCatalogError("");}})
      .catch(e=>{if(active){setCatalogError(String(e));setModels([]);}});
    return ()=>{active=false;};
  }, [revision]);

  useLayoutEffect(() => {
    if (focusRequest) promptField.current?.focus();
  }, [focusRequest]);

  useEffect(() => {
    const activeIds = new Set(Object.values(drafts).map(entry => entry.reference?.id));
    for (const { reference } of Object.values(previousDrafts.current)) {
      if (reference?.name && !activeIds.has(reference.id)) {
        // The library keeps inputs already used by a job and retries cleanup at startup.
        void desktop.discardReference(reference.id).catch(err => console.warn("Reference cleanup failed", err));
      }
    }
    previousDrafts.current = drafts;
  }, [drafts]);

  function focusEditor() {
    setMobileEditor(true);
    setFocusRequest(n => n + 1);
  }
  function updateDraft(changes: Partial<Draft>) {
    setDrafts(prev => ({ ...prev, [draftKey]: { ...(prev[draftKey] ?? emptyDraft), ...changes } }));
  }
  async function attach(file: File) {
    if (busy || importing) return;
    setImporting(true);
    setError("");
    setNotice("");
    promptField.current?.focus();
    try {
      const id = await desktop.importReference(file);
      updateDraft({ reference: { id, name: file.name || "Pasted image" } });
      setUnavailableReference(undefined);
      if (document.activeElement === fileInput.current || document.activeElement === document.body) {
        promptField.current?.focus();
      }
    } catch (err) { setError(err instanceof Error ? err.message : String(err)); }
    finally { setImporting(false); }
  }
  function pasteReference(event: ClipboardEvent) {
    const data = event.clipboardData;
    const files = data.files.length ? Array.from(data.files) :
      Array.from(data.items).flatMap(item => item.kind === "file" ? item.getAsFile() ?? [] : []);
    const images = files.filter(file => file.type.startsWith("image/") || /\.(png|jpe?g|webp)$/i.test(file.name));
    if (!images.length) return;
    event.preventDefault();
    if (busy || importing) return;
    if (images.length > 1) {
      setError("Paste one reference image at a time.");
      return;
    }
    void attach(images[0]);
  }
  function removeReference() {
    updateDraft({ reference: undefined });
    setUnavailableReference(undefined);
    setError("");
    setNotice("");
    focusEditor();
  }
  function newImage() {
    setDraftKey("new");
    setPreview(undefined);
    setError("");
    setNotice("");
    focusEditor();
  }
  function edit(image: GalleryImage) {
    setDrafts(prev => ({ ...prev, [image.id]: { ...(prev[image.id] ?? draft), reference: { id: image.id } } }));
    setDraftKey(image.id);
    setUnavailableReference(undefined);
    setPreview(undefined);
    setError("");
    setNotice("");
    focusEditor();
  }
  function reuse(job: Job) {
    const options = { ...job.parameters, n: job.parameters.n ?? 1, background: job.parameters.background ?? "auto" };
    const key = job.inputId ?? "new";
    setDrafts(prev => ({ ...prev, [key]: { prompt: job.prompt, modelId: job.model, options, reference: job.inputId ? { id: job.inputId } : undefined } }));
    setDraftKey(key);
    setUnavailableReference(undefined);
    setError(job.message ?? (job.status === "unconfirmed" ? "Submitting again may incur another charge." : ""));
    setNotice("");
    focusEditor();
  }
  async function deleteImage(id: string) {
    await desktop.deleteImage(id);
    if (id === reference) setUnavailableReference(id);
    refresh(n => n + 1);
  }
  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!model || !options || !canSubmit || inFlight.current) return;
    const requestId = crypto.randomUUID();
    inFlight.current = requestId;
    setPending({
      id: requestId, model: model.id, provider: model.provider, prompt: draft.prompt.trim(),
      parameters: options, inputId: reference ?? null, images: [], status: "running",
      message: null, created: Math.floor(Date.now() / 1000),
    });
    setHistory(false);
    setBusy(true);
    setError("");
    setNotice(`Generating ${options.n} ${options.n === 1 ? "image" : "images"}…`);
    try {
      await desktop.submit({ requestId, model: model.id, prompt: draft.prompt, inputId: reference, options });
      setDrafts(prev => {
        const next = { ...prev };
        if (draftKey !== "new") delete next[draftKey];
        else next.new = { ...draft, reference: undefined, prompt: "", modelId:model.id, options };
        return next;
      });
      setDraftKey("new");
      setNotice("Saved to your gallery.");
    } catch (e) { setError(String(e)); setNotice(""); }
    finally {
      inFlight.current = null;
      setBusy(false);
      setSettledId(requestId);
    }
  }

  return <>
    <AppBar><nav aria-label="Workspace">
      <button className="toolbar-icon" aria-label="History" aria-pressed={history}
        title={history ? "Back to studio" : "History"} onClick={() => { setHistory(v => !v); setPreview(undefined); }}>
        <HistoryIcon />
      </button>
      <button id="workspace-settings" className="toolbar-icon" aria-label="Settings" title="Settings" onClick={e => openSettings(e.currentTarget)}><SettingsIcon/></button>
    </nav></AppBar>
    <main className="studio">
      <aside className="control-panel" aria-label="Image controls">
        <button className="mobile-toggle" aria-expanded={mobileEditor} aria-controls="image-form" onClick={() => setMobileEditor(v => !v)}>Image controls <span aria-hidden="true">{mobileEditor ? "−" : "+"}</span></button>
        <form id="image-form" className={mobileEditor ? "editor expanded" : "editor"} onSubmit={submit}>
          <section className="panel-section prompt-section">
            <Label className="section-title" htmlFor="prompt">{reference ? "Edit instructions" : "Prompt"}</Label>
            <div className="prompt-box" onPaste={pasteReference}>
              <div aria-live="polite" aria-atomic="true">{reference && <div className="reference">
                {!missingReference && <img key={reference} src={imageURL(reference)} alt="Reference image" onError={() => setUnavailableReference(reference)}/>}
                <span title={draft.reference?.name}>{missingReference ? "Reference image unavailable" : draft.reference?.name ?? "Reference image"}</span>
                <button type="button" aria-label="Remove reference" disabled={busy || importing} onClick={removeReference}>×</button></div>}</div>
              <textarea id="prompt" ref={promptField} value={draft.prompt} disabled={busy} onChange={e => updateDraft({prompt:e.target.value})}
                placeholder={reference ? "Describe what to change…" : "Describe what you want to see…"}
                onKeyDown={e => {
                  if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && !e.nativeEvent.isComposing && e.nativeEvent.keyCode !== 229) {
                    e.preventDefault();
                    e.currentTarget.form?.requestSubmit();
                  }
                }}/>
              <div className="prompt-footer">
                <input type="file" ref={fileInput} hidden aria-label="Reference image file" accept="image/png,image/jpeg,image/webp" onChange={e => {
                  const file = e.currentTarget.files?.[0];
                  e.currentTarget.value = "";
                  if (file) void attach(file);
                }}/>
                <button type="button" className="attach-image" disabled={busy || importing} onClick={() => fileInput.current?.click()}
                  aria-label={importing ? "Adding image" : reference ? "Replace image" : "Add image"} aria-busy={importing}
                  title={reference ? "Replace reference image (or paste with ⌘V)" : "Add reference image (or paste with ⌘V)"}>
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="m21 11-9.2 9.2a6 6 0 0 1-8.5-8.5l10-10a4 4 0 0 1 5.7 5.7l-10 10a2 2 0 0 1-2.8-2.8l9.2-9.2"/></svg>
                </button>
                <kbd title="Command + Return">⌘ ↩</kbd>
                <button type="submit" className="primary submit" disabled={!canSubmit}>{busy ? "Creating…" : reference ? (options && options.n>1 ? `Apply edit · ${options.n} images` : "Apply edit") : (options && options.n>1 ? `Generate ${options.n} images` : "Generate image")}<span aria-hidden="true">↑</span></button>
              </div>
            </div>
            {model && !available && <p className="feedback">Choose a key to create images. <button type="button" className="link" onClick={e => openSettings(e.currentTarget)}>Settings</button></p>}
            {reference && model && !model.canEdit && <p role="alert">This model cannot edit a reference. Remove it to generate a new image.</p>}
            {error && <p role="alert" className="feedback">{error}</p>}
            {!busy && notice && <p className="feedback">{notice}</p>}
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
        <div role="status" className="sr-only">{notice}</div>
        {loadError ? <p role="alert">{loadError} <button onClick={() => refresh(n => n + 1)}>Try again</button></p> :
          loading ? <p role="status">Loading images…</p> :
          history ? <History jobs={jobs} open={setPreview}/> :
          <Gallery jobs={jobs} open={setPreview} create={newImage} reuse={reuse} busy={busy || importing} revealJobId={pending?.id} deleteImage={deleteImage}/>}
      </section>
    </main>
    {preview && <ImagePreview key={preview.id} image={preview} close={() => setPreview(undefined)} edit={() => edit(preview)} busy={busy || importing}/>}
  </>;
}
