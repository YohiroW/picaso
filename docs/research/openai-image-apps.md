# 用自己的 OpenAI API 额度生图：5 个开源项目对比

核对日期：2026-11-25。范围是能接 OpenAI 官方图片接口的应用，或对实现有明确参考价值的示例；不含需要自备 GPU 的本地推理工具。本次核对了仓库文档、关键源码、根许可证和 OpenAI 官方文档；没有部署这些应用，也没有使用真实 Key 发出付费请求。下方源码链接固定到核对时的提交。

**结论：如果 Picaso 首先是给自己用的，最值得参考的是 Houtx 的 Web 版。** 它已经把「服务端读自己的 Key → 文生图或改图 → 保存图片、参数和费用估算」串起来。想设计更好的创作体验，看 CookSleep；想让用户装在本机、又不让网页接触 OpenAI Key，看 honlnk 的 Companion。不必为了实现这条主流程，先引入桌面壳、多供应商或账号系统。

| 项目 / 根许可证 | 谁提供 Key，请求从哪发 | 最值得借鉴 | 不宜直接照搬 |
|---|---|---|---|
| [Houtx/gpt-image-playground][houtx] · [MIT][houtx-license] | Web：部署者在服务端配 Key；Android：用户在设备上配 Key | 从提示词到下载、历史、用量估算的完整小应用 | 两端的密钥边界不同；首版不必同时维护 APK |
| [CookSleep/gpt_image_playground][cook] · [MIT][cook-license] | 用户配置端点和 Key；Web 支持浏览器直连，也可配代理 | 参考图、遮罩、批量任务、结果复用、本地画廊 | 静态网页的配置不是秘密存储；不需要先复制所有供应商 |
| [honlnk/gpt-image-studio][hon] · [MIT][hon-license] | 直连模式由浏览器持 Key；Companion 模式由本机服务持 Key | 同一套聊天式界面，如何从浏览器存储切到本地服务 | Companion 的 OpenAI 兼容预设不是官方地址，不能只看名称 |
| [openai/openai-imagegen-demo][oai] · [MIT][oai-license] | 部署者在服务端配 Key，默认请求官方 `/images/edits` | 生成中预览、按风格展示进度、单项失败重试 | 是基于输入照片的编辑示例，不是通用文生图应用 |
| [vercel-labs/ai-sdk-image-generator][vercel] · [Apache-2.0][vercel-license] | 各供应商 Key 在部署环境，Next.js 路由调用 AI SDK | 同一提示词并排对比不同供应商；把尺寸等差异留给适配层 | 该提交的 OpenAI 模型只有已下线的 DALL·E 2/3，只作架构参考 |

## 具体看哪些代码

### 1. Houtx：最接近个人版 Picaso 的现成产品

它基于 [alasano/gpt-image-playground](https://github.com/alasano/gpt-image-playground) 扩展。先读 [Next.js 图片路由][houtx-route]：由服务端读取 `OPENAI_API_KEY`，通过 OpenAI SDK 分别调用生成或编辑接口。**存 Key 的位置与存图的位置是两个决定**：Web 版可以让 Key 始终留在服务端，本地运行把图保存到 `generated-images/`；部署到没有持久磁盘的环境时，改存浏览器 IndexedDB。历史里同时保留提示词、参数、token 用量和美元估算，适合回答「上次这张图是怎么做出来的、花了多少」。[功能、双端差异与部署说明][houtx]

### 2. CookSleep：最值得借鉴的是“出图后怎么继续改”

React + TypeScript + Vite + Zustand，Web 可静态部署，Windows 客户端使用 Tauri。生成结果可以直接作为下一次的参考图，配合画板标注或局部遮罩，不必先下载再上传；任务支持批量提交、查看实际参数与失败重试。[工作流和技术栈][cook]。再读 [存储分发层][cook-db]：同样的任务和图片操作，Web 落到 IndexedDB，桌面端落到 Rust 侧的 SQLite 与图片文件。即使首版没有桌面端，也值得把图库读写与界面分开。

### 3. honlnk：如果想让用户“自带 Key”，先看它的两种个人模式

Vue 3 + Vite + Pinia 的聊天式工作台。直连模式部署最轻，但页面需要接触供应商 Key；本地 Companion 模式则用 CLI 配置供应商凭据，网页用单独的连接密钥访问回环地址上的 Fastify 服务，不需要拿到 OpenAI Key。它还提供 Docker/JWT 多用户模式，但那解决的是另一种部署需求。[连接方式与技术栈][hon]

一个容易漏掉的细节：在核对的提交中，[Companion 的 OpenAI 兼容预设][hon-profile] 的 `defaultBaseUrl` 指向第三方域名。**“兼容 OpenAI 协议”不等于“使用自己的 OpenAI 额度”。** 如果选它试用，应先核对最终请求发往 `api.openai.com`，再配置官方 Key；不要向第三方中转或不受信任的在线演示提交真实 Key。

### 4. OpenAI Photobooth：官方编辑接口和流式反馈的短路径

Next.js App Router + TypeScript + Tailwind。用户上传或拍摄肖像、选择风格，服务端发出编辑请求。[路由源码][oai-route]把上游返回的中间预览和最终图转成带风格 ID 的 SSE（服务端推送事件）；界面可以保留成功结果，只重试失败的那个风格。[示例说明][oai]。它适合参考“请求耗时几十秒时用户能看到什么”，而不是拿来承担图库或通用文生图。

### 5. Vercel AI SDK Image Generator：读适配层，不沿用模型清单

[路由][vercel-route]把 OpenAI、Replicate、Vertex AI 和 Fireworks 统一到 `generateImage`，同时保留有些模型接收 `size`、有些接收 `aspectRatio` 的区别。[界面][vercel]支持一次输入、并排看不同供应商的结果。限制也很具体：[模型配置][vercel-models]只列出 `dall-e-2` 和 `dall-e-3`，而 [OpenAI 下线记录][deprecations]已列出二者于 2026-05-12 停用。因此不应把它当成“只填今天的 OpenAI Key 就能用”的成品。等确实需要第二家供应商时，再参考这种设计。

## 对 Picaso 首版的选型建议

- **部署和密钥：** 若只有自己使用，建议选本机 Web 应用：浏览器 → 仅监听 `127.0.0.1` 的 Next.js 服务端 → `https://api.openai.com/v1`。Key 通过不入库的服务端环境变量设置，不进前端包、URL、日志或历史导出。如果将来供他人使用，要重新决定是谁付费；把个人服务直接公开，即使看不到 Key，别人也可能消耗你的额度。[OpenAI 密钥安全指南][key-safety]
- **API 与界面：** 首版用 Image API：`/v1/images/generations` 做文生图，`/v1/images/edits` 接参考图或遮罩。先做「提示词 → 生成 → 下载 → 从历史复用参数」。有聊天界面不等于必须用对话 API；当需求变成模型要理解多轮上下文，再考虑 Responses API 的 `image_generation` 工具。模型与参数按当前官方支持和账户权限选择，而不是照抄示例常量。[OpenAI 图片指南][image-guide]
- **历史与费用：** 本机版建议原图落文件，SQLite 只记文件路径、提示词、模型、参数和可得的用量；这样图库和复现记录都在自己手中。估价注明模型与计价依据，不把它显示成精确扣费或账户余额。首版默认单张，失败由用户决定是否重试，避免超时后自动重复提交付费任务。[费率][pricing] · [API 与 ChatGPT 分开计费][billing]

[houtx]: https://github.com/Houtx/gpt-image-playground/blob/f9a5cc8b9ffdddf6f9b0a4702d6f4532e1a5b0de/README.md
[houtx-route]: https://github.com/Houtx/gpt-image-playground/blob/f9a5cc8b9ffdddf6f9b0a4702d6f4532e1a5b0de/src/app/api/images/route.ts
[houtx-license]: https://github.com/Houtx/gpt-image-playground/blob/f9a5cc8b9ffdddf6f9b0a4702d6f4532e1a5b0de/LICENSE
[cook]: https://github.com/CookSleep/gpt_image_playground/blob/4001f4eccad88025dbbcdcdff0c6fd290c80031e/README.md
[cook-db]: https://github.com/CookSleep/gpt_image_playground/blob/4001f4eccad88025dbbcdcdff0c6fd290c80031e/src/lib/db.ts
[cook-license]: https://github.com/CookSleep/gpt_image_playground/blob/4001f4eccad88025dbbcdcdff0c6fd290c80031e/LICENSE
[hon]: https://github.com/honlnk/gpt-image-studio/blob/de6ba4b0e97395a6598856e8a394b581eac7a089/README.md
[hon-profile]: https://github.com/honlnk/gpt-image-studio/blob/de6ba4b0e97395a6598856e8a394b581eac7a089/companion/src/providers/profiles/openai.json
[hon-license]: https://github.com/honlnk/gpt-image-studio/blob/de6ba4b0e97395a6598856e8a394b581eac7a089/LICENSE
[oai]: https://github.com/openai/openai-imagegen-demo/blob/5c18c2abd96f4345e5f0ba2484c71280814f1b16/README.md
[oai-route]: https://github.com/openai/openai-imagegen-demo/blob/5c18c2abd96f4345e5f0ba2484c71280814f1b16/app/api/photobooth/route.ts
[oai-license]: https://github.com/openai/openai-imagegen-demo/blob/5c18c2abd96f4345e5f0ba2484c71280814f1b16/LICENSE
[vercel]: https://github.com/vercel-labs/ai-sdk-image-generator/blob/5130ba8895d3944c81d577565dca3c3486d6c923/README.md
[vercel-route]: https://github.com/vercel-labs/ai-sdk-image-generator/blob/5130ba8895d3944c81d577565dca3c3486d6c923/app/api/generate-images/route.ts
[vercel-models]: https://github.com/vercel-labs/ai-sdk-image-generator/blob/5130ba8895d3944c81d577565dca3c3486d6c923/lib/provider-config.ts
[vercel-license]: https://github.com/vercel-labs/ai-sdk-image-generator/blob/5130ba8895d3944c81d577565dca3c3486d6c923/LICENSE
[image-guide]: https://developers.openai.com/api/docs/guides/image-generation
[deprecations]: https://developers.openai.com/api/docs/deprecations
[key-safety]: https://help.openai.com/en/articles/5112595-best-practices-for-api-key-safety
[billing]: https://help.openai.com/en/articles/9039756-managing-billing-for-chatgpt-and-the-api-platform
[pricing]: https://openai.com/api/pricing/
