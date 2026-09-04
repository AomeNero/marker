# 发布流程（本地）

> 本仓库当前没有 CI 构建流水线，发布完全在本机完成。
> 最后核对：2026-09-04（v1.0.3 实测走通）。

## 前置条件

- `master` 分支、工作区干净；`gh` CLI 已登录（`gh auth status`）
- 更新签名密钥：`~/.tauri/marker-updater-v2.key`（与 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey` 配对，无密码）
  - 路径不同时用环境变量 `MARKER_UPDATER_KEY_FILE` 覆盖
- Windows Authenticode 证书签名：**当前未配置**（`tauri.sign.conf.json` 里的证书指纹不在本机证书存储中，`npm run build:sign` 会失败）。安装包不做证书签名，首次运行可能触发 SmartScreen 提示；不影响自动更新（客户端用 minisign 公钥校验）

## 步骤

1. **版本 bump + 打 tag + 推送**（脚本自动跑全部检查：typecheck / vite build / vitest / oxlint / prettier / cargo fmt）：

   ```bash
   npm run release patch   # 或 minor / major；可先 npm run release:check 预演
   ```

2. **一键构建**（minisign 签名的 tauri build → 便携版 zip → 自动生成 `updater-dist/`）：

   ```bash
   npm run build:release
   ```

   `target/` 产物已存在、只需重新暂存 `updater-dist/` 时：`npm run build:release -- --stage-only`

3. **创建 GitHub Release**（tag 已在第 1 步推送；发布说明手写中文，参考 v1.0.2 / v1.0.3 格式：主要变化 + 下载表）：

   ```bash
   gh release create vX.Y.Z --title "Marker vX.Y.Z" --notes-file notes.md \
     "target/release/bundle/msi/Marker_X.Y.Z_x64_zh-CN.msi" \
     "target/release/bundle/nsis/Marker_X.Y.Z_x64-setup.exe" \
     "target/release/bundle/portable/Marker_X.Y.Z_x64_portable.zip" \
     "updater-dist/latest.json"
   ```

4. **上传更新服务器**：把 `updater-dist/`（新版 MSI + `latest.json`）上传到 `marker.aomenero.com` 站点根目录 —— updater 端点 `https://marker.aomenero.com/latest.json` 指向这里，**跳过此步老客户端检测不到新版本**。

5. **验证**：用旧版本客户端触发「检查更新」，确认能升到新版并正常启动；顺带核验 Release 页四个附件的下载链接。

## 产物一览

| 产物 | 用途 |
| :-- | :-- |
| `target/release/bundle/msi/*.msi` | Windows 安装包（MSI，推荐；updater 下载源） |
| `target/release/bundle/nsis/*-setup.exe` | Windows 安装包（NSIS） |
| `target/release/bundle/portable/*_portable.zip` | 绿色免安装版 |
| `updater-dist/latest.json` | 更新清单（站点与 GitHub Release 各传一份，内容保持一致） |

## 相关文件

- `scripts/release.mjs`（`npm run release`）— 版本 bump / tag / push
- `scripts/build-release.mjs`（`npm run build:release`）— 签名构建 + 便携版 + `updater-dist/` 暂存
- `scripts/build-portable.sh`（`npm run build:portable`）— 便携版打包（已含在 build:release 中）
- `updater-dist/` — gitignore 的站点暂存目录
