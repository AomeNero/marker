# 分发与发布检查单

最后调研：2026-07-22（上游项目基线）；2026-08-30 按自托管定制版更新

> 说明：本文件源自上游项目的推广记录。标注「上游」的数据与渠道属于 AomeNero/marker 上游公开发布的历史情况，本定制版仓库（新立仓库）尚未积累自己的数据；引用前请先核对当前仓库实际情况。

## 当前基线

- GitHub：710 stars、38 forks（**上游数据**）
- GitHub Release 附件下载：累计 9,520 次（**上游数据**）
- GitHub 流量（采集时 14 天滚动）：6,000 浏览 / 2,086 独立访客（**上游数据**）
- 官网：上游为 `https://marker.cn/`；本定制版为自托管 `https://marker.aomenero.com/`
- Microsoft Store / WinGet / Scoop Extras：**上游渠道**，定制版未收录
- 已有社区帖：V2EX、Reddit r/tauri、Reddit r/vuejs（**上游发布**，勿重复引用为本项目成果）
- 未覆盖的高意向渠道：Product Hunt、Show HN、AlternativeTo、Homebrew Cask

指标是有日期的基线，不是常青营销文案。Release 下载数按文件计，不等于独立用户数。

## 可立即执行

### 自托管官网 / 更新服务器

- 状态：已上线（定制版）
- 更新端点：`https://marker.aomenero.com/latest.json`（Tauri Updater，minisign 签名校验）
- 静态文件：`updater-dist/`（MSI + latest.json），由 `npm run build:release` 自动生成后上传站点根目录（完整发布流程见 `docs/releasing.md`）
- 遗留事项：可选提交 `https://marker.aomenero.com/sitemap.xml` 到 Google Search Console 与 Bing 站长平台

### GitHub Releases

- 定制版首个发布：https://github.com/AomeNero/marker/releases/tag/v1.0.1
- 附件：MSI、NSIS 安装包、绿色版 zip、latest.json

### Product Hunt

使用 `promo/launch-kit.md` 中的草稿。

官方说明已核对：

- Product Hunt 允许开发者发布自己的产品，登录后点击 Post 按钮。
- 建议发帖后以作者评论开启讨论。
- 可能存在账号注册时长限制。

链接：

- 发帖流程帮助：https://help.producthunt.com/en/articles/479557-how-to-post-a-product
- 新手入门：https://help.producthunt.com/en/articles/2305333-getting-started
- 精选指南：https://help.producthunt.com/en/articles/9883485-product-hunt-featuring-guidelines

### Show HN

使用 `promo/launch-kit.md` 中的草稿。

重要规则：提交的内容必须让用户真的能试用，不能只给落地页。URL 使用 GitHub 仓库或最新 Release。

链接：

- 指南：https://news.ycombinator.com/showhn.html
- 提交：https://news.ycombinator.com/submit

### AlternativeTo

使用以下元数据：

- 名称：`Marker`
- 平台：Windows、macOS
- 许可：Open Source / MIT
- 标签：screen annotation、desktop annotation、whiteboard、drawing、productivity、presentation
- 网站：GitHub 仓库或自托管官网
- 描述：使用 `promo/launch-kit.md` 中的短描述
- 对标条目：ZoomIt、Epic Pen、gInk、ppInk、ScreenBrush

官方 FAQ 说明从用户菜单使用 "Suggest new application"，并提交平台、许可、描述、标签等字段。

链接：https://alternativeto.net/faq/

## 包管理器扩展（上游渠道，定制版未收录）

### Scoop

状态：上游已收录于 Scoop Extras。

- Manifest：https://github.com/ScoopInstaller/Extras/blob/master/bucket/marker.json
- 安装：`scoop bucket add extras && scoop install marker`

上游 manifest 带自动版本检查；每次 Release 后关注其更新 PR，勿重复开 PR。定制版如需进入 Scoop，需要另行提交自己的 manifest。

### Homebrew Cask

上游草稿位于 `packaging/homebrew/marker.rb`。

提交前需在 macOS 测试：

```bash
brew install --cask ./packaging/homebrew/marker.rb
brew uninstall --cask marker
```

官方参考：

- 添加软件：https://docs.brew.sh/Adding-Software-to-Homebrew
- Cask 手册：https://docs.brew.sh/Cask-Cookbook
- 可接受标准：https://docs.brew.sh/Acceptable-Casks

## 可选目录

### SourceForge

适合需要额外开源项目页与下载镜像的场景。SourceForge 要求开源项目、摘要、图标、描述与截图。

链接：

- 创建项目：https://sourceforge.net/p/forge/documentation/Create%20a%20New%20Project/
- 推广项目：https://sourceforge.net/p/forge/documentation/Promoting%20your%20project/

### Softpedia

对曝光有潜在价值，但提交需谨慎，并确保 GitHub/官方渠道指向清晰。优先选择只链接官方下载、不二次打包的目录。

## 已有覆盖 — 勿重复

（以下均为**上游项目**的历史发布）

- V2EX：https://www.v2ex.com/t/1204012
- V2EX 后续帖：https://www.v2ex.com/t/1222261
- Reddit r/tauri：https://www.reddit.com/r/tauri/comments/1sjh08d/marker_a_lightweight_screen_annotation_tool/
- Reddit r/vuejs：https://www.reddit.com/r/vuejs/comments/1slvt7f/marker_a_lightweight_screen_annotation_tool/
- 第三方中文下载页（搜索发现）：https://www.cr173.com/soft/1668222.html（版本过时；勿为其二次打包背书）
- 自然收录文章：https://myqqjd.com/84790.html

只有在有实质功能故事、教程或里程碑时才在既有社区再次发帖；小版本修复不要重发。

## 30 天发布顺序

### 第 1 周 — 转化基础

1. 部署官网元数据与常青文案（自托管站）。
2. 核对公开 Release 与全部官方下载链接。
3. 录制一段 10–15 秒「按 V 隐藏/恢复标注」演示与一段白板演示。
4. 以官网为规范链接提交 AlternativeTo 与 Product Hunt。
5. 以 GitHub 仓库为 URL、附技术性作者评论提交 Show HN。

### 第 2 周 — 中文创作者渠道

1. 发布一条 Bilibili 演示视频，并复用竖版切片到 视频号 / 抖音 / 小红书。
2. 在知乎发布透明署名的「作者自荐」回答，比较 Marker 与 ZoomIt / gInk。
3. 在掘金发布 Tauri 实现文章；可选以规范链接跨发 CSDN / OSCHINA。
4. 演示视频与最新截图就绪后再投放少数派。

### 第 3 周 — 开源分发

1. （上游渠道）确认 Scoop manifest 在最新 Release 后自动更新。
2. 尽量在 Apple Silicon 与 Intel 上测试 Homebrew cask 后提交。
3. 只在 Marker 明确符合范围的精选 awesome 列表提交。
4. 以开源 / 架构角度发 r/opensource；勿复用 r/tauri 文案。

### 第 4 周 — 媒体与 SEO 沉淀

1. 向 10 个软件类 newsletter / 教育科技编辑发送个性化外联。
2. 在自托管官网发布「Marker vs ZoomIt vs Epic Pen」「如何在任意应用上标注」文章。
3. 以披露身份在知乎、Reddit、Stack Exchange 回答相关问题，提供有用的对比内容。
4. 复盘 UTM 流量、星标、下载与留存；加倍投入质量流量最好的两个渠道。

## 渠道优先级矩阵

| 优先级 | 渠道 | 最佳角度 | 所需素材 | 状态 |
| :--- | :--- | :--- | :--- | :--- |
| P0 | Product Hunt | 微型开源应用 + 白板模式 | 3–5 张图、作者评论 | 未提交 |
| P0 | Show HN | 透明覆盖层架构 | 仓库 URL、技术帖 | 未提交 |
| P0 | AlternativeTo | 开源 Epic Pen / ZoomIt 替代 | 列表元数据 | 未提交 |
| P0 | Bilibili | 15 秒前后对比演示 | 横版视频 | 未发布 |
| P1 | r/opensource | 本地优先 MIT 项目 | 定制帖 | 未发 |
| P1 | 掘金 / dev.to | Tauri 覆盖层实现 | 技术文章 | 草稿就绪 |
| P1 | 少数派 | 真实教学/演示工作流 | 精修截图 + 视频 | 待投放 |
| P2 | SourceForge | 开源目录曝光 | 项目页素材 | 可选 |
| P2 | Softpedia | 软件发现 | 仅官方链接提交 | 可选 |
| 上游已做 | 知乎 / Scoop / Store | — | — | 上游渠道，勿重复引用 |

## 成功指标

按渠道在 24 小时、7 天、30 天跟踪：

- 有效官网会话（UTM）
- GitHub 星标与关注者
- Release 附件下载数（按文件计，非用户数）
- Issue 质量与首次贡献者
- 视频完播率与收藏/分享
- 落地页到官方下载的转化

避免只优化曝光量；能带来下载、有用反馈或回头用户的渠道才更重要。
