import { useLayoutEffect, useRef, useState } from "react";
import { desktop, imageURL } from "../../lib/desktop";
import type { GalleryImage } from "./Gallery";

export function ImagePreview({ image, close, edit, busy }: {
  image: GalleryImage; close: () => void; edit: () => void; busy: boolean;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [missing, setMissing] = useState(false);
  const [imageOnly, setImageOnly] = useState(false);
  useLayoutEffect(() => {
    const el = dialog.current!;
    el.showModal();
    return () => { el.close(); };
  }, []);
  async function save() {
    setError("");
    setNotice("");
    try { if (await desktop.export(image.id)) setNotice("Original saved."); }
    catch (e) { setError(String(e)); }
  }
  async function copyPrompt() {
    setError("");
    setNotice("");
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(image.job.prompt);
      } else {
        // Some desktop webviews expose only the user-initiated copy event.
        const write = (event: ClipboardEvent) => {
          event.clipboardData?.setData("text/plain", image.job.prompt);
          event.preventDefault();
        };
        document.addEventListener("copy", write);
        try {
          if (!document.execCommand("copy")) throw new Error();
        } finally {
          document.removeEventListener("copy", write);
        }
      }
      setNotice("Prompt copied.");
    } catch { setError("Couldn’t copy the prompt. Select the text and copy it manually."); }
  }
  function dismiss() {
    if (imageOnly) setImageOnly(false);
    else close();
  }
  const { job } = image;
  const editable = ["png","jpeg","webp"].includes(job.parameters.output_format);
  const toggleLabel = imageOnly ? "Back to image details" : "View image only";
  return <dialog ref={dialog} className={imageOnly ? "image-preview image-only" : "image-preview"} aria-label={imageOnly ? "Image only" : "Image preview"}
    onCancel={e => { e.preventDefault(); dismiss(); }} onClick={e => { if (e.target === dialog.current) dismiss(); }}>
    <div className="preview-body">
      {!imageOnly && <div className="preview-actions"><span>{new Date(job.created * 1000).toLocaleDateString()}</span>
        <button onClick={close}>Close</button></div>}
      {missing ? <p role="alert">This original is missing. Restore it to view or export the image.</p> :
        <div className="preview-frame">
          <img className="preview-original" src={imageURL(image.id)} alt={job.prompt} onError={() => { setMissing(true); setImageOnly(false); }}/>
          <button className="preview-expand" aria-label={toggleLabel} title={toggleLabel}
            onClick={e => { e.currentTarget.focus(); setImageOnly(!imageOnly); }}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
              {imageOnly ? <path d="m6 6 12 12M18 6 6 18"/> : <><circle cx="10" cy="10" r="6.5"/><path d="m15 15 5 5M7 10h6m-3-3v6"/></>}
            </svg>
          </button>
        </div>}
      {!imageOnly && <div className="preview-info"><p>{job.prompt}<button className="copy-prompt" aria-label="Copy prompt" onClick={copyPrompt}>
        {notice === "Prompt copied." ? "Copied" : "Copy"}</button></p>
        <small>{job.model} · {job.parameters.size} · {job.parameters.quality} · {job.parameters.output_format.toUpperCase()} · {job.parameters.n ?? 1} requested · {job.parameters.background ?? "auto"} background</small>
        {!editable && <p>Only PNG, JPEG and WebP originals can be used as edit inputs.</p>}
        <div className="actions"><button disabled={busy || missing || !editable} onClick={edit}>Continue editing</button>
          <button disabled={missing} onClick={save}>Save original…</button></div>
        {error && <p role="alert">{error}</p>}{notice && <p className={notice === "Prompt copied." ? "sr-only" : undefined} role="status">{notice}</p>}
      </div>}
    </div>
  </dialog>;
}
