# Picaso

Picaso 是用于 Apple Silicon Mac 的本地图像工作台。使用自己的 OpenAI API 账户生成图片，重开后可从历史中选择一张原图继续修改。当前交付为本地开发版。

## 安装与使用

1. 打开 [0.1.6 开发版 DMG](src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Picaso_0.1.6_aarch64.dmg)，把 Picaso 拖入 Applications。最低系统版本为 macOS 14。开发版采用 ad hoc 签名。
2. 首次打开时选择密钥来源。如果窗口检测到 `OPENAI_API_KEY`，可以点击 **Use environment key**。如果只在终端中设置了环境变量，可以从该终端启动 `~/Applications/Picaso.app/Contents/MacOS/picaso`（安装在系统 Applications 时去掉 `~`）。也可以从右上角齿轮打开 **Settings → API key**，输入自己的 API Key，点击 **Save and use key** 存入当前用户的 macOS 钥匙串。可在此更换、切换或删除密钥。API 额度由 OpenAI API 账户提供，与 ChatGPT 订阅分别计费。
3. 在左栏 **Prompt** 描述图片，选择模型、画幅、质量和张数（1–4）。自定义画幅可填写像素宽高；**Advanced** 可选择输出格式与背景，点击 **Generate image**。右侧最多显示最近 8 个图片位置。提交后，每张图会按所选画幅显示 **Generating…** 占位，缩略图就绪后在原位显示。失败或结果未确认时可点 **Edit prompt** 恢复提示词、参数和参考图，再决定是否提交。末尾 **＋** 可回到新建草稿。右上角 **History** 可查看所有任务，包括失败或结果未确认的任务。点击任意一张作品可预览原图。点击 **Continue editing**，在 **Edit instructions** 中输入修改要求，再点 **Apply edit** 或按 ⌘↩。新结果和原图都会保留。
4. **Save original…** 可将选中的原始文件 保存到自行选择的位置。在作品详情中点击提示词末尾的 **Copy** 可复制全文。点击图片右上角的放大镜可单独看图，按 Esc 返回详情。右键点击 **Recent creations** 中的图片，选择 **Delete image…** 并确认后，会从图库和 History 永久删除这张原图。同批其他图片和已生成的编辑结果会保留。查看或导出旧图无需联网，首次使用也可以点击 **Back to workspace**。

只有明确提交“生成”或“编辑”会向 OpenAI 请求图片。超时或意外退出后，如果任务显示 **unconfirmed**，上游可能已经计费，请先检查历史和账户用量再决定是否重新提交。

## 数据与升级

图库位于 `~/Library/Application Support/app.picaso.desktop/`，其中 `library.sqlite3` 存放任务与图片引用，`originals/` 保存原图，`credential-source.json` 只保存选用的来源。备份时先退出应用，再复制整个目录。手动密钥由系统钥匙串单独管理。升级时退出旧版并替换应用包，保留数据目录。

## 模型配置

模型下拉框和 Rust 请求校验共用[内置模型目录](src-tauri/models.json)。自定义目录位于 `~/Library/Application Support/app.picaso.desktop/models.json`。同一 ID 可覆盖部分字段（包括默认值），新增 ID 需要完整的模型条目；可以复制内置条目作为起点。配置在工作台载入和提交前读取。重新打开应用可立即查看修改结果。

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
cargo test --manifest-path src-tauri/Cargo.toml
npm run build:mac
```

离线测试版可使用 `npm run tauri -- dev --features fixture-provider`。它使用单独的图库和本地测试供应商。正常开发包的目标为 Apple Silicon，产物位于 `src-tauri/target/aarch64-apple-darwin/release/bundle/`。平台范围、测试细节见[实施计划](docs/implementation-plan.md)与[OpenAI 接入记录](docs/openai-provider.md)。
