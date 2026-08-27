# WinGet Distribution

Marker is already discoverable through the Microsoft Store source:

```powershell
winget search --name Marker --accept-source-agreements
winget install --id 9N6623X973JV --source msstore --accept-source-agreements
```

Current result checked on 2026-07-03:

```text
Name      ID           Version   Source
Marker  9N6623X973JV Unknown   msstore
```

Recommended next step: keep the Microsoft Store listing as the canonical WinGet path unless a separate direct-installer manifest is needed in `microsoft/winget-pkgs`.

If a direct WinGet community manifest is submitted later, use (replace `X.Y.Z` / hashes with the current GitHub Release before submitting):

- Package identifier: `ifer47.Marker`
- Package name: `Marker`
- Publisher: `ifer47`
- Homepage: `https://marker.cn/`
- License: `MIT`
- Release URL: `https://github.com/ifer47/marker/releases/tag/vX.Y.Z`
- Installer URL: `https://github.com/ifer47/marker/releases/download/vX.Y.Z/Marker_X.Y.Z_x64_zh-CN.msi`
- Installer SHA256: _(compute from the published MSI)_

Example values for **v2.2.0** (verify against the live release assets before use):

- Release URL: `https://github.com/ifer47/marker/releases/tag/v2.2.0`
- Installer URL: `https://github.com/ifer47/marker/releases/download/v2.2.0/Marker_2.2.0_x64_zh-CN.msi`
