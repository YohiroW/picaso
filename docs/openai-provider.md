# OpenAI 图片适配器

2026-11-25 核对官方文档。默认服务商为 OpenAI，模型为 `gpt-image-2`，同一模型支持生成和编辑。0.1.4 从模型目录读取画幅、质量、张数、格式和背景能力。默认模型使用方形、Low、1 张 PNG 和 Auto 背景。支持相应能力的模型可进行自定义像素尺寸、Extra high 或 Max 质量、透明输出，以及 SVG 或 GIF 生成。SVG 限单张生成；透明背景限 PNG 或 WebP。GIF 可用性、分辨率和动画像素预算还受所选模型和账户权限影响。[图片输出选项](https://developers.openai.com/api/docs/guides/image-generation)

OpenAI 按所选 API 账户计费。[模型说明](https://developers.openai.com/api/docs/models/gpt-image-2) · [图片 API 指南](https://developers.openai.com/api/docs/guides/image-generation)

生成使用 `POST /v1/images/generations` 的 JSON 请求。编辑使用 `POST /v1/images/edits` 的 multipart 请求，Rust 从图库读取所选单张原图，通过 `image` 字段提交。两个端点均固定为 `https://api.openai.com`。每次提交仅发送一次请求，张数通过 `n` 指定。输入文件 MIME 按原图格式设置，输出字节按所选格式验证、逐张保存，服务商提供的 usage 与任务一起保存。[编辑接口](https://developers.openai.com/api/reference/resources/images/methods/edit)

自动测试使用虚构凭据和本地 HTTP 服务校验请求路径、字段、原图字节、用量和错误映射。对连接中断、超时及服务端结果未确认的请求提示再次提交可能产生费用，由用户决定下一次操作。

## 开发版实测

在安装的 0.1.1 应用中明确选择环境变量后，从主工作台生成 Picaso 折纸字母 P 图标，关闭并重开应用，从历史原图继续修改以改善 Dock 小尺寸可读性。生成、编辑各成功一次，返回的 total_tokens 分别为 277、1289。两张原图均为 1024 × 1024 PNG，数据库保留两条成功任务和一条原图引用。从原生保存面板导出的[编辑版母图](../src-tauri/icons/picaso-master.png)字节与图库原图一致，SHA-256 为 `b6a8e263527560532ea4421c3803be1b314d67e519fcc3669e23f9b5ec1e98e8`。
