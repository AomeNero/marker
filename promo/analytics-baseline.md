# Marker 推广数据基线

采集时间：2026-07-22（Asia/Shanghai，**上游项目数据**）

> 说明：以下仓库与流量数据来自上游 AomeNero/marker 项目在 2026-07-22 的历史快照。本定制版仓库为新建仓库，数据从零开始；发起推广战役前请先用本文件的结构刷新一份自己的基线。

用此文件衡量推广带来的增量。每次大型推广前刷新基线。

## 仓库

| 指标 | 基线（上游 2026-07-22） |
| :--- | ---: |
| GitHub stars | 710 |
| Forks | 38 |
| Watchers | 3 |
| Open issues | 11 |
| Open pull requests | 1 |
| Release 附件下载总量 | 9,520 |

GitHub Release 下载数按附件计，不等于独立用户数。一次安装可能下载多个文件或版本。

## GitHub 流量

GitHub 流量 API 在采集时报告的可用滚动周期：

| 指标 | 总量 | 独立 |
| :--- | ---: | ---: |
| 仓库浏览 | 6,000 | 2,086 |
| 仓库克隆 | 1,236 | 302 |

最新的完整日值截至 2026-07-20。不要把不完整窗口当作同等周期对比。

## 已确认的公开存在（上游）

- 上游官网：https://marker.cn/
- GitHub：https://github.com/AomeNero/marker
- Microsoft Store：https://apps.microsoft.com/detail/9n6623x973jv
- V2EX：https://www.v2ex.com/t/1204012
- Reddit r/tauri：https://www.reddit.com/r/tauri/comments/1sjh08d/marker_a_lightweight_screen_annotation_tool/
- Reddit r/vuejs：https://www.reddit.com/r/vuejs/comments/1slvt7f/marker_a_lightweight_screen_annotation_tool/
- 掘金技术文章（2026-07-22 提交，审核中）：https://juejin.cn/spost/7664877035786600458

## 定制版的公开存在（2026-08-30 起）

- 官网 / 更新服务器：https://marker.aomenero.com/
- GitHub 仓库：https://github.com/AomeNero/marker（定制版 master，自 2026-08-30 起推送）
- GitHub Release：https://github.com/AomeNero/marker/releases/tag/v1.0.1

## 战役度量

每次推广战役记录：

1. 时间戳与准确的公开 URL。
2. 使用的 UTM source / medium / campaign。
3. 发帖前即刻的星标与下载基线。
4. +24h、+7d、+30d 的流量、星标、下载、issue 与评论。
5. 定性反馈主题及其推动的产品改动。

## UTM 模板

`https://marker.aomenero.com/?utm_source={{source}}&utm_medium={{medium}}&utm_campaign=evergreen_2026`

推荐 source 取值：`producthunt`、`hackernews`、`reddit`、`bilibili`、`zhihu`、`juejin`、`sspai`、`newsletter`。
