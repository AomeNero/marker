import { describe, it, expect, beforeEach } from 'vitest'
import {
  clearDiagnosticEvents,
  getDiagnosticEventCount,
  getDiagnosticEvents,
  logActionEvent,
  logDiagnostic,
  logSessionEvent,
} from './diagnosticEvents'

describe('diagnosticEvents', () => {
  beforeEach(() => {
    clearDiagnosticEvents()
  })

  it('stores and returns events', () => {
    logDiagnostic('copy', 'copyScreen invoked', { reason: 'keyboard' })
    expect(getDiagnosticEventCount()).toBe(1)
    expect(getDiagnosticEvents()[0]?.category).toBe('copy')
  })

  it('logSessionEvent uses session category', () => {
    logSessionEvent('overlay mode changed', { from: 'hidden', to: 'drawing' })
    expect(getDiagnosticEvents()[0]?.category).toBe('session')
  })

  it('logActionEvent uses action category', () => {
    logActionEvent('undo', { reason: 'keyboard' })
    expect(getDiagnosticEvents()[0]?.category).toBe('action')
  })

  it('trims buffer to max size', () => {
    for (let i = 0; i < 520; i++) {
      logDiagnostic('pointer', `event-${i}`)
    }
    expect(getDiagnosticEventCount()).toBe(500)
    expect(getDiagnosticEvents()[0]?.message).toBe('event-20')
  })
})
