import { useLayoutEffect, useRef, useState } from "react";
import { imageURL } from "../../lib/desktop";
import type { GalleryImage } from "./Gallery";

export function DeleteImageDialog({ image, remove, close }: {
  image: GalleryImage; remove: (id: string) => Promise<void>; close: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState("");
  useLayoutEffect(() => {
    const el = dialog.current!;
    el.showModal();
    return () => el.close();
  }, []);

  async function confirm() {
    setDeleting(true);
    setError("");
    try {
      await remove(image.id);
      close();
    } catch (e) {
      setError(String(e));
      setDeleting(false);
    }
  }
  return <dialog ref={dialog} className="image-preview delete-image" aria-labelledby="delete-title" aria-describedby="delete-description"
    onCancel={e => { e.preventDefault(); if (!deleting) close(); }}>
    <div className="delete-summary">
      <img src={imageURL(image.id)} alt=""/>
      <div><h2 id="delete-title">Delete this image?</h2>
        <p id="delete-description">This permanently deletes the original from your library.</p></div>
    </div>
    {error && <p role="alert">{error}</p>}
    <div className="actions"><button autoFocus disabled={deleting} onClick={close}>Cancel</button>
      <button className="primary" disabled={deleting} onClick={confirm}>{deleting ? "Deleting…" : "Delete image"}</button></div>
  </dialog>;
}
