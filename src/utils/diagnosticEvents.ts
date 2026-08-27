import { invoke } from '@tauri-apps/api/core'
import type { DiagnosticEvent, DiagnosticLevel } from '../types/diagnostics'

const MAX_EVENTS = 500
const events: DiagnosticEvent[] = []

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export function logDiagnostic(
  category: string,
  message: string,
  detail?: Record<string, unknown>,
  level: DiagnosticLevel = 'info',
): void {
  const event: DiagnosticEvent = {
    ts: Date.now(),
    level,
    category,
    message,
    detail,
  }
  events.push(event)
  if (events.length > MAX_EVENTS) {
    events.splice(0, events.length - MAX_EVENTS)
  }
  if (isTauriRuntime()) {
    void invoke('append_diagnostic_event', { event }).catch(() => {})
  }
}

export function getDiagnosticEvents(): DiagnosticEvent[] {
  return events.slice()
}

export function getDiagnosticEventCount(): number {
  return events.length
}

export function clearDiagnosticEvents(): void {
  events.length = 0
}

export type DiagnosticReason =
  | 'keyboard'
  | 'toolbar'
  | 'global-shortcut'
  | 'tray'
  | 'focus-loss'
  | 'activate'
  | 'unknown'

export function logSessionEvent(
  message: string,
  detail?: Record<string, unknown>,
  level: DiagnosticLevel = 'info',
): void {
  logDiagnostic('session', message, detail, level)
}

export function logActionEvent(
  message: string,
  detail?: Record<string, unknown>,
  level: DiagnosticLevel = 'info',
): void {
  logDiagnostic('action', message, detail, level)
}
