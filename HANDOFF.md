# Picaso 工作站交接

首个桌面包优先交付 Apple Silicon macOS。当前工程位于仓库根目录，采用 Tauri 2、Vite、React 和 TypeScript。`refs/` 为只读参考资料。

## 从这里继续

1. [需求基线](docs/requirements.md) 定义产品行为和平台顺序。
2. [实施计划](docs/implementation-plan.md) 定义 M0–M5 的交付与验收条件。
3. [macOS M0 安装记录](docs/m0-macos.md) 提供构建命令、包体、签名状态和实际验证范围。此阶段无需 OpenAI Key。当前开发版安装与测试结果见[开发版验收](docs/development-build.md)，操作流程见[使用说明](README.md)。
4. 设计后续工作台时参考[历史单图继续修改的源码走读](docs/research/history-edit-architecture.md)和[Field 布局指引](refs/design-guidance/arknights-techwear-frontend/SKILL.md)。后续的密钥来源、任务去重和单图修改应使用假数据测试。

Windows x64 当前用户 NSIS 及 WebView2 安装体验仍需在 Windows 平台单独验收。

## 参考资料

[开源项目调研](docs/research/openai-image-apps.md)保留了上游项目、版本和根许可证信息。`refs/` 内含[Claude+ 主题](refs/claude.css)、两个参考项目的只读源码节选及根许可证。解压或迁移工程时保持目录相对位置，按锁文件重建依赖与安装包。

M1、M3 可使用 `impeccable` 检查引导、工作台、键盘焦点与对比度。实现凭据选择、提交去重及历史图编辑时可使用 `tdd`；真实付费冒烟测试仍需单独征得许可。

离线验收图库可运行 `npm run tauri -- dev --features fixture-provider`。点击 “Browse without a key” 进入工作台。此编译选项提供本地测试图片，图库在应用数据目录的 `fixture-workspace/` 下，钥匙串条目也单独命名。
