<script setup lang="ts">
// 顶部页签 + 右侧状态徽标与启动/关闭按钮（全局操作，常驻不随页签切换）。
import { computed } from 'vue';
import { useSshTunnel } from '../../composables/useSshTunnel';
import { t } from '../../composables/usePrefs';
import type { PanelKey } from '../../types';

defineProps<{ active: PanelKey }>();
const emit = defineEmits<{ (e: 'change', v: PanelKey): void }>();

const { state, startTunnel, stopTunnel } = useSshTunnel();

const tabs: { key: PanelKey; labelKey: string }[] = [
  { key: 'connect', labelKey: 'topbar.connect' },
  { key: 'forward', labelKey: 'topbar.forward' },
  { key: 'log', labelKey: 'topbar.logs' },
  { key: 'settings', labelKey: 'topbar.settings' },
];

const isConnected = computed(() => state.connState === 'connected');
const startDisabled = computed(
  () => state.connState === 'connecting' || state.connState === 'connected' || state.connState === 'disconnecting',
);
const stopDisabled = computed(() => state.connState !== 'connected');

const statusLabel = computed(() => {
  switch (state.connState) {
    case 'connected': return t('status.connected');
    case 'connecting': return t('status.connecting');
    case 'disconnecting': return t('status.disconnecting');
    case 'error': return t('status.error');
    default: return t('status.idle');
  }
});
</script>

<template>
  <div class="tabbar" data-tauri-drag-region>
    <nav class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="tab"
        :class="{ active: active === tab.key }"
        type="button"
        @click="emit('change', tab.key)"
      >
        <svg v-if="tab.key === 'connect'" width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M8 12h8M13 8l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
          <path d="M16 5l4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          <path d="M8 3H4v4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <svg v-else-if="tab.key === 'forward'" width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M7 4h13v13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          <path d="M4 20L20 4M11 4h9v9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <svg v-else-if="tab.key === 'log'" width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <rect x="4" y="4" width="16" height="16" rx="3" stroke="currentColor" stroke-width="1.5" />
          <path d="M8 9h8M8 13h8M8 17h4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.5" />
          <path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M19.1 4.9L17 7M7 17l-2.1 2.1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span>{{ t(tab.labelKey) }}</span>
        <span class="bar"></span>
      </button>
    </nav>

    <div class="right">
      <span class="status" :class="{ on: isConnected }">
        <span class="dot"></span>
        <span class="label">{{ statusLabel }}</span>
      </span>
      <button class="btn start" type="button" :disabled="startDisabled" @click="startTunnel">
        {{ state.connState === 'connecting' ? t('conn.connecting') : t('conn.start') }}
      </button>
      <button class="btn stop" type="button" :disabled="stopDisabled" @click="stopTunnel">
        {{ t('conn.stop') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.tabbar {
  flex: 0 0 auto;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 14px;
  background: var(--side-bg);
  border-bottom: 1px solid var(--border);
  user-select: none;
}

.tabs {
  display: flex;
  align-items: stretch;
  gap: 4px;
  height: 100%;
}
.tab {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 14px;
  background: transparent;
  border: none;
  border-radius: 7px;
  color: var(--text-3);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.tab:hover { color: var(--text-2); }
.tab.active {
  background: var(--sel);
  color: var(--text);
}
.tab:focus-visible { outline: 2px solid var(--blue); outline-offset: -2px; }

.bar {
  position: absolute;
  left: 50%;
  bottom: 5px;
  transform: translateX(-50%);
  width: 0;
  height: 2.5px;
  border-radius: 2px;
  background: var(--blue);
  transition: width 0.12s;
}
.tab.active .bar { width: 24px; }

.right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.status {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 13px;
  color: var(--text-3);
}
.status .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-3);
}
.status.on { color: var(--green-text); }
.status.on .dot { background: var(--green); }

.btn {
  width: 68px;
  height: 28px;
  border-radius: var(--radius-btn);
  border: 1px solid transparent;
  font-size: 13px;
  cursor: pointer;
  transition: filter 0.12s, opacity 0.12s;
}
.btn:focus-visible { outline: 2px solid var(--blue); outline-offset: 1px; }
.btn:disabled { opacity: 0.45; cursor: not-allowed; }
.btn.start {
  background: var(--blue);
  color: #fff;
}
.btn.start:hover:not(:disabled) { filter: brightness(1.08); }
.btn.stop {
  background: var(--btn-bg);
  border-color: var(--btn-border);
  color: var(--text);
}
.btn.stop:hover:not(:disabled) { filter: brightness(1.08); }
</style>
