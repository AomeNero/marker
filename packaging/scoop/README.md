# Scoop Distribution Notes

Status: published in Scoop Extras.

- Manifest: https://github.com/ScoopInstaller/Extras/blob/master/bucket/marker.json
- Initial inclusion: https://github.com/ScoopInstaller/Extras/pull/18304
- Install after adding Extras: `scoop bucket add extras && scoop install marker`

The upstream manifest currently extracts the signed Windows installer and stores normal Marker configuration outside the Scoop version directory. Switching it to the new portable zip would enable a cleaner archive install but would also change configuration behavior for existing Scoop users. Do not change that packaging path without a migration plan and `persist` testing.

Scoop is strongest when the app has a portable `.zip` release that can be extracted into Scoop's app directory without running a traditional installer.

Windows GitHub Release assets include:

- `Marker_X.Y.Z_x64-setup.exe` (NSIS)
- `Marker_X.Y.Z_x64_zh-CN.msi`
- `Marker_X.Y.Z_x64.msix`
- `Marker_X.Y.Z_x64_portable.zip` (green / portable)

Portable zip contents:

- `Marker.exe`
- `WebView2Loader.dll` (required for windows-gnu builds)
- `marker.portable` marker file (config / logs / WebView data → `./data/`)
- `README.txt`

Portable-build reference for future maintenance:

1. Use `Marker_X.Y.Z_x64_portable.zip` only after migration behavior is agreed.
2. Add `persist` for `data` so portable configuration survives Scoop upgrades.
3. Test migration from an existing installer-based Scoop install.

Draft metadata:

- App name: `marker`
- Description: `Lightweight screen annotation tool with whiteboard mode`
- Homepage: `https://marker.cn/`
- License: `MIT`
- Architecture: `64bit`
- Bin/shortcut target: `Marker.exe`
