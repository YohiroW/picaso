import { useLayoutEffect, useRef, useState, type CSSProperties } from "react";
import * as ContextMenu from "@radix-ui/react-context-menu";
import { imageURL, type Job } from "../../lib/desktop";
import { DeleteImageDialog } from "./DeleteImageDialog";

export interface GalleryImage { id: string; job: Job; index: number }

function ratioOf(size: string) {
  const [w, h] = size.split("x").map(Number);
  return w > 0 && h > 0 ? w / h : 1;
}

// Place in the shortest column in reading order. Keep buttons in that same DOM order.
function arrange(ratios: number[], width: number) {
  const gap = 10;
  const minWidth = width <= 480 ? 154 : 208;
  const columns = Math.max(1, Math.min(6, Math.floor((width + gap) / (minWidth + gap))));
  const tileWidth = (width - (columns - 1) * gap) / columns;
  const bottoms = Array<number>(columns).fill(0);
  const tiles = ratios.map(ratio => {
    const column = bottoms.indexOf(Math.min(...bottoms));
    const height = tileWidth / ratio;
    const top = bottoms[column];
    bottoms[column] += height + gap;
    return { left: column * (tileWidth + gap), top, width: tileWidth, height };
  });
  return { height: Math.max(...bottoms) - gap, tiles };
}

interface Slot { job: Job; index: number; imageId?: string }

function recentSlots(jobs: Job[]): Slot[] {
  return jobs.slice().reverse().flatMap(job => {
    if (job.status === "succeeded") {
      return job.images.map((imageId, index) => ({ job, index, imageId }));
    }
    return Array.from({ length: job.parameters.n ?? 1 }, (_, index) => ({ job, index }));
  }).slice(-8);
}

function TileImage({ image, position, open, onRatio }: {
  image: GalleryImage; position: number; open: (image: GalleryImage) => void; onRatio: (ratio: number) => void;
}) {
  const [thumbnail, setThumbnail] = useState<"loading" | "ready" | "missing">("loading");
  if (thumbnail === "missing") return <div tabIndex={0} className="tile-state">Original unavailable</div>;
  return <>
    <button className="tile-preview" disabled={thumbnail !== "ready"} aria-haspopup="menu"
      aria-label={`View image ${position}: ${image.job.prompt}`} onClick={() => open(image)}>
      <img src={imageURL(image.id)} alt="" decoding="async" onError={() => setThumbnail("missing")}
        onLoad={e => {
          onRatio(e.currentTarget.naturalWidth / e.currentTarget.naturalHeight);
          setThumbnail("ready");
        }}/>
      {thumbnail === "ready" && <span className="tile-prompt" aria-hidden="true">{image.job.prompt}</span>}
    </button>
    {thumbnail === "loading" && <div className="tile-state"><span className="tile-spinner" aria-hidden="true"/>Loading image…</div>}
  </>;
}

function Tile({ slot: { job, index, imageId }, position, style, open, reuse, busy, onRatio, confirmDelete }: {
  slot: Slot; position: number; style: CSSProperties; open: (image: GalleryImage) => void;
  reuse: (job: Job) => void; busy: boolean; onRatio: (ratio: number) => void; confirmDelete: (image: GalleryImage) => void;
}) {
  const image = imageId ? { id: imageId, job, index } : undefined;
  const generating = job.status === "running";
  return <ContextMenu.Root modal={false}>
    <ContextMenu.Trigger asChild disabled={!image}>
      <div className="image-tile" style={style} data-job-id={job.id} onKeyDown={e => {
        if (image && (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10"))) {
          e.preventDefault();
          const { left, top } = e.currentTarget.getBoundingClientRect();
          e.currentTarget.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: left + 8, clientY: top + 8 }));
        }
      }}>
        {image && <TileImage key={image.id} image={image} position={position} open={open} onRatio={onRatio}/>}
        {generating && <div className="tile-state"><span className="tile-spinner" aria-hidden="true"/>Generating…</div>}
        {!generating && !imageId && <div className="tile-state tile-error" title={job.prompt}>
          <span>{job.status === "unconfirmed" ? "Result unconfirmed" : "Generation failed"}</span>
          <button className="link" disabled={busy} onClick={() => reuse(job)}
            aria-label={`Edit prompt: ${job.prompt}`}>Edit prompt</button>
        </div>}
      </div>
    </ContextMenu.Trigger>
    {image && <ContextMenu.Portal><ContextMenu.Content className="creation-menu" collisionPadding={6} aria-label="Image actions">
      <ContextMenu.Item onSelect={() => confirmDelete(image)}>Delete image…</ContextMenu.Item>
    </ContextMenu.Content></ContextMenu.Portal>}
  </ContextMenu.Root>;
}

export function Gallery({ jobs, open, create, reuse, busy, revealJobId, deleteImage }: {
  jobs: Job[]; reuse: (job: Job) => void; revealJobId?: string; open: (image: GalleryImage) => void; create: () => void; busy: boolean;
  deleteImage: (id: string) => Promise<void>;
}) {
  const container = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(640);
  const [ratios, setRatios] = useState<Record<string, number>>({});
  const [deleteTarget, setDeleteTarget] = useState<GalleryImage>();
  useLayoutEffect(() => {
    const el = container.current;
    if (!el) return;
    const measure = () => { if (el.clientWidth) setWidth(el.clientWidth); };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  useLayoutEffect(() => {
    if (revealJobId) {
      container.current?.querySelector<HTMLElement>(`[data-job-id="${revealJobId}"]`)?.scrollIntoView?.({ block: "nearest" });
    }
  }, [revealJobId, width]);

  const slots = recentSlots(jobs);
  const layout = arrange([...slots.map(({ job, imageId }) =>
    (imageId && ratios[imageId]) || ratioOf(job.parameters.size)), 1], width);
  return <div ref={container} className="masonry" style={{ height: layout.height }}>
    {slots.map((slot, i) => <Tile key={`${slot.job.id}-${slot.index}`} slot={slot} position={i + 1}
      style={layout.tiles[i]} open={open} reuse={reuse} confirmDelete={setDeleteTarget} busy={busy} onRatio={ratio => {
        const id = slot.imageId;
        if (id && ratio > 0 && ratio !== (ratios[id] ?? ratioOf(slot.job.parameters.size))) {
          setRatios(previous => ({...previous, [id]: ratio}));
        }
      }}/>)}
    <button className="new-tile" style={layout.tiles[slots.length]} onClick={create} disabled={busy} aria-label="New image" title="New image">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg>
      {slots.length === 0 && <span>New image</span>}
    </button>
    {deleteTarget && <DeleteImageDialog image={deleteTarget} close={() => setDeleteTarget(undefined)} remove={deleteImage}/>}
  </div>;
}
