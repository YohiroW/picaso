# M0 · Picaso 0.1.0 安装记录

2026-11-25 在 Apple Silicon、macOS 26.7.1（25G241）的开发工作站上，从本地构建的 DMG 安装并首次启动成功。应用使用系统 WebKit，未出现额外的运行时下载或安装步骤。M0 只展示本机安装信息，无需 OpenAI Key，启动流程没有供应商调用。

| 项目 | 本次实测 |
|---|---|
| 安装包 | [Picaso_0.1.0_aarch64.dmg](../src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Picaso_0.1.0_aarch64.dmg)，2,314,579 字节（2.21 MiB） |
| 应用包 | `Picaso.app` 文件内容合计 5,717,392 字节（5.45 MiB）。可执行文件为 arm64，包声明最低 macOS 14.0 |
| DMG SHA-256 | `2b9e47d3dbd29f169a3ca8c5d85842f2c2225aa381565eacbdc0c6407e5465e3` |
| 应用标识 | `app.picaso.desktop` |
| 签名 | 应用采用 ad hoc 本地签名，安装副本通过 `codesign --verify --deep --strict`。此内部包未经 Apple 公证 |

## 安装与首次启动

1. 使用 `hdiutil verify` 校验 DMG，再只读挂载。映像中可见 `Picaso.app` 和指向 `/Applications` 的快捷方式。手动安装时可将应用拖入该文件夹。[Tauri DMG 安装方式](https://v2.tauri.app/distribute/dmg/)
2. 本次用 `ditto` 将应用复制到当前用户的 `~/Applications/Picaso.app`，退出磁盘映像，再用 Launch Services 打开副本。验证前该位置与 Picaso 数据目录均不存在。
3. 窗口实际显示 `Version` 为 `0.1.0`，`Data folder` 为当前用户的 `~/Library/Application Support/app.picaso.desktop` 完整路径。已从窗口辅助功能树读回这两项，并在磁盘核对目录已创建。
4. 安装副本的二进制直接链接系统 `WebKit.framework`，前端从 `tauri://localhost` 加载。无需启动 Vite 或额外安装浏览器引擎。未测量系统级网络流量。

本次覆盖开发机上的本地安装。干净 Mac、macOS 14 实机和从互联网下载后带隔离标记的 Gatekeeper 首开体验尚待验证。Windows x64 安装包、WebView2 的额外下载量及缺少运行时的离线提示顺延至 Windows 验收。

## 复现构建

在已安装 Xcode 开发工具、Node.js、npm、Rust 和 `aarch64-apple-darwin` 目标的 Mac 上，从仓库根目录执行。本次使用 Node.js 26.11.1、npm 11.20.0、Rust 1.99.0 和 Xcode 27.2，npm 与 Cargo 均有锁文件。

```sh
npm ci
npm run build:mac
```

构建命令包含 TypeScript 检查和 Vite 生产构建。产物在 `src-tauri/target/aarch64-apple-darwin/release/bundle/{macos,dmg}/`。开发时可用 `npm run tauri -- dev` 打开窗口。另已通过 `cargo fmt --manifest-path src-tauri/Cargo.toml --check` 及 release 配置下的 `cargo test`，当前 Rust 单元测试数量为 0。安装与窗口检查结果见上文。
