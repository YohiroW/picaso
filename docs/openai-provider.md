# OpenAI 图片适配器

2026-11-25 核对官方文档。默认服务商为 OpenAI，模型为 `gpt-image-2`，同一模型支持生成和编辑。当前版本固定请求一张 1024 × 1024、low 质量的 PNG，便于预测一次提交的范围。OpenAI 按所选 API 账户计费。[模型说明](https://developers.openai.com/api/docs/models/gpt-image-2) · [图片 API 指南](https://developers.openai.com/api/docs/guides/image-generation)

生成使用 `POST /v1/images/generations` 的 JSON 请求。编辑使用 `POST /v1/images/edits` 的 multipart 请求，Rust 从图库读取所选单张原图，通过 `image` 字段提交。两个端点均固定为 `https://api.openai.com`。响应中的 base64 解码为原始 PNG，服务商提供的 usage 与任务一起保存。[编辑接口](https://developers.openai.com/api/reference/resources/images/methods/edit)

自动测试使用虚构凭据和本地 HTTP 服务校验请求路径、字段、原图字节、用量和错误映射。对连接中断、超时及服务端结果未确认的请求提示再次提交可能产生费用，由用户决定下一次操作。

## 开发版实测

在安装的 0.1.1 应用中明确选择环境变量后，从主工作台生成 Picaso 折纸字母 P 图标，关闭并重开应用，从历史原图继续修改以改善 Dock 小尺寸可读性。生成、编辑各成功一次，返回的 total_tokens 分别为 277、1289。两张原图均为 1024 × 1024 PNG，数据库保留两条成功任务和一条原图引用。从原生保存面板导出的[编辑版母图](../src-tauri/icons/picaso-master.png)字节与图库原图一致，SHA-256 为 `b6a8e263527560532ea4421c3803be1b314d67e519fcc3669e23f9b5ec1e98e8`。
