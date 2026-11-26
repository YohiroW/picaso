# Picaso

Picaso 是用于 Apple Silicon Mac 的本地图像工作台。使用自己的 OpenAI API 账户生成图片，重开后可从历史中选择一张原图继续修改。当前交付为本地开发版。

## 安装与使用

1. 打开 [0.1.6 开发版 DMG](src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Picaso_0.1.6_aarch64.dmg)，把 Picaso 拖入 Applications。最低系统版本为 macOS 14。开发版采用 ad hoc 签名。
2. 首次打开时选择密钥来源。如果窗口检测到 `OPENAI_API_KEY`，可以点击 **Use environment key**。如果只在终端中设置了环境变量，可以从该终端启动 `~/Applications/Picaso.app/Contents/MacOS/picaso`（安装在系统 Applications 时去掉 `~`）。也可以从右上角齿轮打开 **Settings → API key**，输入自己的 API Key，点击 **Save and use key** 存入当前用户的 macOS 钥匙串。可在此更换、切换或删除密钥。API 额度由 OpenAI API 账户提供，与 ChatGPT 订阅分别计费。
3. 在左栏 **Prompt** 描述图片，选择模型、画幅、质量和张数（1–4）。自定义画幅可填写像素宽高；**Advanced** 可选择输出格式与背景，点击 **Generate image**。右侧最多显示最近 8 个图片位置。提交后，每张图会按所选画幅显示 **Generating…** 占位，缩略图就绪后在原位显示。失败或结果未确认时可点 **Edit prompt** 恢复提示词、参数和参考图，再决定是否提交。末尾 **＋** 可回到新建草稿。右上角 **History** 可查看所有任务，包括失败或结果未确认的任务。点击任意一张作品可预览原图。点击 **Continue editing**，在 **Edit instructions** 中输入修改要求，再点 **Apply edit** 或按 ⌘↩。新结果和原图都会保留。
4. **Save original…** 可将选中的原始文件 保存到自行选择的位置。在作品详情中点击提示词末尾的双层方框图标 可复制全文。点击图片右上角的放大镜可单独看图，按 Esc 返回详情。右键点击 **Recent creations** 中的图片，选择 **Delete image…** 并确认后，会从图库和 History 永久删除这张原图。同批其他图片和已生成的编辑结果会保留。查看或导出旧图无需联网，首次使用也可以点击 **Back to workspace**。

只有明确提交“生成”或“编辑”会向 OpenAI 请求图片。超时或意外退出后，如果任务显示 **unconfirmed**，上游可能已经计费，请先检查历史和账户用量再决定是否重新提交。

## macOS 命令行

在仓库根目录运行 `cargo install --locked --path cli`，即可将独立的 `picaso` 安装到 Cargo 的 bin 目录。CLI 在 Apple Silicon Mac 上可直接运行。生成和编辑从当前进程的 `OPENAI_API_KEY` 取密钥；查看模型、读取任务和导出原图都可离线使用。

下面的例子使用 `jq`，从管道读取多行提示词，生成两张图，继续编辑第二张，再将编辑结果导出为 PNG。

```sh
set -e
picaso generate --count 2 --output-format png --out-dir ./images --json > generated.json <<'PROMPT'
一座白色的灯塔，俯瞰海面。
清晨有雾，使用柔和的自然光。
PROMPT

image_id=$(jq -er '.images[1].imageId' generated.json)
printf '%s\n' '保留构图，把时间改成黄昏。' |
  picaso edit --image-id "$image_id" --output-format png --json > edited.json
picaso export --image-id "$(jq -er '.images[0].imageId' edited.json)" --out ./final.png
```

### 子命令与全局选项

| 命令 | 用途 |
| --- | --- |
| `picaso models` | 列出工作区的模型 ID、可用参数和默认值，包括 `models.json` 中的覆盖项。 |
| `picaso generate` | 根据提示词创建一个任务，生成一张或多张图片。 |
| `picaso edit --image-id <UUID>` | 用提示词修改当前工作区内的一张 PNG、JPEG 或 WebP 原图。图片 ID 取自上一步返回的 `images[].imageId`。 |
| `picaso show <JOB_ID>` | 按任务 UUID 查询状态、实际参数和现有原图。位置参数取自生成或编辑结果的 `jobId`。 |
| `picaso export --image-id <UUID> --out <文件>` | 按图片 ID 导出原始字节。`--out` 必填，父目录须已存在且位于工作区外。遇到内容不同的已有文件会报错。 |

`--workspace <目录>` 和 `--json` 可放在子命令前后。默认工作区是 `~/Library/Application Support/app.picaso.cli/`，可与桌面端同时使用。同一流程的各步应使用同一工作区。`--json` 将默认缩进的 JSON 压成单行，方便管道读取。

用 `picaso --help`（`-h`）查看命令总览，`picaso help <子命令>` 或 `picaso <子命令> --help` 查看该命令的参数，`picaso --version`（`-V`）查看版本。

### 生成与编辑参数

`generate` 和 `edit` 共用下表中的选项。画幅、质量、张数、格式和背景均采用所选模型的默认值，显式传入的选项覆盖对应字段。

| 参数 | 取值与行为 |
| --- | --- |
| `--prompt <文本>` | 直接传入提示词，与 `--prompt-file` 互斥。两者都省略时，从管道读取提示词。 |
| `--prompt-file <文件或 ->` | 读取 UTF-8 文本文件。传 `-` 时从 stdin 读到 EOF，支持多行。 |
| `--model <ID>` | 使用 `picaso models` 返回的模型 ID，默认使用列表中的第一个模型。 |
| `--size <尺寸>` | 例如 `1024x1024`、`1536x1024` 或 `auto`。可用预设和自定义尺寸能力见模型的 `sizes` 和 `customSize`。 |
| `--quality <质量>` | 从模型的 `qualities` 取值，例如 `low`、`medium` 或 `high`。 |
| `--count <张数>`（别名 `--n`） | 从 1 到模型的 `maxImages`，最多 4 张。 |
| `--output-format <格式>` | 从模型支持的 `png`、`jpeg`、`webp`、`svg`、`gif` 中选择。SVG 仅用于单张生成。 |
| `--background <背景>` | 从模型支持的 `auto`、`opaque`、`transparent` 中选择。透明背景需要 PNG 或 WebP。 |
| `--out-dir <目录>` | 将每张结果按图片 ID 命名，导出到工作区外的目录。目录可自动创建。省略时返回工作区原图路径。 |
| `--request-id <UUID>` | 指定任务 ID，默认自动生成。工作流重跑时复用同一 UUID，以便取回已完成的结果。 |
| `--key-source env` | 当前支持的唯一值，也是默认值。密钥取自调用进程的 `OPENAI_API_KEY`。 |

### 自动化结果与重跑

生成和编辑完成后向 stdout 写入一个 JSON 对象，包含 `jobId`、`status`、实际参数和按生成顺序排列的 `images`。每张图都有 `imageId`、`format` 和绝对路径 `path`。

跨 CI 运行重用任务时，保存整个工作区和每一步的请求 UUID。同一工作区中，使用相同 ID、提示词、输入图和参数重跑已成功的任务会复用原图。命令失败时以非零状态退出，脚本可读取 stdout 中的 `error.code` 决定下一步。

| 错误码 | 处理方式 |
| --- | --- |
| `workspace_busy` | 图库正由其他进程持锁，等待该进程退出后再执行。 |
| `request_conflict` | 该 UUID 已用于其他提交内容。若要创建新任务，请使用新的 UUID。 |
| `unconfirmed` | 上游可能已处理请求。先查询任务和账户用量，再决定是否重新提交。 |
| `export_failed` | 若生成或编辑已完成，使用 `error.images` 返回的图片 ID 单独运行 `export`。也可通过 `error.jobId` 查询原任务。 |

## 数据与升级

图库位于 `~/Library/Application Support/app.picaso.desktop/`，其中 `library.sqlite3` 存放任务与图片引用，`originals/` 保存原图，`credential-source.json` 只保存选用的来源。备份时先退出应用，再复制整个目录。手动密钥由系统钥匙串单独管理。升级时退出旧版并替换应用包，保留数据目录。

## 模型配置

模型下拉框和 Rust 请求校验共用[内置模型目录](core/models.json)。自定义目录位于 `~/Library/Application Support/app.picaso.desktop/models.json`。同一 ID 可覆盖部分字段（包括默认值），新增 ID 需要完整的模型条目；可以复制内置条目作为起点。配置在工作台载入和提交前读取。重新打开应用可立即查看修改结果。

例如，要把 Sunburst 的新草稿默认设为透明 PNG，可创建以下文件：

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

## 本地开发

```sh
npm ci
npm run tauri -- dev
npm test
cargo test --workspace
cargo test -p picaso-cli --features fixture-provider
cargo test --manifest-path src-tauri/Cargo.toml
npm run build:mac
```

离线测试版可使用 `npm run tauri -- dev --features fixture-provider`。它使用单独的图库和本地测试供应商。正常开发包的目标为 Apple Silicon，产物位于 `src-tauri/target/aarch64-apple-darwin/release/bundle/`。平台范围、测试细节见[实施计划](docs/implementation-plan.md)与[OpenAI 接入记录](docs/openai-provider.md)。
