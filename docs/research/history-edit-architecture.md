# Picaso：从历史图片继续修改，应该借哪部分架构？

**建议：借 Houtx 的服务端调用，借 CookSleep 的「输出图可以成为下一次任务的输入」；两者之间用图片 ID 衔接，不用文件名、Base64 或聊天记录衔接。** 这样同一张历史图可以改两次、得到两个不同结果，原图始终保留。

本文只分析 `refs/` 里的本地副本（Houtx `package.json` 为 1.1.0，CookSleep 为 0.9.5），聚焦 Web 画廊路径。下文先描述源码，再给 Picaso 的取舍；未运行参考应用或调用付费接口。

## Houtx：保留调用方向，改掉历史图的往返搬运

1. **密钥在服务端。** Web 页面的 [`handleApiCall`][h-submit] 把提示词、参数和 `mode` 放进 `FormData`，编辑时附上 `image_0` 等文件和可选 `mask`，发到同源 `/api/images`。路由中的 [`createOpenAIClient`][h-route] 从环境读 `OPENAI_API_KEY`；`POST` 按模式调用 `openai.images.generate` 或 [`openai.images.edit`][h-edit]。普通结果含图片与 `usage`，流式分支用 SSE 给页面推预览。Android 有另一套调用方式，不属于这个选择。
2. **历史与文件不是一回事。** [`HistoryMetadata`][h-meta] 存提示词、模式、输出文件名、费用等，[页面][h-history] 把这份索引保存在 `localStorage`；原图则在 `generated-images/` 或浏览器 IndexedDB。即使选了磁盘模式，清掉浏览器数据也会丢失这份历史索引，而磁盘上仍可能留有图片。记录里没有这次编辑使用的源图 ID。
3. **它其实已经能改历史图。** 从历史中选一批结果，打开其中一张，输出面板把[当前图片的文件名][h-output]交给 [`handleSendToEdit`][h-send]。磁盘模式会再次 GET 原图，转换成浏览器 `File`，替换编辑表单的参考图；用户提交时再把文件传回本机服务端。这对「用户刚上传的文件」很自然，但对「服务端明明已有的历史图」是多走一趟。

**取舍：** 留下浏览器 → 本机 Next.js → OpenAI SDK 这条路。已有图片提交 ID，新上传的图片才传文件。让同一个服务端负责原图及其历史索引，而不是让页面分别协调文件系统、IndexedDB 和 `localStorage`。

## CookSleep：关键不是 Zustand，而是分清三个动作

| 用户动作 | 本地源码的实际行为 |
|---|---|
| 编辑输出 | [画廊卡片][c-grid]调用 [`editOutputs(task)`][c-edit]。它遍历该任务的 **全部** `outputImages`，按 ID 加载图片，经 [`toReferenceImage`][c-ref] 处理后，追加到当前输入区，跳过已有参考图。不会因此提交请求，也不会把旧提示词填回来。 |
| 复用配置 | [`reuseConfig(task)`][c-reuse] 取回的是旧任务的**输入图**、提示词和参数；若原遮罩及目标图仍存在，再恢复遮罩。这是重用上次配方，不是拿上次的成品改图。 |
| 提交任务 | [`submitTask`][c-submit] 固化此刻的输入图 ID、提示词和参数，生成新任务 ID，先持久化 `running` 记录；[`executeTask`][c-execute] 再解析图片、请求接口、保存输出图，最后把输出 ID 和状态写回该任务。后续改输入框，不会让这条记录变成另一份草稿。 |

背后的数据结构比界面更值得参考：[`TaskRecord`][c-task] 记录一次请求，`inputImageIds` 与 `outputImages` 都引用单独存储的图。图片存储[按内容去重][c-db]，任务不必各自嵌入一份大图。同一份画廊逻辑可以跑在不同存储上；**Picaso 需要的是任务和图片分开，不是也实现两套存储。**

## 放到 Picaso：让「继续修改」作用于所选单图

例如，任务 A 输出了两张图 `a1`、`a2`。用户选 `a2`，输入「天空换成黄昏」，再点提交：

```text
任务 A：无参考图 + 原始提示词    → generate → [a1, a2]
点 a2 的「继续修改」           → 草稿引用 a2；不发 OpenAI 请求
任务 B：[a2] + 天空换成黄昏    → edit     → [b1]
```

- **界面只管草稿。** 按钮位于每张结果图上，而不是只在整条任务上。进入编辑时展示所选原图，请用户写本次修改指令；不默认带入整批输出、旧提示词或旧遮罩。若输入区已有未提交内容，不静默丢弃。另设「复用参数」时也不要混用这个动作。
- **本机历史是唯一记录。** 用 SQLite 记任务（ID、提示词、参数、状态、输入/输出图 ID、可得的用量），用本地文件存原图，图片 ID 映射到服务端管理的文件。浏览器用预览地址显示图片，但再编辑只交 ID。仅记录“来自任务 A”不够，因为 A 可能有两张输出。
- **付费调用只有一个入口。** 提交时服务端按 ID 查原图；无参考图用 `images.generate`，有参考图用 `images.edit`，本机读取原图后交给 SDK。图片不存在就告诉用户重新选择，**不要静默降级成文生图**。Key 与官方端点只由服务端配置。执行后写一条新结果，不覆盖源图；以后加 SSE，只改变反馈方式，不改变任务和图片的关系。
- **先不做的部分。** 不需要会话 ID、Agent、多供应商、桌面壳或后台队列。也不必照搬 Houtx 页面中分散在流式/非流式分支的历史写入，或 CookSleep 集中在大型 `store.ts` 中的任务执行：让浏览器管理草稿，服务端统一写历史即可。

**第一条验收场景：** 用模拟的 OpenAI 调用生成 `a1/a2` → 刷新后从历史选 `a2` → 点「继续修改」时调用计数不变 → 提交后恰有一次 `edit`，收到的是 `a2` 的原图和新指令 → 刷新后 A、B 均可查看，并能再次从 `a2` 改出另一版。再测一次源文件缺失：应提示失败、保留草稿，付费调用计数不变。这条用例可以在不消耗额度的情况下检验两套架构是否接对了。

[h-submit]: ../../refs/gpt-image-playground-houtx/src/app/page.tsx#L488
[h-route]: ../../refs/gpt-image-playground-houtx/src/app/api/images/route.ts#L26
[h-edit]: ../../refs/gpt-image-playground-houtx/src/app/api/images/route.ts#L291
[h-meta]: ../../refs/gpt-image-playground-houtx/src/app/page.tsx#L25
[h-history]: ../../refs/gpt-image-playground-houtx/src/app/page.tsx#L220
[h-output]: ../../refs/gpt-image-playground-houtx/src/components/image-output.tsx#L234
[h-send]: ../../refs/gpt-image-playground-houtx/src/app/page.tsx#L1056
[c-grid]: ../../refs/gpt_image_playground-cooksleep/src/components/TaskGrid.tsx#L330
[c-edit]: ../../refs/gpt_image_playground-cooksleep/src/store.ts#L4286
[c-ref]: ../../refs/gpt_image_playground-cooksleep/src/lib/inputImages.ts#L29
[c-reuse]: ../../refs/gpt_image_playground-cooksleep/src/store.ts#L4209
[c-submit]: ../../refs/gpt_image_playground-cooksleep/src/store.ts#L1740
[c-execute]: ../../refs/gpt_image_playground-cooksleep/src/store.ts#L3777
[c-task]: ../../refs/gpt_image_playground-cooksleep/src/types.ts#L328
[c-db]: ../../refs/gpt_image_playground-cooksleep/src/lib/db.ts#L187
