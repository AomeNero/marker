<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { useI18n } from '../../i18n'
import { useUpdater } from '../../composables/useUpdater'
import { resolvePortableMode } from '../../utils/portable'

const { t } = useI18n()

const portableMode = ref<boolean | null>(null)
const appVersion = ref('')
const { status, newVersion, progress, checkForUpdate, downloadAndInstall } = useUpdater()

const updateUiExpanded = computed(
  () =>
    portableMode.value === false &&
    (status.value === 'available' ||
      status.value === 'downloading' ||
      status.value === 'checking' ||
      status.value === 'up-to-date'),
)

onMounted(async () => {
  portableMode.value = await resolvePortableMode()
  try {
    appVersion.value = await getVersion()
  } catch (error) {
    console.error('Failed to read app version:', error)
  }
})

async function openUrl(url: string) {
  try {
    await invoke('open_url', { url })
  } catch (error) {
    console.error('Failed to open URL:', error)
  }
}
</script>

<template>
  <div class="flex-1 flex flex-col min-h-0 overflow-hidden items-center px-7 py-8 w-full">
    <div class="shrink-0 flex flex-col items-center w-full max-w-85">
      <div
        class="settings-app-icon w-16 h-16 rounded-2xl flex items-center justify-center"
        :class="updateUiExpanded ? 'mb-3' : 'mb-4'"
      >
        <svg class="w-8 h-8" viewBox="0 0 1254 1254" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
          <defs>
            <linearGradient
              id="brand-ring-about"
              data-name="brand ring"
              x1="-1657.64"
              y1="1167.84"
              x2="-1656.31"
              y2="1166.52"
              gradientTransform="translate(1167956.22 740010.45) rotate(-2.06) scale(688 -669)"
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0" stop-color="#ff2c20" />
              <stop offset=".52" stop-color="#ff3327" />
              <stop offset="1" stop-color="#ff3b2c" />
            </linearGradient>
            <linearGradient
              id="brand-nib-about"
              data-name="brand nib"
              x1="-1652.42"
              y1="1168.03"
              x2="-1651.63"
              y2="1169.08"
              gradientTransform="translate(390539.54 299548.25) scale(235.76 -256.08)"
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0" stop-color="#092a67" />
              <stop offset=".52" stop-color="#0f377a" />
              <stop offset="1" stop-color="#173f84" />
            </linearGradient>
          </defs>
          <path
            fill="none"
            stroke="url(#brand-ring-about)"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="120"
            d="M835.33,247.4c-69.81-51.85-154.98-77.32-239.12-74.31-249.72,8.97-444.68,218.45-435.71,468.18,8.98,249.72,221.32,448.65,473.75,439.58,252.43-9.07,468.72-230.19,459.26-493.48l-1.12-31.22"
          />
          <path
            fill="url(#brand-nib-about)"
            fill-rule="evenodd"
            d="M1052.99,159.07c11.61-12.46,23.25-13.28,34.87-2.49l108.36,109.6c10.8,10.8,11.2,21.58,1.24,32.39l-117.07,124.55c-4.14,4.99-9.97,8.72-17.44,11.2l-123.3,32.39c-16.6,4.14-26.16-1.65-28.65-17.44l22.42-158.18c.84-4.99,3.33-9.55,7.47-13.7l112.09-118.33h.01ZM970.78,313.51h38.61v51.06h51.06v37.36l-29.9,6.22-41.1,8.72-28.65-32.39,9.97-70.98h.01Z"
          />
        </svg>
      </div>

      <h1 class="font-semibold settings-text-heading tracking-wide mb-1">Marker</h1>
      <p class="settings-text-body text-center" :class="updateUiExpanded ? 'mb-3' : 'mb-4'">
        {{ t('about.tagline') }}
      </p>

      <p
        v-if="appVersion"
        class="settings-text-subtle text-center text-xs m-0"
        :class="updateUiExpanded ? 'mb-2' : 'mb-3'"
      >
        {{ t('about.currentVersion') }} {{ appVersion }}
      </p>

      <div class="flex flex-col items-center gap-2 w-full" :class="updateUiExpanded ? 'mb-3' : 'mb-6'">
        <p v-if="portableMode === true" class="settings-text-subtle text-center text-xs leading-relaxed m-0 px-2">
          {{ t('about.portableUpdateHint') }}
        </p>

        <template v-else-if="portableMode === false">
          <button
            v-if="status === 'idle' || status === 'error'"
            class="settings-btn-accent-outline px-4 py-1.5 rounded-lg cursor-pointer"
            @click="checkForUpdate()"
          >
            {{ status === 'error' ? t('about.updateError') : t('about.checkUpdate') }}
          </button>

          <span v-else-if="status === 'checking'" class="settings-text-subtle flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 animate-spin" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            {{ t('about.checking') }}
          </span>

          <div v-else-if="status === 'available'" class="flex flex-col items-center gap-1.5">
            <span class="settings-text-accent">{{ t('about.updateAvailable', { version: newVersion }) }}</span>
            <button
              class="settings-btn-accent-primary px-4 py-1.5 rounded-lg cursor-pointer"
              @click="downloadAndInstall()"
            >
              {{ t('about.installAndRestart') }}
            </button>
          </div>

          <div v-else-if="status === 'downloading'" class="flex flex-col items-center gap-1.5 w-full max-w-50">
            <span class="settings-text-subtle">{{ t('about.downloading', { progress: String(progress) }) }}</span>
            <div class="settings-progress-track w-full h-1.5 rounded-full overflow-hidden">
              <div
                class="settings-progress-fill h-full rounded-full transition-all duration-300"
                :style="{ width: progress + '%' }"
              />
            </div>
          </div>

          <span v-else-if="status === 'up-to-date'" class="settings-status-success">
            {{ t('about.upToDate') }}
          </span>
        </template>
      </div>
    </div>

    <div class="w-full max-w-85 shrink-0">
      <div class="settings-card w-full overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 ui-divider-b settings-row-hover transition-colors">
          <span class="settings-text-row-key">{{ t('about.license') }}</span>
          <span class="settings-text-value">MIT License</span>
        </div>
        <button
          class="w-full flex items-center justify-between px-4 py-3 ui-divider-b settings-row-hover-strong transition-colors cursor-pointer bg-transparent border-x-0 border-t-0"
          @click="openUrl('https://github.com/AomeNero/marker')"
        >
          <span class="settings-text-row-key">GitHub</span>
          <span class="flex items-center gap-1.5 settings-text-accent-link">
            AomeNero/marker
            <svg
              class="w-3 h-3 opacity-50"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
          </span>
        </button>
        <button
          class="w-full flex items-center justify-between px-4 py-3 settings-row-hover-strong transition-colors cursor-pointer bg-transparent border-none"
          @click="openUrl('https://github.com/AomeNero/marker/issues')"
        >
          <span class="settings-text-row-key">{{ t('about.feedback') }}</span>
          <svg
            class="w-3.5 h-3.5 settings-text-dim"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="9 18 15 12 9 6" />
          </svg>
        </button>
      </div>
    </div>

    <p class="shrink-0 mt-auto pt-6 pb-1 settings-text-footer tracking-wide">
      &copy; 2026 AomeNero &middot; Open Source
    </p>
  </div>
</template>
