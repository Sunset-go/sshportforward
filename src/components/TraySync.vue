<script setup lang="ts">
// 语言同步组件：在应用启动及语言切换时，把当前 locale 同步到后端，
// 供原生对话框（关闭询问、连接提醒）使用正确的文案。
import { watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePrefs } from '../composables/usePrefs';

const { locale, t } = usePrefs();

async function sync() {
  try {
    await Promise.all([
      invoke('set_ui_locale', { locale: locale.value }),
      invoke('set_tray_texts', {
        show: t('tray.show'),
        closeToTray: t('tray.closeToTray'),
        quit: t('tray.quit'),
      }),
    ]);
  } catch (e) {
    // 托盘未就绪时忽略
    console.warn('托盘同步失败（可能是启动早期）', e);
  }
}

sync(); // 挂载时立即同步一次
watch(locale, sync);
watch(() => t('tray.show'), sync);
</script>

<template>
  <!-- 纯逻辑组件，无 DOM 输出 -->
  <div style="display:none" aria-hidden="true"></div>
</template>
