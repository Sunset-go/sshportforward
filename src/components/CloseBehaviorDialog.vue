<script setup lang="ts">
// 关闭行为询问弹窗：后端拦截窗口关闭且数据库未记录偏好时触发（ask-close-behavior 事件）。
// 用户选择"最小化到托盘"或"退出"后写入数据库（app_settings.close_to_tray），之后不再询问。
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { t } from '../composables/usePrefs';

/** 与 Rust 端 commands::settings::CLOSE_TO_TRAY_KEY 保持一致 */
const CLOSE_TO_TRAY_KEY = 'close_to_tray';

const visible = ref(false);
let unlisten: UnlistenFn | null = null;

onMounted(async () => {
  unlisten = await listen('ask-close-behavior', () => {
    visible.value = true;
  });
  syncTrayTexts();
});

onUnmounted(() => unlisten?.());

// 跟随语言切换更新托盘菜单文案（t 内部读取 reactive locale，可被 watch 追踪）
watch(() => t('tray.show'), syncTrayTexts);

async function syncTrayTexts(): Promise<void> {
  try {
    await invoke('set_tray_texts', {
      show: t('tray.show'),
      closeToTray: t('tray.closeToTray'),
      quit: t('tray.quit'),
    });
  } catch {
    /* 托盘未就绪时忽略 */
  }
}

/** 记录选择并执行对应行为：tray → 隐藏到托盘；exit → 退出应用 */
async function choose(value: 'tray' | 'exit'): Promise<void> {
  visible.value = false;
  try {
    await invoke('set_app_setting', { key: CLOSE_TO_TRAY_KEY, value });
  } catch (e) {
    console.error('保存关闭行为设置失败', e);
    return;
  }
  try {
    if (value === 'tray') {
      await invoke('hide_main_window');
    } else {
      await invoke('exit_app');
    }
  } catch (e) {
    console.error('执行关闭行为失败', e);
  }
}

/** 取消：不记录偏好，下次关闭仍询问 */
function cancel(): void {
  visible.value = false;
}
</script>

<template>
  <Transition name="modal">
    <div v-if="visible" class="overlay">
      <div class="modal" role="dialog" aria-modal="true" :aria-label="t('closeDlg.title')">
        <div class="modal-head">
          <h3>{{ t('closeDlg.title') }}</h3>
        </div>
        <div class="modal-body">
          <p class="msg">{{ t('closeDlg.message') }}</p>
        </div>
        <div class="modal-foot">
          <button class="btn" type="button" @click="cancel">{{ t('common.cancel') }}</button>
          <button class="btn" type="button" @click="choose('exit')">{{ t('closeDlg.exit') }}</button>
          <button class="btn primary" type="button" @click="choose('tray')">{{ t('closeDlg.toTray') }}</button>
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
  z-index: 1100;
}
.modal {
  width: 440px; max-width: calc(100vw - 32px);
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
.modal-body { padding: 16px; }
.msg { margin: 0; font-size: 13px; line-height: 1.7; color: var(--text); }
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
