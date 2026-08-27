export type DefaultEntryMode = 'screen' | 'whiteboard'

export const DEFAULT_ENTRY_MODE_OPTIONS: DefaultEntryMode[] = ['screen', 'whiteboard']

export function resolveDefaultEntryMode(general?: { defaultEntryMode?: DefaultEntryMode }): DefaultEntryMode {
  return general?.defaultEntryMode ?? 'screen'
}

/** Whether entering whiteboard should wipe existing drawings. */
export function shouldClearWhiteboardOnEntry(options: {
  whiteboardPreserveDrawings: boolean
  preserveDrawings: boolean
  fromDefaultEntry: boolean
  hasDrawings: boolean
}): boolean {
  if (options.whiteboardPreserveDrawings) return false
  if (options.fromDefaultEntry && options.preserveDrawings && options.hasDrawings) return false
  return true
}
