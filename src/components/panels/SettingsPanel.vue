<script setup lang="ts">
// 设置页签（由原弹窗改造为页签面板）：主题/语言/最小化到托盘/开机自启。
import { onActivated } from 'vue';
import { useSettings } from '../../composables/useSettings';
import { t } from '../../composables/usePrefs';
import type { Locale, ThemeMode } from '../../composables/useSettings';

const { settings, load, patch } = useSettings();

const themeOptions: { value: ThemeMode; labelKey: string }[] = [
  { value: 'dark', labelKey: 'settings.dark' },
  { value: 'light', labelKey: 'settings.light' },
];
const localeOptions: { value: Locale; labelKey: string }[] = [
  { value: 'zh', labelKey: 'settings.zh' },
  { value: 'en', labelKey: 'settings.en' },
];

// 激活时从后端刷新，确保 minimizeToTray 与 DB 最新一致（托盘弹窗可能已改过）
onActivated(() => void load());
</script>

<template>
  <div class="settings-panel">
    <section class="card">
      <div class="card-head">
        <h2>{{ t('settings.title') }}</h2>
      </div>
      <div class="body">
        <div class="row">
          <label class="label">{{ t('settings.theme') }}</label>
          <select class="sel" :value="settings.theme" @change="patch({ theme: ($event.target as HTMLSelectElement).value as ThemeMode })">
            <option v-for="opt in themeOptions" :key="opt.value" :value="opt.value">{{ t(opt.labelKey) }}</option>
          </select>
        </div>
        <div class="row">
          <label class="label">{{ t('settings.locale') }}</label>
          <select class="sel" :value="settings.locale" @change="patch({ locale: ($event.target as HTMLSelectElement).value as Locale })">
            <option v-for="opt in localeOptions" :key="opt.value" :value="opt.value">{{ t(opt.labelKey) }}</option>
          </select>
        </div>
        <div class="row">
          <label class="label">{{ t('settings.minimizeToTray') }}</label>
          <input
            class="chk"
            type="checkbox"
            :checked="settings.minimizeToTray"
            @change="patch({ minimizeToTray: ($event.target as HTMLInputElement).checked })"
          />
        </div>
        <div class="row">
          <label class="label">{{ t('settings.autoStart') }}</label>
          <input
            class="chk"
            type="checkbox"
            :checked="settings.autoStart"
            @change="patch({ autoStart: ($event.target as HTMLInputElement).checked })"
          />
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-panel {
  flex: 1 1 0;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.card {
  flex: 0 0 auto;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  box-shadow: var(--panel-shadow);
  overflow: hidden;
}
.card-head {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.card-head h2 { margin: 0; font-size: 13px; font-weight: 600; color: var(--text); }
.body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.label { font-size: 13px; color: var(--text); }
.sel {
  height: 30px;
  padding: 0 10px;
  background: var(--input);
  border: 1px solid var(--border);
  border-radius: var(--radius-input);
  color: var(--text);
  font-size: 13px;
  min-width: 160px;
}
.sel:focus { outline: none; border-color: var(--blue); box-shadow: 0 0 0 3px var(--accent-soft); }
.chk { width: 16px; height: 16px; accent-color: var(--blue); cursor: pointer; }
</style>
