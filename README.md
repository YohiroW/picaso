# Picaso

**English** · [简体中文](README.zh-CN.md)

<img src="src-tauri/icons/icon.png" width="80" alt="" align="right">

Picaso is an open-source image workbench for Windows and Apple silicon Macs. Use your own OpenAI API key to generate an image, then pick a result and describe what you want to change. Each edit creates a new image and keeps the original. Picaso saves your images, prompts, and settings on your computer so you can revisit your work and export originals offline.

Use the desktop app to compare results and try another edit, or use the standalone `picaso` CLI to generate images in batches and automate edits.

## Quick start

To run the desktop app from source, install Node.js 22.12 or later with npm and the stable Rust toolchain. On Windows, install Microsoft C++ Build Tools and WebView2. On an Apple silicon Mac, use macOS 14 or later and install the Xcode Command Line Tools. See the [Tauri setup guide](https://v2.tauri.app/start/prerequisites/) for platform-specific instructions.

```sh
git clone https://github.com/YohiroW/picaso.git
cd picaso
npm ci
npm run tauri -- dev
```

### Connect to OpenAI

- **Windows** — Save your key in a user environment variable named `OPENAI_API_KEY`. Open a new terminal, start Picaso, and select **Use environment key** in Settings.
- **macOS** — Open **Settings → API key**, enter your key, and select **Save and use key** to store it in your login keychain. You can also start Picaso from a terminal with `OPENAI_API_KEY` set and select **Use environment key**.

### Generate and edit images

1. In **Prompt**, describe a scene such as “A white lighthouse in the morning mist, in landscape orientation.” Choose a model, size, quality, and image count, then generate up to four images. Open **Advanced** to change the output format or background.
2. Open an image you like, select **Continue editing**, and enter “Keep the composition and change the time of day to dusk.” Select **Apply edit**. To start from your own PNG, JPEG, or WebP image, use **Add image** beside the prompt or paste the image into the prompt box.
3. Select **Save original…** to export the image. Next time you open Picaso, **History** shows your images together with their prompts and generation settings.

With the prompt box focused, press `Ctrl+Enter` on Windows or `⌘+Enter` on macOS to submit. To delete an original, right-click it in **Recent creations**, select **Delete image…**, and confirm. Picaso keeps the other images in the batch and any edits you have already made.

If a job shows **Result unconfirmed**, OpenAI may already have processed the request. Check your job history and API account usage before submitting it again.

## Command line

Install the CLI from the repository root. The `picaso models` command lists available models, image sizes, and output formats offline.

```sh
cargo install --locked --path cli
picaso models
```

In a terminal with `OPENAI_API_KEY` set, generate two lighthouse images, export them to `./images`, and save the job details to `generated.json`.

```sh
picaso generate --prompt "A white lighthouse in the morning mist" --count 2 --out-dir ./images --json > generated.json
```

On macOS, use `jq` to get the second image’s ID, edit that image, and export the result as a PNG.

```sh
image_id=$(jq -er '.images[1].imageId' generated.json)
picaso edit --image-id "$image_id" --prompt "Keep the composition and change the time of day to dusk" --output-format png > edited.json
picaso export --image-id "$(jq -er '.images[0].imageId' edited.json)" --out ./lighthouse.png
```

- Look up a saved job with `picaso show <JOB_ID>`. The result identifies the job by `jobId` and each image by `images[].imageId`.
- For multiline prompts, use `--prompt-file <file>` or pipe text to standard input. Run `picaso generate --help` for size, quality, and background options.
- Use `--workspace <directory>` to choose a library for a script. If you repeat a successful job in the same workspace with the same `--request-id <UUID>` and inputs, the CLI reuses the saved images.

<details>
<summary>How should a script handle a failed job?</summary>

On failure, the CLI exits with a nonzero status and writes a JSON object to standard output. Read its `error.code` field to decide how to proceed.

| Error code | What to do |
| --- | --- |
| `workspace_busy` | Wait for the process using the library to exit, then retry. |
| `request_conflict` | Check whether the prompt, input image, or settings changed. Use a new request ID to submit a different job. |
| `unconfirmed` | Look up the job with `picaso show` and check your API account usage before submitting again. |
| `export_failed` | If generation succeeded, retry the export using the image IDs in `error.images`. Choose a destination outside the library with an existing parent directory. |

</details>

## Local data and models

| Library | Default location |
| --- | --- |
| Windows desktop app | `%LOCALAPPDATA%\app.picaso.desktop\` |
| macOS desktop app | `~/Library/Application Support/app.picaso.desktop/` |
| macOS CLI | `~/Library/Application Support/app.picaso.cli/` |

Each library stores jobs and image references in `library.sqlite3` and original files in `originals/`. To back up a library, quit the app using it and copy the entire directory. macOS manages keys stored in the login keychain separately.

To change model defaults, create a `models.json` file in the root of the library you use. For an existing model, include its ID and the fields you want to override. The example below sets Sunburst to produce transparent PNGs by default. To add a new model, start with a complete entry from the [bundled model catalog](core/models.json).

```json
{
  "models": [
    {
      "id": "gpt-image-2.6-sunburst",
      "defaults": { "background": "transparent", "output_format": "png" }
    }
  ]
}
```

## Development and testing

The desktop app uses React and TypeScript for its [interface](src/) and Tauri 2 for [native integration](src-tauri/). The desktop app and the [CLI](cli/) share a [Rust core](core/) that handles model configuration, image jobs, and library storage.

After running `npm ci`, you can start the desktop app with an offline test provider. It uses a separate library and local test images. If Settings opens on first launch, select **Back to workspace** to try generating and editing images without an API key.

```sh
npm run tauri -- dev --features fixture-provider
```

Run the frontend, core, CLI, and desktop tests from the repository root.

```sh
npm test
cargo test --workspace
cargo test -p picaso-cli --features fixture-provider
cargo test --manifest-path src-tauri/Cargo.toml
```

| Build platform | Command | Output |
| --- | --- | --- |
| Windows | `npm run tauri -- build` | NSIS installer |
| Apple silicon Mac | `npm run build:mac` | App bundle and DMG with ad hoc signing |

## License

Picaso is available under the [MIT License](LICENSE).
