# Marker 推广资产指南

最后核对：2026-08-30

## 推荐资产

| 用途 | 资产 | 尺寸 | 备注 |
| :--- | :--- | :--- | :--- |
| 方形中文封面 | `assets/酷图_1080x1080.png` | 1080×1080 | 品牌安全的通用封面 |
| 高分辨率方形 | `assets/酷图_2160x2160.png` | 2160×2160 | 同一通用封面的 2× 版本 |
| 竖版封面 | `assets/招贴画_720x1080.png` | 720×1080 | 适合小红书 / 视频号 / 故事格式 |
| 竖版高分辨率 | `assets/招贴画_1440x2160.png` | 1440×2160 | 高分辨率竖版封面 |
| 真实桌面场景 | `assets/desktop-annotation.png` / `assets/桌面标注场景.png` | 2880×1530 | 由 `scene1-desktop*.html` 重新导出 |
| 快捷键速查 | `assets/shortcuts-overview.png` / `assets/快捷键一览.png` | 2880×1530 | 由商店场景重新导出（10 种工具 / `1-7` + `E` + `T` + `N`） |
| 标注工具网格 | `assets/annotation-tools.png` / `assets/十种标注工具.png` | 2880×1530 | 由商店场景重新导出（10 种工具 / `1-7` + `E` + `T` + `N`） |
| 设置 / 面板 | `assets/settings-panel.png` / `assets/设置面板.png` | 2880×1530 | 由 `scene3-panel*.html` 重新导出 |
| 主视觉 / 品牌 | `assets/Marker.png` / `assets/Marker_en.png` / `docs/assets/hero.png` | 2880×1530 | 由 `scene0-hero*.html` 重新导出 |

> 快捷键速查与工具网格图已于 2026-08-30 按当前键位（激光笔 3、箭头 4、橡皮擦 E、颜色循环 Q/R、V 隐藏标注）重新导出。
> 工具栏相关场景（主视觉 / 设置面板 / 快捷键速查）已于 2026-09-04 按新版一行式工具栏（横条 UI：工具分组 ▾ 弹出、8 色选色盘、5 档线宽）与当前键位重新导出。

未使用的历史设置截图（README/官网未引用；UI 已变化——拖拽模式、诊断页等）：

- `assets/language-switcher.png`
- `assets/preserve-drawings-setting.png`

再次对外推广前，优先从实际运行的程序重新截图。

## 重新导出营销 PNG

源 HTML 位于 `assets/store-screenshots/scene*.html`（按 1920×1080 设计）。导出为 2880×1530 用于 README / 官网 / 商店上架。

辅助脚本：`scripts/export-store-screenshots.mjs`（需要 Playwright Chromium；`npm i -D playwright && npx playwright install chromium` 后运行）。或用 Chrome 以 1920×1080 打开每个 HTML 截取视口，再缩放到 2880×1530。

修改这些 HTML 中的工具 / 快捷键 / 面板文案后，重新导出并覆盖上述 PNG 路径（以及 `docs/assets/*` 中的对应副本）。

## 图集顺序

用于 Product Hunt、目录站与媒体资料包：

1. 真实桌面标注截图。
2. 真实快捷键速查。
3. 真实设置 / 工具截图。
4. 通用品牌封面（酷图 / 招贴画）。

此顺序让差异化行为先于功能列表被看到。
