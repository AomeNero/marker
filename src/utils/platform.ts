/** True when running in a macOS environment (Tauri webview or Safari-like UA). */
export function isMacOS(): boolean {
  if (typeof navigator === 'undefined') return false
  const p = navigator.platform ?? ''
  const ua = navigator.userAgent ?? ''
  return /Mac|iPhone|iPad|iPod/i.test(p) || /Mac OS/i.test(ua)
}

/** WKWebView on macOS often ignores `cursor: none` on transparent overlay windows. */
export const MAC_HIDDEN_CURSOR =
  "url('data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7'), none"

/** Hide the OS cursor in the overlay webview while the custom pen cursor is shown. */
export function setMacOverlaySystemCursorHidden(hidden: boolean): void {
  if (!isMacOS() || typeof document === 'undefined') return
  const value = hidden ? MAC_HIDDEN_CURSOR : ''
  document.documentElement.style.cursor = value
  document.body.style.cursor = value
}
