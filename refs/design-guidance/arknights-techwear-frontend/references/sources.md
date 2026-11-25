# 研究来源、观察与边界

核对日期：**2026-10-18，America/Los_Angeles**。本轮阅读了两作官方页面、官方游戏场景展示图及 Nike 官方产品/设计说明。视觉判断是本次分析；未获得鹰角完整设计系统或设计团队访谈，不能称为官方设计理念或官方规范。

## S1 · 明日方舟官网：档案与海报的并置

[首页](https://ak.hypergryph.com/) · [情报](https://ak.hypergryph.com/#information) · [干员](https://ak.hypergryph.com/#operator) · [设定](https://ak.hypergryph.com/#world)

本轮实际浏览截图与页面文本：

- 黑灰画布、青色选中/页号、细线分区、英文大写导航配短中文标签。
- 干员页把大幅角色图、低明度背景像、超大底字与较窄的档案信息栏并置；文字包括 PROFILE、CHARACTER VOICE。
- 情报区用类别、日期、标题行与大幅内容视觉并置；设定区用对齐目录行和分隔线组织。
- 首页和页面框架有编号页轨；不是所有编号都值得在产品内重现。

**归纳：**图像承担情绪与主体，网格、索引和行结构支持检索。前端可借用尺度差、档案行和选中信号，而不是复制游戏官网的全部装饰英文、加载或全屏导航机制。

## S2 · 终末地官网：工业导视与开放内容区

[首页](https://endfield.hypergryph.com/) · [干员](https://endfield.hypergryph.com/#operator) · [玩法与工业介绍](https://endfield.hypergryph.com/#gameplay)

本轮实际浏览：

- 白灰大面、黑色字与黄色框架信号；左侧导航轨保持可定位的结构。
- 干员页有排线填充大字、几何标记、标签化元数据、图像叠层；玩法区有超大 ENDFIELD 和黄色竖向导视带。
- 背景可见网点/等高线感纹理。当前版本内容美术也使用绿、金与绘画式主视觉，说明官网框架和活动美术不能混为一套固定皮肤。
- 页面有 [REC]、序号、LORE、阵营/种族/配音等元数据。部分细节是展示叙事，不一定是实际应用所需信息。

[官网「自动化生产」游戏场景图](https://web.hycdn.cn/endfield/official-v4/_next/static/media/01.aae95e75.png) 经 HTTP 与图像查看核对（1600×900）。场景为自然环境中的成组工业设备与输送结构，浅色壳体、深接缝/轨道、小面积黄红标记、重复模块与真实连接路径。

**归纳：**前端可借用浅色壳层、深色结构边界、少量信号色和模块间有含义的连接。场景图没有 HUD；本轮不能据此断言全部游戏内交互、字体、间距或色值。后续若要复现具体游戏屏幕，应重新取证该屏幕。

## S3 · 可拆与防装错：Nike ACG Smith Summit

[官方商品 HV0591-368](https://www.nike.com/t/acg-smith-summit-mens-zip-cargo-pants-Vgm0Hb/HV0591-368)

官方说明包含 detachable lower panels；“We added color-coded zippers to ensure each leg ends up in the right place.”；双层 CORDURA 膝部加强；带拉链收纳；织带与快拆扣调节。

**明确机制：**可拆模块、接口辨识、局部防护与快速操作。**前端类比：**详情收起/展开、成组控件、颜色与形状共同标明连接/选择、重要操作更强边界。不是要求在界面画服装部件。

## S4 · 按环境分层：Nike 穿衣分层指南

[How to Layer Clothes for Every Season](https://www.nike.com/a/how-to-layer-clothes)（页面标注 2023-03-01 更新）

官方区分 base / mid / shell：底层处理水分与体温，中层保温且便于增减，外层防风雨；天气与运动强度决定层数。

**前端类比：**内容基础稳定，任务控制按需出现，临时状态用适当的提示层；小屏重新组织而非缩小全部层。这是可选思考模型，不是固定三层 DOM/z-index 架构。

## S5 · 轻量也是机能：ACG Lava Flow Ultralight

[Behind the Design](https://www.nike.com/acg/acg-lava-flow-ultralight-behind-the-design)

官方比较功能丰富外套的携带负担，并提到 super light hardware and zippers、保温与重量的取舍、可独立穿/放在外壳下/收进背包。

**前端类比：**每个可见控件都带来注意力成本，默认露出高价值操作，次要操作可收纳；不将机能风等同于满屏框线、条码或动画。

## S6 · 可识别色服务任务：ACG Racing Department

[All Conditions Racing Department Kit](https://www.nike.com/acg/acg-racing-department-kit)

官方说明 “athletes will be easy to spot this year—their kit is dressed up in screaming oranges”，并结合越野跑具体需求解释装备。

**前端类比：**高可见色可以承担找路径、找当前动作的职责；颜色可以不是黄或青，但作用应稳定，并有非颜色状态线索。

## 如何使用这些结论

- 观察和类比不等于原作者明确表达的意图。本 skill 的命名、调色、布局比例、断点、字号和动效参数是自拟的实现建议。
- ACG 可代表一组有材料与功能证据的例子，不能覆盖全部 techwear 流派。ACRONYM 与 Stone Island 相关页本轮返回 403，未引用为已核验依据。
- 未识别两作官方字体文件、全量组件或断点；不以猜测补上所谓官方 token。
- 官网会随版本更新，CDN URL 也可能变化。需要精确复现时重新查看用户指定的那一版；通常使用本地参考即可，不必每次重新研究。
- 来源链接用于研究追溯，不包含复用原站代码、Logo、角色立绘、徽章、字体或其他第三方素材的许可。资产目录仅提供本次原创的 CSS 与 HTML 示例。
