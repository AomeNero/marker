# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |
| < latest | :x:               |

Only the latest release receives security updates. Please upgrade to the newest version.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT** open a public issue
2. Email **** with:
   - A description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

You should receive a response within 72 hours. We will work with you to understand the issue and coordinate a fix before any public disclosure.

## Security Measures

Marker is designed with security in mind:

- **No telemetry** — the app does not collect usage analytics or upload annotations
- **Local-only config** — settings stay on disk under the OS app data directory, or under `data\` for portable builds
- **Optional update network** — installer builds may contact GitHub Releases only when you check for or install updates; portable builds update manually
- **URL allowlist** — the `open_url` command only permits pre-approved domains
- **Memory-only drawings** — annotations exist only in RAM and are never written to disk
- **CSP enforced** — Content Security Policy restricts the webview to local resources only
