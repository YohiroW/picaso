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
  useLayoutEffect(() => {
    const el = dialog.current!;
    el.showModal();
    return () => { el.close(); };
  }, []);
  async function save() {
    try { if (await desktop.export(image.id)) setNotice("Original saved."); }
    catch (e) { setError(String(e)); }
  }
  const { job } = image;
  const editable = ["png","jpeg","webp"].includes(job.parameters.output_format);
  return <dialog ref={dialog} className="image-preview" aria-label="Image preview"
    onCancel={e => { e.preventDefault(); close(); }} onClick={e => { if (e.target === dialog.current) close(); }}>
    <div className="preview-body">
      <div className="preview-actions"><span>{new Date(job.created * 1000).toLocaleDateString()}</span>
        <button onClick={close} autoFocus>Close</button></div>
      {missing ? <p role="alert">This original is missing. Restore it to view or export the image.</p> :
        <img className="preview-original" src={imageURL(image.id)} alt={job.prompt} onError={() => setMissing(true)}/>}
      <div className="preview-info"><p>{job.prompt}</p>
        <small>{job.model} · {job.parameters.size} · {job.parameters.quality} · {job.parameters.output_format.toUpperCase()} · {job.parameters.n ?? 1} requested · {job.parameters.background ?? "auto"} background</small>
        {!editable && <p>Only PNG, JPEG and WebP originals can be used as edit inputs.</p>}
        <div className="actions"><button disabled={busy || missing || !editable} onClick={edit}>Continue editing</button>
          <button disabled={missing} onClick={save}>Save original…</button></div>
        {error && <p role="alert">{error}</p>}{notice && <p role="status">{notice}</p>}
      </div>
    </div>
  </dialog>;
}
