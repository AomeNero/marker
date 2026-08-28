<script setup lang="ts">
import { useI18n } from '../../i18n'

const { t } = useI18n()

const APP_VERSION = __APP_VERSION__
</script>

<template>
  <div class="mt-auto pl-5 pr-3 ui-divider-h settings-sidebar-footer">
    <p class="settings-sidebar-footer__line">
      <span class="settings-sidebar-footer__version-btn" :aria-label="`v${APP_VERSION}`">v{{ APP_VERSION }}</span>
      <span class="settings-sidebar-footer__sep" aria-hidden="true">·</span>
      <span class="settings-sidebar-footer__open">{{ t('settings.sidebarOpenSource') }}</span>
      <span class="settings-sidebar-footer__sep" aria-hidden="true">·</span>
      <span class="settings-sidebar-footer__license">MIT</span>
    </p>
  </div>
</template>
