# Picaso

Picaso 是用于 Apple Silicon Mac 的本地图像工作台。使用自己的 OpenAI API 账户生成图片，重开后可从历史中选择一张原图继续修改。当前交付为本地开发版。

## 安装与使用

1. 打开 [0.1.3 开发版 DMG](src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Picaso_0.1.3_aarch64.dmg)，把 Picaso 拖入 Applications。最低系统版本为 macOS 14。开发版采用 ad hoc 签名。
2. 首次打开时选择密钥来源。如果窗口检测到 `OPENAI_API_KEY`，可以点击 **Use environment key**。如果只在终端中设置了环境变量，可以从该终端启动 `~/Applications/Picaso.app/Contents/MacOS/picaso`（安装在系统 Applications 时去掉 `~`）。也可以在 **Key settings** 输入自己的 API Key，点击 **Save and use key** 存入当前用户的 macOS 钥匙串。可在此更换、切换或删除密钥。API 额度由 OpenAI API 账户提供，与 ChatGPT 订阅分别计费。
3. 在左栏 **Prompt** 描述图片，选择画幅和质量，点击 **Generate image**。右侧按时间展示最近 8 张作品。末尾 **＋** 可回到新建草稿。右上角 **History** 可查看所有任务，包括失败或结果未确认的任务。点击任意一张作品可预览原图。点击 **Continue editing**，在 **Edit instructions** 中输入修改要求，再点 **Apply edit** 或按 ⌘↩。新结果和原图都会保留。
4. **Save original…** 可将选中的 PNG 保存到自行选择的位置。预览窗口显示历史任务的提示词和实际参数。查看或导出旧图无需联网，首次使用也可以选择 **Browse without a key**。

只有明确提交“生成”或“编辑”会向 OpenAI 请求图片。超时或意外退出后，如果任务显示 **unconfirmed**，上游可能已经计费，请先检查历史和账户用量再决定是否重新提交。

## 数据与升级

图库位于 `~/Library/Application Support/app.picaso.desktop/`，其中 `library.sqlite3` 存放任务与图片引用，`originals/` 保存原图，`credential-source.json` 只保存选用的来源。备份时先退出应用，再复制整个目录。手动密钥由系统钥匙串单独管理。升级时退出旧版并替换应用包，保留数据目录。

## 本地开发

```sh
npm ci
npm run tauri -- dev
npm test
cargo test --manifest-path src-tauri/Cargo.toml
npm run build:mac
```

离线测试版可使用 `npm run tauri -- dev --features fixture-provider`。它使用单独的图库和本地测试供应商。正常开发包的目标为 Apple Silicon，产物位于 `src-tauri/target/aarch64-apple-darwin/release/bundle/`。平台范围、测试细节见[实施计划](docs/implementation-plan.md)与[OpenAI 接入记录](docs/openai-provider.md)。
