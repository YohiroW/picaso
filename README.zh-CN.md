# Picaso

[English](README.md) · **简体中文**

<img src="src-tauri/icons/icon.png" width="80" alt="" align="right">

Picaso 是一款用于 Windows 和 Apple Silicon Mac 的开源图像工作台。用自己的 OpenAI API 密钥生成图片，或选中一张原图，描述想改的地方。每次编辑都会保留原图和新版本，提示词、参数和作品一起保存在本机，之后可以离线查看和导出。

桌面端适合边看边改。需要批量生成或把出图接入脚本时，可以使用独立的 `picaso` 命令行工具。

## 快速开始

桌面端可从源码运行。准备好 Node.js 22.12 或更高版本（附带 npm）、Rust 稳定版，以及对应平台的编译工具。Windows 需要 Microsoft C++ Build Tools 和 WebView2，Apple Silicon Mac 需要 macOS 14 或更高版本和 Xcode Command Line Tools。安装步骤见 [Tauri 开发环境指南](https://v2.tauri.app/start/prerequisites/)。

```sh
git clone https://github.com/YohiroW/picaso.git
cd picaso
npm ci
npm run tauri -- dev
```

### 连接 OpenAI

- **Windows** — 将密钥保存为用户环境变量 `OPENAI_API_KEY`。重新打开终端并启动 Picaso，在设置里选择 **Use environment key**。
- **macOS** — 在 **Settings → API key** 输入密钥，点击 **Save and use key** 存入登录钥匙串。也可以从已设置 `OPENAI_API_KEY` 的终端启动应用，再选择 **Use environment key**。

### 生成和编辑图片

1. 在 **Prompt** 中描述画面，例如「清晨薄雾里的一座白色灯塔，横幅构图」。选择模型、画幅、质量和张数，然后点击生成。每次最多生成 4 张，输出格式和背景可在 **Advanced** 中调整。
2. 打开喜欢的图片，点击 **Continue editing**，输入「保留构图，把时间改成黄昏」，再点 **Apply edit**。也可以用提示框旁的 **Add image** 选择自己的 PNG、JPEG 或 WebP 图片，或把图片直接粘贴到提示框。
3. 点击 **Save original…** 导出原图。下次打开应用时，可从 **History** 找回作品、提示词和生成参数。

在提示框内，按 `Ctrl+Enter`（Windows）或 `⌘+Enter`（macOS）可提交当前草稿。要删除某张原图，在 **Recent creations** 中右键选择 **Delete image…** 并确认。同批的其他图片和已生成的编辑版本都会保留。

如果任务显示 **Result unconfirmed**，OpenAI 可能已经处理了请求。再次提交前，请先检查任务历史和 API 账户用量。

## 命令行

在仓库根目录安装 CLI。运行 `picaso models` 即可离线查看可用模型、尺寸和输出格式。

```sh
cargo install --locked --path cli
picaso models
```

在已设置 `OPENAI_API_KEY` 的终端中，下面的命令会生成两张灯塔图，将图片导出到 `./images`，并把任务信息写入 `generated.json`。

```sh
picaso generate --prompt "清晨薄雾里的一座白色灯塔" --count 2 --out-dir ./images --json > generated.json
```

在 macOS 终端中，可以用 `jq` 取出第二张图的 ID，编辑并导出新版本。

```sh
image_id=$(jq -er '.images[1].imageId' generated.json)
picaso edit --image-id "$image_id" --prompt "保留构图，把时间改成黄昏" --output-format png > edited.json
picaso export --image-id "$(jq -er '.images[0].imageId' edited.json)" --out ./lighthouse.png
```

- 用 `picaso show <JOB_ID>` 查询已保存的任务。任务 ID 对应结果中的 `jobId`，每张图片的 ID 对应 `images[].imageId`。
- 多行提示词可用 `--prompt-file <文件>` 或通过标准输入传入。运行 `picaso generate --help` 查看尺寸、质量、背景等选项。
- 脚本可以用 `--workspace <目录>` 指定图库。同一工作区内，以相同的 `--request-id <UUID>` 和相同输入重跑成功的任务时，CLI 会复用已有结果。

<details>
<summary>脚本如何处理失败的任务？</summary>

CLI 出错时以非零状态退出，并将错误码写入标准输出的 `error.code` 字段。

| 错误码 | 处理方式 |
| --- | --- |
| `workspace_busy` | 等待占用该图库的进程退出后再运行。 |
| `request_conflict` | 检查提示词、输入图或参数是否变更。要提交新任务，请使用新的请求 ID。 |
| `unconfirmed` | 用 `picaso show` 查询任务，并核对 API 账户用量后再决定是否重新提交。 |
| `export_failed` | 如果图片已生成，使用 `error.images` 中的图片 ID 重新导出。目标文件应位于图库外，且父目录已存在。 |

</details>

## 本地数据与模型

| 图库 | 默认位置 |
| --- | --- |
| Windows 桌面端 | `%LOCALAPPDATA%\app.picaso.desktop\` |
| macOS 桌面端 | `~/Library/Application Support/app.picaso.desktop/` |
| macOS CLI | `~/Library/Application Support/app.picaso.cli/` |

每个图库用 `library.sqlite3` 记录任务和图片引用，用 `originals/` 保存原图。备份时，先退出使用该图库的应用，再复制整个目录。macOS 钥匙串中的密钥由系统单独管理。

要调整模型默认值，可以在所用图库的根目录创建 `models.json`。已有模型只需填写 ID 和要修改的字段。例如，以下配置将 Sunburst 的默认输出设为透明 PNG。添加新模型时，可复制一条完整的[内置模型配置](core/models.json)。

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

## 开发与测试

桌面界面使用 React 和 TypeScript（[src/](src/)），由 Tauri 2 提供系统集成（[src-tauri/](src-tauri/)）。桌面端和 [CLI](cli/) 共用 [Rust 核心库](core/) 处理模型配置、生成任务和图库读写。

本地调试时，可在 `npm ci` 后运行离线测试版。它使用独立图库和本地测试图片，无需 API 密钥。首次进入设置页时，点击 **Back to workspace** 即可体验生成和编辑流程。

```sh
npm run tauri -- dev --features fixture-provider
```

在仓库根目录运行前端、核心库、CLI 和桌面端测试。

```sh
npm test
cargo test --workspace
cargo test -p picaso-cli --features fixture-provider
cargo test --manifest-path src-tauri/Cargo.toml
```

| 打包平台 | 命令 | 产物 |
| --- | --- | --- |
| Windows | `npm run tauri -- build` | NSIS 安装程序 |
| Apple Silicon Mac | `npm run build:mac` | 应用包和 DMG（ad hoc 签名） |

## 许可证

本项目采用 [MIT 许可证](LICENSE)。
