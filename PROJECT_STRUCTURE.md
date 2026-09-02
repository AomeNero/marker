# Marker 项目结构说明（清理参考）

> 生成于 2026-08-28 · 针对 AomeNero 自用定制版（v1.0.1）
> 技术栈：Tauri v2 + Vue 3 + TypeScript + Vite（前端）｜Rust（后端）

## 目录树总览

```
marker/
├── src/                  前端源码（Vue + TS，含测试）
├── src-tauri/            Rust 后端源码 + Tauri 配置 + 图标
├── assets/               MSIX 打包用 logo + 商店素材（构建引用）
├── public/               网页静态资源（favicon）
├── scripts/              构建 / 发布 / 校验脚本
├── docs/                 官网源码（19MB，主体是图片）+ 上游设计文档
├── promo/                上游作者的营销文案资料
├── packaging/            第三方包管理器发布配置（homebrew/scoop/winget）
├── .cursor/              Cursor 编辑器的规则和技能（AI 协作配置）
├── .husky/               git 提交钩子
├── .vscode/              VS Code 编辑器配置
├── .cargo/               Cargo 构建配置（输出目录指向根 target/）
├── dist/                 前端构建产物（vite 生成）
├── target/               Rust 构建产物 + 安装包（cargo 生成，当前约数 GB）
├── node_modules/         npm 依赖
└── （根目录若干配置文件，见下表）
```

---

## 一、根目录文件

| 文件 | 用途 | 清理建议 |
|------|------|----------|
| `package.json` / `package-lock.json` | npm 依赖与脚本定义 | ✅ 保留 |
| `index.html` | 应用入口 HTML（overlay/toolbar/settings 共用） | ✅ 保留 |
| `vite.config.ts` / `vitest.config.ts` | 前端构建 / 测试配置 | ✅ 保留 |
| `tsconfig.json` / `tsconfig.node.json` | TypeScript 配置 | ✅ 保留 |
| `.gitignore` / `.gitattributes` | git 忽略规则 / 换行符规范 | ✅ 保留 |
| `.npmrc` | npm 强制校验 Node 版本 | ✅ 保留 |
| `.nvmrc` / `.node-version` | Node 版本锁定（nvm/asdf 等工具读取） | ✅ 保留 |
| `.prettierrc` / `.prettierignore` | 代码格式化配置 | ✅ 保留 |
| `commitlint.config.js` | 提交信息规范校验（配合 husky） | ✅ 保留 |
| `AGENTS.md` | AI 协作索引（指向 .cursor 文档） | ⚠️ 若删 .cursor 需同步改 |
| `CONTRIBUTING.md` | 贡献者指南（面向上游开源社区） | ✂️ 自用可删 |
| `PRIVACY.md` | 隐私声明（面向应用发布） | ⚠️ 不发布可删 |
| `SECURITY.md` | 安全漏洞报告指引（面向开源社区） | ✂️ 自用可删 |
| `LICENSE` | MIT 许可证 | ⚠️ 法律文件，建议保留 |
| `README.md` / `README_zh.md` | 项目说明（已更新为 Alt 快捷键） | ✅ 保留 |
| `appxmanifest.xml` | Microsoft Store MSIX 打包清单 | ⚠️ 不做商店发布可删（连带 `scripts/build-msix.sh` 和 `package.json` 的 `build:msix`） |

## 二、核心源码（全部保留）

### `src/` — 前端

| 路径 | 内容 |
|------|------|
| `main.ts` / `App.vue` | 入口，按 URL hash 路由到 overlay / toolbar / settings 三窗口 |
| `components/DrawingOverlay.vue` | **核心**：画布、绘制交互、快捷键、光标（2600+ 行） |
| `components/ToolToolbar.vue` | 一行式工具栏（本次改版主体） |
| `components/ToolbarWindow.vue` | 工具栏独立窗口壳（与 overlay 的状态桥接） |
| `components/TextBox.vue` | 文字输入框 |
| `components/SettingsView.vue` + `settings/` | 设置窗口（通用/关于/诊断三个标签页） |
| `composables/` | 绘图引擎（useDrawing）、几何/渲染、快捷键、主题、tooltip 等 |
| `constants/` | 颜色/工具/笔粗档位/印章/激光等常量 |
| `utils/` | 工具栏定位、拖拽模式、橡皮模式等纯逻辑（均带测试） |
| `i18n/` | 中英文语言包（en.ts / zh-CN.ts） |
| `types/` | TS 类型定义 |
| `test/` | vitest 全局 setup |
| `style.css` | 全局样式与设计 token（含深/浅主题变量） |
| `data/` | **空目录**（赞助数据删除后残留） → ✂️ 可删 |

### `src-tauri/` — Rust 后端

| 路径 | 内容 |
|------|------|
| `src/lib.rs` / `main.rs` | 入口：托盘、窗口管理、事件路由、全局快捷键 |
| `src/overlay.rs` | overlay/toolbar 窗口生命周期、停靠定位 |
| `src/overlay_windows.rs` | **多显示器编排**：一屏一 overlay 窗口、拓扑 diff、热插拔 watcher、光标屏路由 |
| `src/timeline.rs` | 全局撤销时间线（跨屏 Ctrl+Z 的轻量 op 记录，clear 折叠） |
| `src/config.rs` | config.json 读写与归一化（快捷键/笔粗档位等） |
| `src/win32.rs` | Win32 API：显示器工作区、透明度、无焦点置顶 |
| `src/monitor.rs` | 显示器边界查询（按调用窗口/光标屏参数化） |
| `src/commands.rs` | 前端可调用的 IPC 命令（含工具栏动作转发、时间线命令） |
| `src/theme.rs` / `macos*.rs` / `single_instance_win.rs` / `portable.rs` 等 | 主题跟随、macOS 适配、单实例、便携版标记 |
| `icons/` | 应用全套图标（ico/icns/png + NSIS/WiX 安装器图） |
| `tauri.conf.json` | Tauri 主配置（窗口、打包、updater） |
| `tauri.sign.conf.json` | 更新签名配置（无私钥时用不上） → ⚠️ 可删（连带 `build:sign` 脚本） |
| `capabilities/` / `gen/` / `build.rs` / `Cargo.*` | Tauri 权限与构建配置 |

## 三、构建脚本 `scripts/`

| 脚本 | 用途 | 清理建议 |
|------|------|----------|
| `build.sh` | **本地手动编译入口**（免 updater 签名） | ✅ 保留 |
| `build-portable.sh` | 便携版 zip 打包 | ✅ 自用便携版保留 |
| `build-msix.sh` | MSIX 商店包打包 | ⚠️ 随 appxmanifest 一起决定 |
| `check-node.mjs` / `check-lock-engines.mjs` | Node 版本校验 | ✅ 保留 |
| `release.mjs` / `lib/version.mjs` | GitHub 正式发布流程（打 tag/生成更新清单） | ✂️ 不发布可删（连带 npm scripts） |
| `consolidate-github-release.mjs` | 合并 release 资产 | ✂️ 同上 |
| `merge-updater-json.mjs`(+test) | 自动更新清单合并 | ✂️ 不做 updater 可删 |
| `export-store-screenshots.mjs` | 导出商店截图 | ✂️ 可删 |
| `generate-cert.ps1` | 生成签名证书 | ✂️ 可删 |

## 四、上游遗留资产（清理重点）

| 路径 | 大小 | 内容 | 清理建议 |
|------|------|------|----------|
| `docs/assets/` | **19MB** | 官网展示图（hero/功能截图/GIF/微信群二维码） | ✂️ 连同官网可删 |
| `docs/`（其余） | ~1MB | 官网源码（index/help/i18n/CNAME=marker.cn 域名绑定） | ✂️ 不维护官网可整体删除 |
| `docs/superpowers/` | 120K | 上游开发时的设计文档/计划（specs+plans，共 9 份 md） | ✂️ 历史资料，可删或归档 |
| `promo/` | 68K | 营销文章、SEO 关键词、发布清单、数据追踪表 | ✂️ 可删 |
| `packaging/` | 12K | homebrew/scoop/winget 第三方安装源配置 | ✂️ 不做分发自用可删 |
| `.cursor/` | 45K | Cursor 编辑器 AI 规则/技能（你现在用 Claude Code） | ⚠️ 可删，但 AGENTS.md 引用了它（需同步更新）；内容对未来开发仍有参考价值，建议保留或迁移 |

## 五、生成物 / 缓存（可删但会重建）

| 路径 | 说明 |
|------|------|
| `target/` | Rust 构建缓存 + 安装包（**数 GB 大头**）。删除后下次构建全量重编约 5-8 分钟 |
| `dist/` | 前端构建产物，每次构建自动重建，已 git 忽略 |
| `node_modules/` | npm 依赖，`npm install` 可恢复 |

---

## 推荐清理方案

### 保守清理（约省 19MB，零风险）
```bash
rm -rf docs promo packaging src/data
```

### 彻底清理（含发布流程，约省 20MB + 简化维护）
在保守方案基础上：
1. 删 `scripts/` 中 5 个发布脚本 + `package.json` 里对应 scripts（`release`、`release:check`、`build:msix`、`build:all`、`build:sign`）
2. 删 `tauri.sign.conf.json`、`appxmanifest.xml`、`scripts/build-msix.sh`
3. 删 `CONTRIBUTING.md`、`SECURITY.md`（`PRIVACY.md` 视是否发布定）
4. 若删 `.cursor/`：同步精简 `AGENTS.md` 中对它的引用

### 深度清理（释放磁盘）
```bash
cargo clean --release   # 或整个删 target/（下次全量重编）
```

> ⚠️ 注意：删除任何被 `package.json` scripts 或 `tauri.conf.json` 引用的文件后，需同步清理引用，否则对应命令报错。动手前建议先 `git commit` 一次，删错了可以恢复。
