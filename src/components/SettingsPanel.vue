<script setup lang="ts">
// 偏好设置面板：主题/语言/最小化到托盘/开机自启。
// 每项变更经 useSettings.patch 立即应用并防抖落盘；minimizeToTray 经 save_settings
// 桥接到 DB close_to_tray 键。打开时重新 load 以反映托盘/弹窗刚改过的最新值。
import { watch } from 'vue';
import { useSettings } from '../composables/useSettings';
import { t } from '../composables/usePrefs';
import type { Locale, ThemeMode } from '../composables/useSettings';

const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{ (e: 'update:open', v: boolean): void }>();

const { settings, load, patch } = useSettings();

const themeOptions: { value: ThemeMode; labelKey: string }[] = [
  { value: 'dark', labelKey: 'settings.dark' },
  { value: 'light', labelKey: 'settings.light' },
];
const localeOptions: { value: Locale; labelKey: string }[] = [
  { value: 'zh', labelKey: 'settings.zh' },
  { value: 'en', labelKey: 'settings.en' },
];

// 面板打开时从后端刷新，确保 minimizeToTray 与 DB 最新一致
watch(
  () => props.open,
  (v) => {
    if (v) void load();
  },
);

function close(): void {
  emit('update:open', false);
}
</script>

<template>
  <Transition name="modal">
    <div v-if="props.open" class="overlay" @click.self="close">
      <div class="modal" role="dialog" aria-modal="true" :aria-label="t('settings.title')">
        <div class="modal-head">
          <h3>{{ t('settings.title') }}</h3>
        </div>
        <div class="modal-body">
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
        <div class="modal-foot">
          <button class="btn primary" type="button" @click="close">{{ t('settings.close') }}</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0;
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(2px);
  display: flex; align-items: center; justify-content: center;
  z-index: 1200;
}
.modal {
  width: 420px; max-width: calc(100vw - 32px);
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: var(--modal-shadow);
  overflow: hidden;
}
.modal-head {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.modal-head h3 { margin: 0; font-size: 14px; font-weight: 600; color: var(--text); }
.modal-body { padding: 16px; display: flex; flex-direction: column; gap: 14px; }
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.label { font-size: 13px; color: var(--text); }
.sel {
  padding: 6px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text);
  font-size: 13px;
  min-width: 150px;
}
.sel:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.chk { width: 16px; height: 16px; accent-color: var(--accent); cursor: pointer; }
.modal-foot {
  display: flex; justify-content: flex-end; gap: 8px;
  padding: 12px 16px; border-top: 1px solid var(--border);
}
.btn {
  padding: 7px 16px; font-size: 13px;
  background: var(--bg-input); color: var(--text);
  border: 1px solid var(--border); border-radius: var(--radius);
  cursor: pointer; transition: all 0.12s;
}
.btn:hover { border-color: var(--accent); color: var(--accent); }
.btn.primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.btn.primary:hover { filter: brightness(1.1); color: #fff; }
.modal-enter-active, .modal-leave-active { transition: opacity 0.18s; }
.modal-enter-active .modal, .modal-leave-active .modal { transition: transform 0.18s, opacity 0.18s; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-from .modal { transform: scale(0.95) translateY(-8px); opacity: 0; }
.modal-leave-to .modal { transform: scale(0.95); opacity: 0; }
</style>
