/** Hit-test whether screen-space pointer coords fall inside a panel rect (toolbar window client space). */
export function isPointerOverPanelRect(
  screenX: number,
  screenY: number,
  windowScreenX: number,
  windowScreenY: number,
  rect: { left: number; top: number; right: number; bottom: number },
): boolean {
  const clientX = screenX - windowScreenX
  const clientY = screenY - windowScreenY
  return clientX >= rect.left && clientX <= rect.right && clientY >= rect.top && clientY <= rect.bottom
}
