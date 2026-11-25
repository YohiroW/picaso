import { Label } from "../../components/ui/label";
import type { ImageOptions, Model } from "../../lib/desktop";

const sizes: Record<string, string> = {
  auto: "Auto",
  "1024x1024": "Square · 1024 × 1024",
  "1024x1536": "Portrait · 1024 × 1536",
  "1536x1024": "Landscape · 1536 × 1024",
  "2560x1440": "2K · 2560 × 1440",
  "3840x2160": "4K · 3840 × 2160",
};
const names: Record<string, string> = {
  auto: "Auto",
  low: "Low",
  medium: "Medium",
  high: "High",
  xhigh: "Extra high",
  max: "Max",
  png: "PNG",
  jpeg: "JPEG",
  webp: "WebP",
  svg: "SVG",
  gif: "GIF",
  transparent: "Transparent",
  opaque: "Opaque",
};

export interface SettingIssue {
  field: "size" | "quality" | "output_format" | "background" | "n";
  message: string;
}

export function optionsError(
  model: Model,
  options: ImageOptions,
  editing: boolean,
): SettingIssue | undefined {
  if (!model.sizes.includes(options.size)) {
    const [w, h] = options.size.split("x").map(Number);
    if (
      !model.customSize ||
      !/^[1-9]\d*x[1-9]\d*$/.test(options.size) ||
      w % 16 ||
      h % 16 ||
      Math.max(w, h) > 3840 ||
      Math.max(w, h) > 3 * Math.min(w, h) ||
      w * h < 655360 ||
      w * h > 8294400
    )
      return {
        field: "size",
        message:
          "Use multiples of 16 up to 3840 px, an aspect ratio within 3:1, and 655,360–8,294,400 pixels.",
      };
  }
  if (!model.qualities.includes(options.quality))
    return { field: "quality", message: "Choose a quality supported by this model." };
  if (!model.formats.includes(options.output_format))
    return { field: "output_format", message: "Choose a supported output format." };
  if (!model.backgrounds.includes(options.background))
    return { field: "background", message: "Choose a supported background." };
  if (options.background === "transparent" && !["png", "webp"].includes(options.output_format))
    return { field: "background", message: "Transparent backgrounds require PNG or WebP." };
  if (options.output_format === "svg" && (editing || options.n !== 1))
    return { field: "output_format", message: "SVG requires a new generation with one image." };
  if (!Number.isInteger(options.n) || options.n < 1 || options.n > model.maxImages)
    return { field: "n", message: "Choose an available image count." };
  return undefined;
}

interface Props {
  models: Model[];
  model: Model;
  options: ImageOptions;
  custom: boolean;
  editing: boolean;
  disabled: boolean;
  issue?: SettingIssue;
  selectModel: (model: Model) => void;
  update: (newOptions: ImageOptions, custom?: boolean) => void;
}
export function ImageSettings({
  models,
  model,
  options,
  custom,
  editing,
  disabled,
  issue,
  selectModel,
  update,
}: Props) {
  const [width = "", height = ""] = options.size.split("x");
  const counts = Array.from({ length: model.maxImages }, (_, i) => i + 1);
  const svgReason = editing ? "generation only" : options.n !== 1 ? "requires one image" : "";
  const hint = (field: SettingIssue["field"]) =>
    issue?.field === field ? (
      <p className="option-hint" id="settings-error" role="alert">
        {issue.message}
      </p>
    ) : null;
  return (
    <fieldset disabled={disabled} className="generation-settings">
      <section className="panel-section">
        <h2>Model</h2>
        <div className="control-row">
          <Label htmlFor="model">Model</Label>
          <select
            id="model"
            value={model.id}
            onChange={(e) => {
              const next = models.find((m) => m.id === e.target.value);
              if (next) selectModel(next);
            }}
          >
            {models.map((m) => (
              <option key={m.id} value={m.id}>
                {m.displayName}
              </option>
            ))}
          </select>
        </div>
      </section>
      <section className="panel-section">
        <h2>Image settings</h2>
        <div className="control-row">
          <Label htmlFor="image-size">Size &amp; orientation</Label>
          <select
            id="image-size"
            value={custom ? "custom" : options.size}
            onChange={(e) => {
              const isCustom = e.target.value === "custom";
              update(
                {
                  ...options,
                  size: isCustom
                    ? options.size === "auto"
                      ? "1024x1024"
                      : options.size
                    : e.target.value,
                },
                isCustom,
              );
            }}
          >
            {model.sizes.map((s) => (
              <option key={s} value={s}>
                {sizes[s] ?? s}
              </option>
            ))}
            {model.customSize && <option value="custom">Custom…</option>}
          </select>
        </div>
        {custom && (
          <div className="custom-size">
            <label>
              Width (px)
              <input
                type="number"
                aria-invalid={issue?.field === "size"}
                aria-describedby={issue?.field === "size" ? "settings-error" : undefined}
                min="16"
                max="3840"
                step="16"
                value={width}
                onChange={(e) => update({ ...options, size: e.target.value + "x" + height })}
              />
            </label>
            <label>
              Height (px)
              <input
                type="number"
                aria-invalid={issue?.field === "size"}
                aria-describedby={issue?.field === "size" ? "settings-error" : undefined}
                min="16"
                max="3840"
                step="16"
                value={height}
                onChange={(e) => update({ ...options, size: width + "x" + e.target.value })}
              />
            </label>
          </div>
        )}
        {hint("size")}
        <div className="control-row">
          <Label htmlFor="image-quality">Quality</Label>
          <select
            id="image-quality"
            value={options.quality}
            onChange={(e) => update({ ...options, quality: e.target.value })}
          >
            {model.qualities.map((q) => (
              <option key={q} value={q}>
                {names[q]}
              </option>
            ))}
          </select>
        </div>
        {hint("quality")}
        <div className="control-row">
          <Label htmlFor="image-count">Number of images</Label>
          <div className="count-control">
            <input
              type="range"
              id="image-count"
              min="1"
              max={model.maxImages}
              step="1"
              value={options.n}
              disabled={model.maxImages === 1 || options.output_format === "svg"}
              onChange={(e) => update({ ...options, n: Number(e.target.value) })}
            />
            <div className="range-ticks" aria-hidden="true">
              {counts.map((n) => (
                <span key={n} className={n === options.n ? "selected" : undefined}>{n}</span>
              ))}
            </div>
          </div>
        </div>
        {hint("n")}
      </section>
      <details
        className="panel-section advanced"
        open={issue?.field === "output_format" || issue?.field === "background" ? true : undefined}
      >
        <summary>Advanced</summary>
        <div className="control-row">
          <Label htmlFor="output-format">Output format</Label>
          <select
            id="output-format"
            value={options.output_format}
            onChange={(e) => update({ ...options, output_format: e.target.value })}
          >
            {model.formats.map((f) => {
              const reason =
                (f === "svg" && svgReason) ||
                (options.background === "transparent" && !["png", "webp"].includes(f)
                  ? "requires opaque or auto"
                  : "");
              return (
                <option key={f} value={f} disabled={!!reason}>
                  {names[f]}
                  {reason && " · " + reason}
                </option>
              );
            })}
          </select>
        </div>
        {hint("output_format")}
        <div className="control-row">
          <Label htmlFor="background">Background</Label>
          <select
            id="background"
            value={options.background}
            onChange={(e) => update({ ...options, background: e.target.value })}
          >
            {model.backgrounds.map((b) => {
              const blocked =
                b === "transparent" && !["png", "webp"].includes(options.output_format);
              return (
                <option key={b} value={b} disabled={blocked}>
                  {names[b]}
                  {blocked && " · PNG or WebP only"}
                </option>
              );
            })}
          </select>
        </div>
        {hint("background")}
      </details>
    </fieldset>
  );
}
