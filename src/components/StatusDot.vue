<script setup lang="ts">
import { computed } from 'vue';
import type { ConnState } from '../types';
import { t } from '../composables/usePrefs';

const props = defineProps<{ state: ConnState }>();

const map: Record<ConnState, { color: string; labelKey: string }> = {
  idle: { color: 'var(--text-dim)', labelKey: 'status.idle' },
  connecting: { color: 'var(--warn)', labelKey: 'status.connecting' },
  connected: { color: 'var(--success)', labelKey: 'status.connected' },
  error: { color: 'var(--error)', labelKey: 'status.error' },
  disconnecting: { color: 'var(--warn)', labelKey: 'status.disconnecting' },
};

const info = computed(() => map[props.state]);
const pulsing = computed(() => props.state === 'connecting' || props.state === 'disconnecting');
</script>

<template>
  <span class="status-dot" :style="{ '--dot-color': info.color }">
    <span class="dot" :class="{ pulsing }"></span>
    <span class="label">{{ t(info.labelKey) }}</span>
  </span>
</template>

<style scoped>
.status-dot {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text);
}
.dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: var(--dot-color);
  box-shadow: 0 0 6px var(--dot-color);
}
.dot.pulsing {
  animation: pulse 1s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.7); }
}
.label { white-space: nowrap; }
</style>
