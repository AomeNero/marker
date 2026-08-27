export type DiagnosticLevel = 'info' | 'warn' | 'error'

export interface DiagnosticEvent {
  ts: number
  level: DiagnosticLevel
  category: string
  message: string
  detail?: Record<string, unknown>
}
