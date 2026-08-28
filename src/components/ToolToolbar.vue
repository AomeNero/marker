<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted, computed, watch } from 'vue'
import { Undo2, Redo2, Trash2, Layout, Copy, MousePointer2, X, ChevronDown } from '@lucide/vue'
import type { Tool } from '../composables/useDrawing'
import { useI18n } from '../i18n'
import {
  SELECT_TOOL_DEF,
  WIDTH_PRESET_LABEL_KEYS,
  getWidthPresets,
  PEN_GROUP_TOOLS,
  PEN_GROUP_DEFAULT,
  SHAPE_GROUP_TOOLS,
  SHAPE_GROUP_DEFAULT,
  toolbarGroupOf,
  toolDefOf,
  TOOL_ICON_MAP,
} from '../constants/tools'
import { COLOR_ROWS } from '../constants/colors'
import {
  TEXT_OUTLINE_WIDTH_PRESETS,
  normalizeTextOutline,
  resolveTextOutlineColor,
  resolveAutoTextOutlineColor,
} from '../constants/textOutline'
import { saveToolbarPosition, clampToolbarWindowPosition } from '../utils/toolbarPosition'
import {
  fitToolbarWindow,
  measureToolbarPanelHeight,
  fetchOverlayMonitorBounds,
  refreshToolbarWindowScreenOrigin,
  repositionToolbarAfterHeightChange,
  getToolbarPanelHeight,
  TOOLBAR_PANEL_WIDTH,
} from '../utils/toolbarWindow'
import type { MonitorLogicalBounds } from '../utils/toolbarPosition'
import { LogicalPosition } from '@tauri-apps/api/dpi'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { TextOutlineStyle } from '../composables/drawingTypes'
import { isMacOS } from '../utils/platform'

const { t } = useI18n()

const props = defineProps<{
  pinned: boolean
  standaloneWindow?: boolean
  currentTool: Tool
  currentColor: string
  lineWidth: number
  textOutline: TextOutlineStyle
  whiteboardMode: boolean
  penetrationMode?: boolean
  canUndo: boolean
  canRedo: boolean
  canClear: boolean
  pointerX: number
  pointerY: number
}>()

const emit = defineEmits<{
  selectTool: [tool: Tool]
  selectColor: [color: string]
  updateLineWidth: [width: number]
  updateTextOutline: [textOutline: TextOutlineStyle]
  close: []
  undo: []
  redo: []
  clearAll: []
  toggleWhiteboard: []
  copy: []
  togglePenetration: []
  exitDrawing: []
  panelHover: [hovering: boolean]
  panelDrag: [dragging: boolean]
}>()

const eraserDef = computed(() => ({ ...toolDefOf('eraser'), label: t('tools.eraser') }))
const textDef = computed(() => ({ ...toolDefOf('text'), label: t('tools.text') }))
const selectToolLabel = computed(() => t(`tools.${SELECT_TOOL_DEF.id}`))
const selectToolTitle = computed(() => `${selectToolLabel.value} (${SELECT_TOOL_DEF.key})`)
const colors = COLOR_ROWS
const simpleColors = computed(() => colors[0] ?? [])
const widths = computed(() =>
  getWidthPresets().map((v, i) => ({ value: v, label: t(`widths.${WIDTH_PRESET_LABEL_KEYS[i] ?? 'm'}`) })),
)
const outlineWidths = computed(() =>
  TEXT_OUTLINE_WIDTH_PRESETS.map((_, i) => ({ value: TEXT_OUTLINE_WIDTH_PRESETS[i], label: t(`widths.${WIDTH_PRESET_LABEL_KEYS[i] ?? 'm'}`) })),
)
const outlinePreviewColor = computed(() => resolveTextOutlineColor(props.textOutline, props.currentColor))
const customOutlineColor = computed(() => normalizeTextOutline(props.textOutline).color)

// --- collapsed tool groups -------------------------------------------------
// Which flyout is open (mutually exclusive): a tool-group menu or the settings panel.
type MenuKind = 'pen' | 'shape' | 'settings'
const openMenu = ref<MenuKind | null>(null)

/** Last-used sub-tool shown on each collapsed group button. */
const lastPenTool = ref<Tool>(PEN_GROUP_DEFAULT)
const lastShapeTool = ref<Tool>(SHAPE_GROUP_DEFAULT)

watch(
  () => props.currentTool,
  (tool) => {
    const group = toolbarGroupOf(tool)
    if (group === 'pen') lastPenTool.value = tool
    else if (group === 'shape') lastShapeTool.value = tool
    // Switching to a standalone tool (eraser/text/select) closes any open flyout.
    if (!group && openMenu.value && openMenu.value !== 'settings') openMenu.value = null
  },
  { immediate: true },
)

const penGroupTools = computed(() => PEN_GROUP_TOOLS.map((id) => ({ ...toolDefOf(id), label: t(`tools.${id}`) })))
const shapeGroupTools = computed(() => SHAPE_GROUP_TOOLS.map((id) => ({ ...toolDefOf(id), label: t(`tools.${id}`) })))
const penGroupIcon = computed(() => TOOL_ICON_MAP[lastPenTool.value])
const shapeGroupIcon = computed(() => TOOL_ICON_MAP[lastShapeTool.value])
const penGroupActive = computed(() => toolbarGroupOf(props.currentTool) === 'pen')
const shapeGroupActive = computed(() => toolbarGroupOf(props.currentTool) === 'shape')
const penGroupTitle = computed(() => `${t(`tools.${lastPenTool.value}`)} (${toolDefOf(lastPenTool.value).key})`)
const shapeGroupTitle = computed(() => `${t(`tools.${lastShapeTool.value}`)} (${toolDefOf(lastShapeTool.value).key})`)

function toggleMenu(menu: MenuKind) {
  openMenu.value = openMenu.value === menu ? null : menu
}

function selectTool(tool: Tool) {
  emit('selectTool', tool)
  openMenu.value = null
}

function selectColor(color: string) {
  emit('selectColor', color)
  openMenu.value = null
}

/** Any completed action in the settings flyout returns to the bar. */
function onCopyClick() {
  emit('copy')
  openMenu.value = null
}

function updateWidth(width: number) {
  emit('updateLineWidth', width)
}

function updateTextOutline(patch: Partial<TextOutlineStyle>) {
  emit('updateTextOutline', normalizeTextOutline({ ...props.textOutline, ...patch }))
}

function updateCustomTextOutlineColor(color: string) {
  updateTextOutline({ enabled: true, colorMode: 'fixed', color })
}

/** White ✓ on dark swatches; black ✓ on light ones. */
function needsWhiteCheck(color: string): boolean {
  return resolveAutoTextOutlineColor(color) === '#FFFFFF'
}

const panelRef = ref<HTMLDivElement | null>(null)
/** Panel width follows content (flyouts are narrower than the bar, so it stays constant). */
const panelW = ref(TOOLBAR_PANEL_WIDTH)
const positioned = ref(false)
const isDragging = ref(false)

let syncSizeGeneration = 0
let syncSizeRafId: number | null = null
let panelResizeObserver: ResizeObserver | null = null

async function syncStandaloneWindowSize() {
  if (!props.standaloneWindow || !panelRef.value) return
  const generation = ++syncSizeGeneration
  await nextTick()
  if (generation !== syncSizeGeneration || !panelRef.value) return
  const width = Math.ceil(panelRef.value.getBoundingClientRect().width)
  panelW.value = width
  let oldHeight = getToolbarPanelHeight()
  try {
    const win = getCurrentWindow()
    const [size, scale] = await Promise.all([win.outerSize(), win.scaleFactor()])
    const logicalH = size.toLogical(scale).height
    if (logicalH >= 64) oldHeight = logicalH
  } catch {
    // keep cache
  }
  const height = measureToolbarPanelHeight(panelRef.value)
  await fitToolbarWindow(width, height)
  await repositionToolbarAfterHeightChange(oldHeight, height, { persist: props.pinned })
  if (isMacOS()) {
    await refreshToolbarWindowScreenOrigin()
  }
  syncPanelHover()
}

let lastPanelHoverEmitted: boolean | null = null

/** Avoid redundant cross-window hover events that flicker the overlay pen cursor. */
function emitPanelHover(hovering: boolean) {
  if (lastPanelHoverEmitted === hovering) return
  lastPanelHoverEmitted = hovering
  emit('panelHover', hovering)
}

function probePanelHoverAtScreen(_screenX: number, _screenY: number) {
  // Standalone toolbar: screen-space probe disagrees with pointer enter/leave on
  // multi-monitor / mixed-DPI setups and spuriously hides the overlay pen cursor.
  // Hover is driven by pointer enter/leave on the panel; the overlay clears stale
  // hover when the pointer moves on the canvas (see DrawingOverlay).
}

function scheduleSyncStandaloneWindowSize() {
  if (!props.standaloneWindow) return
  if (syncSizeRafId !== null) cancelAnimationFrame(syncSizeRafId)
  syncSizeRafId = requestAnimationFrame(() => {
    syncSizeRafId = null
    void syncStandaloneWindowSize()
  })
}

function syncPanelHover() {
  if (!panelRef.value || !positioned.value) {
    emitPanelHover(false)
    return
  }
  // macOS standalone: client coords go stale once the cursor returns to the overlay window.
  // Hover is driven by pointer enter/leave on the panel element.
  if (props.standaloneWindow && isMacOS()) return
  const r = panelRef.value.getBoundingClientRect()
  const inside =
    props.pointerX >= r.left && props.pointerX <= r.right && props.pointerY >= r.top && props.pointerY <= r.bottom
  emitPanelHover(inside)
}

function initPosition() {
  nextTick(() => {
    positioned.value = true
    if (props.standaloneWindow && isMacOS()) {
      void refreshToolbarWindowScreenOrigin()
    }
    syncPanelHover()
    void syncStandaloneWindowSize()
  })
}

let cachedPanelH = 46
let lastScreenX = 0
let lastScreenY = 0
let dragRafId: number | null = null
let dragPointerId: number | null = null
let captureTarget: HTMLElement | null = null
let windowDragOffset = { x: 0, y: 0 }
let dragMonitorBounds: MonitorLogicalBounds | null = null

function clampStandaloneWindowPosition(left: number, top: number, panelH: number) {
  if (!dragMonitorBounds) {
    return { left, top }
  }
  return clampToolbarWindowPosition(left, top, panelW.value, panelH, dragMonitorBounds)
}

function scheduleDragUpdate() {
  if (dragRafId !== null) return
  dragRafId = requestAnimationFrame(() => {
    dragRafId = null
    if (!isDragging.value) return
    const panelH = panelRef.value ? measureToolbarPanelHeight(panelRef.value) : cachedPanelH
    const rawLeft = lastScreenX - windowDragOffset.x
    const rawTop = lastScreenY - windowDragOffset.y
    const clamped = clampStandaloneWindowPosition(rawLeft, rawTop, panelH)
    void getCurrentWindow().setPosition(new LogicalPosition(clamped.left, clamped.top))
  })
}

/** Single click-through button toggles both ways (drawing ⇄ penetration). */
function onPenetrationModeClick() {
  if (props.whiteboardMode) return
  emit('togglePenetration')
}

function startDrag(e: PointerEvent) {
  if (e.button !== 0) return
  isDragging.value = true
  emitPanelHover(true)
  emit('panelDrag', true)
  dragPointerId = e.pointerId
  captureTarget = e.currentTarget as HTMLElement
  captureTarget.setPointerCapture(e.pointerId)
  e.preventDefault()
  lastScreenX = e.screenX
  lastScreenY = e.screenY
  windowDragOffset = { x: e.clientX, y: e.clientY }
  dragMonitorBounds = null
  void fetchOverlayMonitorBounds().then((bounds) => {
    dragMonitorBounds = bounds
  })
}

function onPointerMove(e: PointerEvent) {
  if (!isDragging.value) return
  if (dragPointerId !== null && e.pointerId !== dragPointerId) return
  lastScreenX = e.screenX
  lastScreenY = e.screenY
  scheduleDragUpdate()
}

function releaseDragCapture() {
  if (captureTarget && dragPointerId !== null) {
    try {
      captureTarget.releasePointerCapture(dragPointerId)
    } catch {
      // pointer already released
    }
  }
  captureTarget = null
  dragPointerId = null
}

function stopDrag(e?: PointerEvent) {
  if (!isDragging.value) return
  if (e && dragPointerId !== null && e.pointerId !== dragPointerId) return
  isDragging.value = false
  if (dragRafId !== null) {
    cancelAnimationFrame(dragRafId)
    dragRafId = null
  }
  releaseDragCapture()
  emit('panelDrag', false)
  dragMonitorBounds = null
  void (async () => {
    const win = getCurrentWindow()
    const [pos, scale] = await Promise.all([win.outerPosition(), win.scaleFactor()])
    const logical = pos.toLogical(scale)
    saveToolbarPosition(logical.x, logical.y, true)
    await invoke('raise_toolbar')
    if (isMacOS()) {
      await refreshToolbarWindowScreenOrigin()
    }
  })()
  syncPanelHover()
}

function onPanelPointerLeave() {
  if (isDragging.value) return
  emitPanelHover(false)
}

/** Clicking outside the toolbar window (e.g. drawing on the canvas) collapses flyouts. */
function closeMenuOnBlur() {
  openMenu.value = null
}

function stopDragOnBlur() {
  stopDrag()
}

defineExpose({ syncPanelHover, syncStandaloneWindowSize, probePanelHoverAtScreen })

watch(
  () => props.pinned,
  () => {
    openMenu.value = null
    positioned.value = false
    initPosition()
  },
)

watch(openMenu, () => {
  // Flyout open/close changes panel height — resize the window and keep the bar anchored.
  scheduleSyncStandaloneWindowSize()
})

watch(
  () => [props.pointerX, props.pointerY] as const,
  () => {
    syncPanelHover()
  },
)

onMounted(() => {
  initPosition()
  window.addEventListener('pointermove', onPointerMove)
  window.addEventListener('pointerup', stopDrag)
  window.addEventListener('pointercancel', stopDrag)
  window.addEventListener('blur', stopDragOnBlur)
  window.addEventListener('blur', closeMenuOnBlur)
  if (props.standaloneWindow && typeof ResizeObserver !== 'undefined') {
    panelResizeObserver = new ResizeObserver(() => scheduleSyncStandaloneWindowSize())
    nextTick(() => {
      if (panelRef.value) panelResizeObserver?.observe(panelRef.value)
    })
  }
})

onUnmounted(() => {
  syncSizeGeneration += 1
  if (syncSizeRafId !== null) {
    cancelAnimationFrame(syncSizeRafId)
    syncSizeRafId = null
  }
  panelResizeObserver?.disconnect()
  panelResizeObserver = null
  emitPanelHover(false)
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', stopDrag)
  window.removeEventListener('pointercancel', stopDrag)
  window.removeEventListener('blur', stopDragOnBlur)
  window.removeEventListener('blur', closeMenuOnBlur)
  stopDrag()
  if (dragRafId !== null) {
    cancelAnimationFrame(dragRafId)
    dragRafId = null
  }
})
</script>

<template>
  <div class="block w-fit h-fit overflow-hidden">
    <div
      ref="panelRef"
      class="relative"
      style="width: fit-content"
      @pointerenter="emitPanelHover(true)"
      @pointerleave="onPanelPointerLeave"
    >
      <!-- Flyouts render above the bar (in-flow so the window grows upward). -->
      <div
        v-if="openMenu === 'pen' || openMenu === 'shape'"
        class="overlay-panel overlay-panel-surface mb-1.5 px-2 py-2"
      >
        <div class="flex items-center gap-1">
          <button
            v-for="tool in openMenu === 'pen' ? penGroupTools : shapeGroupTools"
            :key="tool.id"
            type="button"
            class="overlay-flyout-tool"
            :class="currentTool === tool.id ? 'overlay-tool-btn--active' : 'overlay-tool-btn'"
            :aria-label="`${tool.label} (${tool.key})`"
            :aria-pressed="currentTool === tool.id"
            :title="`${tool.label} (${tool.key})`"
            @click="selectTool(tool.id)"
          >
            <component :is="tool.icon" :size="16" />
            <span class="text-[10.5px] leading-none font-sans">{{ tool.label }}</span>
          </button>
        </div>
      </div>

      <div
        v-else-if="openMenu === 'settings'"
        class="overlay-panel overlay-panel-surface mb-1.5 px-3 py-2.5"
        style="width: 260px"
      >
        <!-- Colors -->
        <div class="flex items-center justify-between pb-2.5">
          <button
            v-for="color in simpleColors"
            :key="color"
            class="size-7 p-0 border-none rounded-full bg-transparent cursor-pointer relative flex items-center justify-center transition-transform duration-120"
            :class="currentColor === color ? 'scale-[1.18]' : 'hover:scale-[1.18]'"
            :title="t(`colors.${color}`)"
            @click="selectColor(color)"
          >
            <span
              class="w-5.5 h-5.5 rounded-full color-swatch-ring transition-[border-color] duration-120"
              :class="{ 'color-swatch-ring--active': currentColor === color }"
              :style="{ backgroundColor: color }"
            />
            <span
              v-if="currentColor === color"
              class="absolute text-[11px] font-bold pointer-events-none"
              :class="
                needsWhiteCheck(color)
                  ? 'text-white [text-shadow:0_0_2px_rgba(0,0,0,0.5)]'
                  : 'text-black [text-shadow:0_0_2px_rgba(255,255,255,0.5)]'
              "
              >✓</span
            >
          </button>
          <label
            class="size-7 p-0 border-none rounded-full bg-transparent cursor-pointer relative flex items-center justify-center transition-transform duration-120 hover:scale-[1.18]"
            :title="t('panel.customColor')"
            :aria-label="t('panel.customColor')"
          >
            <input
              type="color"
              class="absolute w-0 h-0 opacity-0 pointer-events-none"
              :value="currentColor"
              @input="selectColor(($event.target as HTMLInputElement).value)"
            />
            <span
              class="w-5.5 h-5.5 rounded-full color-picker-ring pointer-events-none flex items-center justify-center shadow-[inset_0_0_2px_rgba(0,0,0,0.5)]"
              style="
                background: conic-gradient(
                  from 90deg,
                  #ff0000,
                  #ff8000,
                  #ffff00,
                  #80ff00,
                  #00ff00,
                  #00ff80,
                  #00ffff,
                  #0080ff,
                  #0000ff,
                  #8000ff,
                  #ff00ff,
                  #ff0080,
                  #ff0000
                );
              "
            >
              <span
                class="text-white text-[11px] leading-none font-light"
                style="text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6)"
                >+</span
              >
            </span>
          </label>
        </div>

        <!-- Text outline (text tool only) -->
        <div v-if="currentTool === 'text'" class="pt-2.5 ui-divider-h">
          <div class="flex items-center justify-between mb-2">
            <span class="text-[11px] font-semibold overlay-text-section tracking-[0.5px] font-sans">{{
              t('panel.textOutline')
            }}</span>
            <button
              type="button"
              class="px-2.5 py-1 rounded-md ui-segment text-[10.5px] leading-none transition-colors duration-120"
              :class="{ 'ui-segment--active': textOutline.enabled }"
              :aria-pressed="textOutline.enabled"
              @click="updateTextOutline({ enabled: !textOutline.enabled })"
            >
              {{ textOutline.enabled ? t('panel.textOutlineOn') : t('panel.textOutlineOff') }}
            </button>
          </div>
          <div class="flex items-center gap-1.5 mb-2">
            <button
              type="button"
              class="flex-1 h-8 rounded-md ui-segment text-[10.5px] leading-none transition-colors duration-120"
              :class="{ 'ui-segment--active': textOutline.enabled && textOutline.colorMode === 'auto' }"
              :aria-pressed="textOutline.enabled && textOutline.colorMode === 'auto'"
              :title="t('panel.textOutlineAuto')"
              @click="updateTextOutline({ enabled: true, colorMode: 'auto' })"
            >
              {{ t('panel.textOutlineAuto') }}
            </button>
            <label
              class="flex-1 h-8 rounded-md ui-segment text-[10.5px] leading-none transition-colors duration-120 cursor-pointer flex items-center justify-center gap-1.5 relative overflow-hidden"
              :class="{ 'ui-segment--active': textOutline.enabled && textOutline.colorMode === 'fixed' }"
              :title="t('panel.textOutlineCustom')"
            >
              <input
                type="color"
                class="absolute w-0 h-0 opacity-0 pointer-events-none"
                :value="customOutlineColor"
                @input="updateCustomTextOutlineColor(($event.target as HTMLInputElement).value)"
              />
              <span
                class="w-3.5 h-3.5 rounded-full color-swatch-ring color-swatch-ring--compact transition-[border-color] duration-120"
                :class="{
                  'color-swatch-ring--active': textOutline.enabled && textOutline.colorMode === 'fixed',
                }"
                :style="{ backgroundColor: customOutlineColor }"
              />
              <span>{{ t('panel.textOutlineCustom') }}</span>
            </label>
          </div>
          <div class="flex items-center gap-2">
            <div class="flex flex-1 gap-1">
              <button
                v-for="w in outlineWidths"
                :key="w.value"
                type="button"
                class="group flex-1 flex items-center justify-center h-8 border-none rounded-lg cursor-pointer transition-all duration-120"
                :class="
                  textOutline.enabled && textOutline.width === w.value
                    ? 'overlay-width-btn--active'
                    : 'overlay-width-btn'
                "
                :title="w.label"
                @click="updateTextOutline({ enabled: true, width: w.value })"
              >
                <span
                  class="w-[70%] rounded-full transition-transform duration-120 group-hover:scale-x-110"
                  :class="
                    textOutline.enabled && textOutline.width === w.value
                      ? 'overlay-width-line--active'
                      : 'overlay-width-line'
                  "
                  :style="{
                    height: Math.max(1.5, w.value * 1.1) + 'px',
                    backgroundColor: textOutline.enabled ? outlinePreviewColor : undefined,
                  }"
                />
              </button>
            </div>
          </div>
        </div>

        <!-- Copy -->
        <div class="pt-2.5 ui-divider-h">
          <button
            type="button"
            class="w-full flex items-center justify-center gap-1.5 h-8 border-none rounded-lg cursor-pointer overlay-tool-btn text-[11px] font-sans"
            :title="t('toolbar.copy')"
            @click="onCopyClick"
          >
            <Copy :size="14" />
            {{ t('toolbar.copy') }}
          </button>
        </div>
      </div>

      <!-- One-line toolbar bar -->
      <div class="overlay-panel overlay-panel-surface overlay-panel--standalone">
        <div class="flex items-center gap-0.5 px-1.5 py-2" :class="isDragging ? 'cursor-grabbing' : ''" @mousedown.stop>
          <!-- Drag grip -->
          <div
            class="toolbar-grip cursor-grab active:cursor-grabbing"
            :class="isDragging ? 'cursor-grabbing' : ''"
            title="⠿"
            @pointerdown="startDrag"
          />

          <!-- Tools -->
          <button
            type="button"
            class="overlay-toolbar-action"
            :class="currentTool === 'select' ? 'overlay-toolbar-action--active' : ''"
            :title="selectToolTitle"
            :aria-label="selectToolTitle"
            :aria-pressed="currentTool === 'select'"
            @click="selectTool('select')"
          >
            <component :is="SELECT_TOOL_DEF.icon" :size="15" />
          </button>
          <button
            type="button"
            class="overlay-toolbar-action overlay-toolbar-flyout"
            :class="penGroupActive ? 'overlay-tool-btn--active' : 'overlay-tool-btn'"
            :title="penGroupTitle"
            :aria-label="penGroupTitle"
            :aria-pressed="penGroupActive"
            :aria-expanded="openMenu === 'pen'"
            @click="toggleMenu('pen')"
          >
            <component :is="penGroupIcon" :size="15" />
            <ChevronDown class="toolbar-flyout-caret" :size="9" />
          </button>
          <button
            type="button"
            class="overlay-toolbar-action overlay-toolbar-flyout"
            :class="shapeGroupActive ? 'overlay-tool-btn--active' : 'overlay-tool-btn'"
            :title="shapeGroupTitle"
            :aria-label="shapeGroupTitle"
            :aria-pressed="shapeGroupActive"
            :aria-expanded="openMenu === 'shape'"
            @click="toggleMenu('shape')"
          >
            <component :is="shapeGroupIcon" :size="15" />
            <ChevronDown class="toolbar-flyout-caret" :size="9" />
          </button>
          <button
            v-for="tool in [eraserDef, textDef]"
            :key="tool.id"
            type="button"
            class="overlay-toolbar-action"
            :class="currentTool === tool.id ? 'overlay-tool-btn--active' : 'overlay-tool-btn'"
            :aria-label="`${tool.label} (${tool.key})`"
            :aria-pressed="currentTool === tool.id"
            :title="`${tool.label} (${tool.key})`"
            @click="selectTool(tool.id)"
          >
            <component :is="tool.icon" :size="15" />
          </button>

          <span class="ui-divider-v h-5.5 mx-1.5" />

          <!-- Color + stroke width -->
          <button
            type="button"
            class="overlay-toolbar-action overlay-toolbar-flyout"
            :class="openMenu === 'settings' ? 'overlay-toolbar-action--active' : ''"
            :title="t('panel.colors')"
            :aria-label="t('panel.colors')"
            :aria-expanded="openMenu === 'settings'"
            @click="toggleMenu('settings')"
          >
            <span class="w-4 h-4 rounded-full color-swatch-ring" :style="{ backgroundColor: currentColor }" />
            <ChevronDown class="toolbar-flyout-caret" :size="9" />
          </button>
          <button
            v-for="w in widths"
            :key="w.value"
            type="button"
            class="group overlay-width-btn"
            :class="lineWidth === w.value ? 'overlay-width-btn--active' : ''"
            :title="w.label"
            :aria-pressed="lineWidth === w.value"
            @click="updateWidth(w.value)"
          >
            <span
              class="w-[70%] rounded-full transition-transform duration-120 group-hover:scale-x-110"
              :class="lineWidth === w.value ? 'overlay-width-line--active' : 'overlay-width-line'"
              :style="{ height: Math.max(1.5, w.value * 1.2) + 'px' }"
            />
          </button>

          <span class="ui-divider-v h-5.5 mx-1.5" />

          <!-- Edit actions -->
          <button
            type="button"
            class="overlay-toolbar-action"
            :disabled="!canUndo"
            :title="t('toolbar.undo')"
            :aria-label="t('toolbar.undo')"
            @click="emit('undo')"
          >
            <Undo2 :size="15" />
          </button>
          <button
            type="button"
            class="overlay-toolbar-action"
            :disabled="!canRedo"
            :title="t('toolbar.redo')"
            :aria-label="t('toolbar.redo')"
            @click="emit('redo')"
          >
            <Redo2 :size="15" />
          </button>
          <button
            type="button"
            class="overlay-toolbar-action"
            :disabled="!canClear"
            :title="t('toolbar.clear')"
            :aria-label="t('toolbar.clear')"
            @click="emit('clearAll')"
          >
            <Trash2 :size="15" />
          </button>

          <span class="ui-divider-v h-5.5 mx-1.5" />

          <!-- Mode -->
          <button
            type="button"
            class="overlay-toolbar-action"
            :class="whiteboardMode ? 'overlay-toolbar-action--active' : ''"
            :title="whiteboardMode ? t('toolbar.exitWhiteboard') : t('toolbar.whiteboard')"
            :aria-label="whiteboardMode ? t('toolbar.exitWhiteboard') : t('toolbar.whiteboard')"
            @click="emit('toggleWhiteboard')"
          >
            <Layout :size="15" />
          </button>
          <button
            v-if="standaloneWindow"
            type="button"
            class="overlay-toolbar-action"
            :class="penetrationMode ? 'overlay-toolbar-action--active' : ''"
            :title="t('toolbar.penetrationMode')"
            :aria-label="t('toolbar.penetrationMode')"
            :aria-pressed="!!penetrationMode"
            :disabled="whiteboardMode"
            @click="onPenetrationModeClick"
          >
            <MousePointer2 :size="15" />
          </button>

          <template v-if="standaloneWindow">
            <span class="ui-divider-v h-5.5 mx-1.5" />
            <button
              type="button"
              class="overlay-toolbar-action"
              :title="t('toolbar.exit')"
              :aria-label="t('toolbar.exit')"
              @click="emit('exitDrawing')"
            >
              <X :size="15" />
            </button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Sharper corners for the toolbar bar and its flyouts (panel default is 16px). */
.overlay-panel {
  border-radius: 8px;
}

.overlay-toolbar-action {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 9px;
  cursor: pointer;
  color: var(--ui-control-text);
  background: var(--ui-control-bg);
  transition:
    background 0.12s,
    color 0.12s;
}

.overlay-toolbar-action:hover {
  background: var(--ui-control-bg-strong);
  color: var(--ui-tool-text-hover);
}

.overlay-toolbar-action:disabled {
  opacity: 0.32;
  cursor: default;
  background: var(--ui-bg-subtle-hover);
  color: var(--ui-text-icon);
}

.overlay-toolbar-action:disabled:hover {
  background: var(--ui-bg-subtle-hover);
  color: var(--ui-text-icon);
}

.overlay-toolbar-action--active {
  background: var(--ui-control-bg-strong);
  color: var(--ui-tool-text-hover);
}

/* Compact tools share action geometry; keep tool color tokens. */
.overlay-toolbar-action.overlay-tool-btn {
  background: var(--ui-control-bg-soft);
  color: var(--ui-tool-text);
}

.overlay-toolbar-action.overlay-tool-btn:hover {
  background: var(--ui-control-bg-hover);
  color: var(--ui-tool-text-hover);
}

.overlay-toolbar-action.overlay-tool-btn--active {
  background: var(--ui-accent-bg-active);
  color: var(--ui-tool-text-active);
  box-shadow: inset 0 0 0 1px var(--ui-accent-border);
}

/* Buttons that open a flyout carry a tiny caret. */
.overlay-toolbar-flyout {
  position: relative;
}

.toolbar-flyout-caret {
  position: absolute;
  right: 1px;
  bottom: 0.5px;
  opacity: 0.55;
  pointer-events: none;
}

/* Stroke width buttons in the bar (compact geometry). */
.overlay-width-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 24px;
  height: 30px;
  border: none;
  border-radius: 7px;
  cursor: pointer;
  background: var(--ui-control-bg-soft);
  transition:
    background 0.12s,
    box-shadow 0.12s;
}

.overlay-width-btn:hover {
  background: var(--ui-control-bg-hover);
}

.overlay-width-btn--active {
  background: var(--ui-accent-bg-active);
  box-shadow: inset 0 0 0 1px var(--ui-accent-border);
}

/* Flyout menu rows (icon + label). */
.overlay-flyout-tool {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 10px;
  border: none;
  border-radius: 9px;
  cursor: pointer;
  color: var(--ui-tool-text);
  background: var(--ui-control-bg-soft);
  transition:
    background 0.12s,
    color 0.12s;
}

.overlay-flyout-tool.overlay-tool-btn:hover {
  background: var(--ui-control-bg-hover);
  color: var(--ui-tool-text-hover);
}

.overlay-flyout-tool.overlay-tool-btn--active {
  background: var(--ui-accent-bg-active);
  color: var(--ui-tool-text-active);
  box-shadow: inset 0 0 0 1px var(--ui-accent-border);
}

/* Drag grip at the left end of the bar. */
.toolbar-grip {
  width: 12px;
  height: 30px;
  flex-shrink: 0;
  border-radius: 6px;
  background-image: radial-gradient(currentColor 1px, transparent 1px);
  background-size: 4px 4px;
  background-position: center;
  background-repeat: no-repeat;
  color: var(--ui-text-icon);
  opacity: 0.4;
  margin-right: 2px;
}

.toolbar-grip:hover {
  opacity: 0.8;
}
</style>
