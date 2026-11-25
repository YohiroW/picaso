import { useLayoutEffect, useRef, useState } from "react";
import { imageURL, type Job } from "../../lib/desktop";

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

function Thumbnail({ image, onRatio }: { image: GalleryImage; onRatio: (ratio: number) => void }) {
  const [missing, setMissing] = useState(false);
  if (missing) return <span className="missing-image">Original unavailable</span>;
  return <>
    <img src={imageURL(image.id)} alt="" loading="lazy" decoding="async"
      onLoad={e => onRatio(e.currentTarget.naturalWidth / e.currentTarget.naturalHeight)}
      onError={() => setMissing(true)} />
    <span className="tile-prompt" aria-hidden="true">{image.job.prompt}</span>
  </>;
}

export function Gallery({ images, open, create, busy }: {
  images: GalleryImage[]; open: (image: GalleryImage) => void; create: () => void; busy: boolean;
}) {
  const container = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(640);
  const [ratios, setRatios] = useState<Record<string, number>>({});
  useLayoutEffect(() => {
    const el = container.current;
    if (!el) return;
    const measure = () => { if (el.clientWidth) setWidth(el.clientWidth); };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  const layout = arrange([...images.map(im => ratios[im.id] ?? ratioOf(im.job.parameters.size)), 1], width);
  return <div ref={container} className="masonry" style={{ height: layout.height }}>
    {images.map((im, i) => <button key={im.id} className="image-tile" style={layout.tiles[i]}
      aria-label={`View image ${i + 1}: ${im.job.prompt}`} onClick={() => open(im)}>
      <Thumbnail image={im} onRatio={ratio => {
        if (ratio > 0 && ratio !== (ratios[im.id] ?? ratioOf(im.job.parameters.size))) {
          setRatios(previous => ({...previous, [im.id]: ratio}));
        }
      }}/>
    </button>)}
    <button className="new-tile" style={layout.tiles[images.length]} onClick={create} disabled={busy} aria-label="New image" title="New image">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg>
      {images.length === 0 && <span>New image</span>}
    </button>
  </div>;
}
