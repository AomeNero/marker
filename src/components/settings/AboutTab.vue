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
        <svg class="w-8 h-8" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
          <path
            fill="#13227a"
            d="M203.62,77.49l-15.23,15.23c-1.55,1.55-4.06,1.55-5.61,0l-36.66-36.66c-1.55-1.55-1.55-4.06,0-5.61l15.23-15.23c6.18-6.18,16.22-6.18,22.43,0l19.85,19.85c6.21,6.18,6.21,16.22,0,22.43h-.01ZM133.04,63.52l-86.73,86.73-7,40.13c-.96,5.42,3.77,10.11,9.18,9.18l40.13-7.03,86.73-86.73c1.55-1.55,1.55-4.06,0-5.61l-36.66-36.66c-1.59-1.55-4.1-1.55-5.65,0h0ZM80.17,142.82c-1.82-1.82-1.82-4.72,0-6.54l50.86-50.86c1.82-1.82,4.72-1.82,6.54,0s1.82,4.72,0,6.54l-50.86,50.86c-1.82,1.82-4.72,1.82-6.54,0ZM68.24,170.6h15.85v11.99l-21.3,3.73-10.27-10.27,3.73-21.3h11.99v15.85Z"
          />
          <path
            fill="#d81e06"
            d="M196.33,202.78c1.66-2.62,6.09-3.81,9.9-2.67s5.55,4.18,3.9,6.8c-6.06,9.54-14.29,14.89-25.02,14.89-6.58,0-10.46-1.89-14.84-5.85l-2.33-2.22c-3.05-2.97-4.47-3.71-7.14-3.71-2.43,0-3.99.6-7.19,2.91l-1.78,1.31c-7.07,5.29-12.14,7.57-21.13,7.57-8.33,0-13.16-1.92-19.41-6.41l-1.96-1.45c-4.17-3.18-5.79-3.92-8.83-3.92-2.77,0-4.4.66-7.39,3.09l-2.01,1.72c-5.63,4.8-9.91,6.99-17.44,6.99-11.24,0-20.38-5.18-27.55-14.56-1.92-2.53-.5-5.65,3.18-6.97s8.23-.34,10.15,2.19h0c4.82,6.33,9.58,9.02,14.21,9.02,1.24,0,2.19-.35,3.97-1.71l2.72-2.25c6.27-5.37,11.29-7.83,20.16-7.83,7.87,0,12.45,1.83,18.47,6.19l1.89,1.4c4.43,3.37,6.25,4.2,9.83,4.2,3.26,0,5.17-.71,8.78-3.29l1.91-1.42c6.63-4.97,11.27-7.08,19.42-7.08s13.08,2.27,18.23,6.95l3.02,2.89,1.11.98.43.34.71.43c.3.14.54.19.8.19,3.44,0,7.3-2.51,11.22-8.69h0v-.03Z"
          />
        </svg>
      </div>

      <h1 class="font-semibold settings-text-heading tracking-wide mb-1">Marker</h1>
      <p class="settings-text-body text-center" :class="updateUiExpanded ? 'mb-3' : 'mb-4'">
        {{ t('about.tagline') }}
      </p>

      <p v-if="appVersion" class="settings-text-subtle text-center text-xs m-0" :class="updateUiExpanded ? 'mb-2' : 'mb-3'">
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
