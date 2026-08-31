# Marker 推广工具包

最后更新：2026-08-31

公开发帖、回复、简介与上架表单以此文件为准。常青帖保持版本中立；只在版本已公开发布后写入版本号。

## 已核实事实表

- 产品：`Marker`
- 分类：轻量级桌面屏幕标注
- 平台：Windows 与 macOS
- 许可：MIT；免费开源
- 隐私：本地优先、无需账号、无遥测或云端依赖
- 安装包：约 1.5 MB
- 核心工作流：快捷键进入绘制，`V` 隐藏/显示标注，`W` 白板，`Esc` 退出
- 工具：画笔、荧光笔、激光笔、箭头、矩形、椭圆、直线、橡皮擦、文字、序号
- 官网 / 更新服务器：https://marker.aomenero.com/
- 源码：https://github.com/AomeNero/marker
- 下载：https://github.com/AomeNero/marker/releases/latest

> 注：Microsoft Store / WinGet / Scoop 为上游项目渠道，本定制版未收录，请勿在推广中引用。

## 当前发布帖 — v1.0.1

发布：2026-08-30

### 中文

Marker v1.0.1 已发布，提供 MSI / NSIS 安装包与 Windows 绿色免安装版。

绿色版下载 zip、解压、直接运行即可；配置保存在程序旁边的 `data\` 目录，不写 AppData。安装版内置自托管自动更新（marker.aomenero.com），关于页可检查更新。

Marker 是一款约 1.5 MB 的开源屏幕标注工具，支持白板模式和全键盘操作，可用于讲课、演示、会议与录屏。

https://github.com/AomeNero/marker/releases/tag/v1.0.1

## 定位

Marker 是一款轻量级开源屏幕标注工具，面向演示、教学、会议与录屏。它从快捷键启动，可在任意应用上绘制，并支持一键隐藏/恢复标注——需要操作下层应用时按 `V` 隐藏，讲完再按恢复。

## 一句话简介

- 轻量开源的屏幕标注工具，支持白板模式。
- 在桌面任意位置绘制，按 `V` 隐藏标注、操作下层应用，再按恢复。
- 为想要快速标注、不想套用重型演示套件的人准备的键盘优先小工具。
- 1.5 MB 的 Tauri 应用，集屏幕标注与白板模式于一体。

## 短描述

Marker 是一个小巧的开源桌面屏幕标注应用。按下快捷键，在任意应用上绘制；需要操作下层屏幕时按 `V` 隐藏标注，讲解需要补标注时再按恢复。支持 10 种标注工具、白板模式、全键盘快捷键，提供 Windows 与 macOS 构建。

## 长描述

Marker 是基于 Tauri v2、Rust、Vue 与 Canvas 构建的轻量级屏幕标注工具，专为需要现场讲解的人设计：教师、培训师、会议主持、演示者、教程作者与开发者。

应用静默运行在系统托盘。按 `Alt+G` 进入标注模式，在任意桌面应用上绘制，用数字键切换工具；要操作下层应用时按 `V` 隐藏标注，再按恢复，全程不退出会话。

Marker 还包含白板模式、撤销/重做、文字、箭头、形状、橡皮擦模式、角度吸附、复制到剪贴板、可展开工具栏面板，以及跨会话内容保留。

官方下载：

- GitHub Releases：https://github.com/AomeNero/marker/releases/latest
- 官网 / 更新服务器：https://marker.aomenero.com/
- 源码：https://github.com/AomeNero/marker

## Product Hunt 草稿

### 名称

Marker

### 标语

轻量级开源屏幕标注，白板模式加持

### 话题

Productivity、Developer Tools、Education、Open Source、Meetings

### 描述

Marker 是一个微小的开源桌面应用，用于在演示、教学、会议与录屏时在屏幕上绘制。标注可随时一键隐藏 / 恢复，方便操作下层应用后继续讲解。

### 作者评论

大家好 Product Hunt！我做 Marker 是因为想要一个不像重型演示套件的快速屏幕标注工具。

它运行在托盘，从快捷键唤醒，可以在任意桌面应用上绘制。需要操作下层应用时按 `V` 隐藏标注，再按恢复，无需退出会话。

亮点：

- 约 1.5 MB 安装包
- 开源、本地优先
- 10 种标注工具，含激光笔与数字序号
- 一键隐藏 / 恢复标注（`V`）
- 白板模式
- 全键盘控制
- Windows 与 macOS 构建

欢迎教师、培训师、教程作者以及 ZoomIt / Epic Pen 类工具的用户反馈。

## Show HN 草稿

标题：

Show HN: Marker – open-source keyboard-first screen annotation

正文：

Hi HN, 我做了 Marker，一个轻量级开源桌面屏幕标注应用。

它面向演示、教学、会议与录屏：按下快捷键，在任意应用上绘制，从键盘切换工具；需要操作下层应用时按 `V` 隐藏标注、再按恢复。这让它在 Epic Pen / ZoomIt 类工作流中可用，同时保持小巧、本地优先、开源。

技术栈：Tauri v2、Rust、Vue 3、TypeScript、Canvas。

下载：https://github.com/AomeNero/marker/releases/latest
源码：https://github.com/AomeNero/marker

特别希望收到关于多显示器行为、macOS 数位板输入，以及键盘优先工作流是否自然的反馈。

## V2EX 草稿

标题：

Marker：约 1.5 MB 的开源屏幕标注工具

正文：

大家好，我做了一个轻量级屏幕标注工具 Marker。

Marker 是一个开源、轻量、快捷键优先的桌面标注工具。按快捷键进入标注，可以在任意应用上画线、箭头、矩形、文字、序号，也支持白板模式。

我自己最常用的是「隐藏标注」：需要点一下下层软件时按 `V`，全部标注暂时隐藏，操作完再按 `V` 恢复。适合讲课、演示、录屏、远程会议里那种「边标注边操作软件」的场景。

特点：

- 安装包约 1.5 MB
- 开源，本地使用，不需要账号
- 支持画笔、荧光笔、激光笔、箭头、矩形、椭圆、直线、橡皮擦、文字、序号
- 按 `V` 隐藏/恢复标注，支持白板模式
- 工具栏可常驻，也可按 Space 呼出
- 支持 Windows / macOS

GitHub: https://github.com/AomeNero/marker
下载: https://github.com/AomeNero/marker/releases/latest

欢迎反馈多屏、macOS 数位板等实际使用问题。

## X / Twitter 草稿

1.

Marker 是一个微小的开源屏幕标注应用。

在任意桌面应用上绘制 / 标注保持可见 / 按 `V` 隐藏后操作下层应用 / 再按恢复

https://github.com/AomeNero/marker

2.

我做 Marker 是因为希望屏幕标注足够即时：

Alt+G -> 绘制
V -> 隐藏/显示标注
W -> 白板
Esc -> 结束

开源、本地优先、约 1.5 MB。

https://github.com/AomeNero/marker

3.

教师、演示者、教程作者：非常欢迎对 Marker 的反馈。

它是一个带标注显隐、白板模式和全键盘控制的轻量级屏幕标注工具。

https://github.com/AomeNero/marker/releases/latest

## Dev.to / Medium 草稿

标题：

Building a 1.5 MB Screen Annotation Tool with Tauri, Rust, Vue, and Canvas

提纲：

1. 为什么现场演示中屏幕标注工具很重要。
2. 为什么 Marker 是键盘优先的。
3. 覆盖层问题：透明置顶窗口。
4. 简化取舍：为什么用一键隐藏标注替代了穿透模式。
5. 渲染策略：canvas、历史、预览与缓存已完成笔迹。
6. 平台注记：Windows、macOS 与多显示器边界情况。
7. 我希望获得哪些反馈。

结尾 CTA：

如果你教书、录教程、跑演示，或使用 ZoomIt / Epic Pen 类工具，非常欢迎对 Marker 提供反馈。

## 中文技术文章草稿

标题：

我用 Tauri 做了一个约 1.5 MB 的开源屏幕标注工具：Marker

提纲：

1. 为什么要做屏幕标注工具，而不是继续用截图/画图/会议白板。
2. Marker 的核心体验：快捷键进入、键盘切工具、轻量常驻托盘。
3. 隐藏/显示标注：为什么一键隐藏比穿透模式更简单。
4. Tauri 多窗口设计：透明 overlay + 独立工具栏窗口。
5. Canvas 绘制和撤销/重做/白板模式。
6. 体积和性能取舍。
7. 当前希望大家帮忙测试的点：多显示器、macOS 数位板。

## 短视频脚本

### 10 秒：快捷键开始绘制

镜头 1：打开文档或浏览器的桌面。
文字："需要现场讲解？"
动作：按 Alt+G。
镜头 2：Marker 覆盖层出现；画箭头并高亮。
文字："随处绘制，即刻完成。"
结尾卡："Marker - 开源屏幕标注"

### 15 秒：隐藏标注，操作下层

镜头 1：在某应用中圈出一个按钮。
文字："标注留在屏幕上。"
镜头 2：按 V。
动作：标注隐藏后点击下层应用。
文字："隐藏标注，直接操作下层。"
镜头 3：再按 V 恢复并补一条箭头。
文字："再按一下，标注回来。"

### 20 秒：白板模式

镜头 1：进入标注模式。
镜头 2：按 W 切换白板。
动作：用箭头、矩形、文字快速画一个流程图。
镜头 3：按 Ctrl+C 粘贴到聊天/文档。
文字："白板、复制、继续。"

## 截图配文

- "在任意应用上绘制，包括任务栏。"
- "按 V 隐藏标注操作下层应用，再按恢复。"
- "使用浮动工具栏或键盘快捷键。"
- "白板模式，快速讲解与教学。"
- "本地优先、开源、轻量。"

## 对比角度

### Marker vs ZoomIt

Marker 专注于轻量标注与白板模式。ZoomIt 在缩放上依旧出色；需要缩放时 Marker 与系统放大器搭配良好。

### Marker vs Epic Pen

Marker 提供开源、本地优先的替代方案：安装包小、键盘优先控制，以及一键隐藏/恢复标注。

### Marker vs 会议白板

会议白板绑定于会议应用。Marker 可覆盖任意桌面应用工作，无需账号，且绘图保存在本地（不上传云端）。

## 中文社交文案

### 通用短帖

做演示、讲课、录屏时，需要在屏幕上临时圈重点，可以试试 Marker：

- 快捷键一按，直接在任意应用上画
- 按 `V` 隐藏标注，操作完下层软件再按恢复
- 按 `W` 切白板，`Esc` 退出
- 约 1.5 MB，免费开源，不要账号，不传云端
- Windows / macOS

官网：https://marker.aomenero.com/
源码：https://github.com/AomeNero/marker

### 小红书 / 即刻

标题：我把「在屏幕上画重点」做成了一个 1.5 MB 的开源小工具

正文：

平时讲课、开会、录教程，经常需要临时圈一下按钮、画个箭头，但又不想先截图再编辑。

Marker 的用法很简单：按 `Alt+G` 直接在桌面上画；需要点下面的软件时按 `V` 暂时隐藏标注，点完再按恢复；按 `W` 变白板；按 `Esc` 退出。

它是免费开源的，不需要账号，也没有云端上传。Windows 和 macOS 都能用。欢迎拿真实的教学 / 演示场景来试，也欢迎在 GitHub 提需求。

### Bilibili / 抖音 / 视频号标题

- 这个 1.5 MB 的开源工具，让整个桌面都能直接画
- 讲课录屏神器：标注不消失，还能继续点下面的软件
- 免费开源的屏幕画笔：快捷键一按就能标重点

### 知乎回答开头

如果你的需求是「演示过程中直接在任意软件上画重点」，而不是截图后再编辑，我更建议用专门的桌面标注层。Marker 是我做的免费开源方案：约 1.5 MB，支持 Windows / macOS，快捷键进入后可用画笔、荧光笔、激光笔、箭头、形状、文字和序号；需要操作下层软件时按 `V` 隐藏标注，操作完再按恢复。

回答后半段应真实比较 ZoomIt、Epic Pen、gInk 的差异，不要伪装成无利益关系的第三方推荐；明确写「作者自荐」。

## 目录元数据

### 80 字符标语

Open-source screen annotation with whiteboard mode for Windows and macOS.

### 160 字符描述

Draw over any desktop app, hide annotations with V when you need to work underneath, or switch to a whiteboard. Lightweight, local-first, and open source.

### 分类

- Productivity
- Education
- Presentation
- Screen annotation
- Whiteboard
- Developer tools
- Open source

### 关键词

`screen annotation`, `desktop drawing`, `screen marker`, `hide annotations`, `whiteboard`, `presentation`, `teaching`, `screen recording`, `Windows`, `macOS`, `Tauri`

## Newsletter / 编辑外联

主题：Open-source screen annotation app for teaching and demos

Hi {{name}},

我是 Marker 的开发者，一款面向 Windows 与 macOS 的轻量级开源屏幕标注应用。它为现场讲解而生：按下快捷键在任意应用上绘制，需要操作下层应用时按 `V` 隐藏标注、操作完再按恢复。

为什么可能适合 {{publication}}：

- 约 1.5 MB 且本地优先
- MIT 许可，无需账号或云端依赖
- 对教师、培训师、教程作者与演示者实用
- Windows 与 macOS 构建

官网：https://marker.aomenero.com/
源码与下载：https://github.com/AomeNero/marker

如有帮助，我可以提供截图、演示短片或 Tauri/Rust/Vue 实现的技术细节。不预设报道；只是觉得它符合你的读者。

此致
AomeNero

## 评论回复

### 安全吗 / 会上传我的屏幕吗？

Marker 本地运行，不需要账号、遥测或云端上传。源码以 MIT 开源，官方下载见 https://marker.aomenero.com/。

### 为什么不用 ZoomIt？

ZoomIt 很出色，尤其缩放。Marker 专注跨平台屏幕绘制、白板模式，以及一键隐藏/恢复标注。

### 为什么不用 Epic Pen？

Epic Pen 是成熟的商业选择。Marker 是小巧、开源、本地优先的替代方案，键盘优先控制。哪个更好取决于你更看重商业功能集还是可审阅的开源工具。

### Linux？

Marker 目前支持 Windows 与 macOS。不要承诺 Linux 时间点；把感兴趣的用户引导到 issue 列表。

## UTM 规范

渠道允许时用官网作为公开链接。只在显得不突兀的地方加 UTM：

`https://marker.aomenero.com/?utm_source={{channel}}&utm_medium={{format}}&utm_campaign=evergreen_2026`

示例：

- Product Hunt: `utm_source=producthunt&utm_medium=launch`
- Reddit: `utm_source=reddit&utm_medium=community`
- Bilibili: `utm_source=bilibili&utm_medium=video`
- 知乎: `utm_source=zhihu&utm_medium=answer`
- Newsletter 外联: `utm_source={{publication}}&utm_medium=editorial`

在偏好裸仓库链接的技术社区，不要给 GitHub 源码 URL 加 UTM。

## 发布守则

1. 声明作者是开发者本人。
2. 绝不购买票数、星标、评论或伪造互动。
3. 不要批量复制相同文案；按社区定制用例。
4. 先给演示或有用的解释，再给一次链接。
5. 只在有意义的版本或新教程时重发，不要为小修小补重发。
6. 前 24 小时内回复每个实质性提问。
