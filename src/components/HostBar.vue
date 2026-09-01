<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useSshTunnel } from '../composables/useSshTunnel';
import { t } from '../composables/usePrefs';
import StatusDot from './StatusDot.vue';
import LangSwitcher from './LangSwitcher.vue';
import ThemeSwitcher from './ThemeSwitcher.vue';

const { state, currentHost, selectHost, saveHost, deleteHost } = useSshTunnel();

const isConnected = computed(() => state.connState === 'connected');

const open = ref(false);
const rootRef = ref<HTMLElement | null>(null);
const hostName = ref('');

watch(
  () => state.currentHostId,
  () => {
    hostName.value = currentHost.value?.name ?? '';
  },
  { immediate: true },
);

function onSave() {
  const h = currentHost.value;
  if (!h) return;
  h.name = hostName.value.trim() || t('common.unnamed');
  saveHost();
}

function toggleOpen() {
  open.value = !open.value;
}
function choose(id: string) {
  selectHost(id);
  open.value = false;
}
function onDocClick(e: MouseEvent) {
  if (rootRef.value && !rootRef.value.contains(e.target as Node)) open.value = false;
}
onMounted(() => document.addEventListener('click', onDocClick));
onUnmounted(() => document.removeEventListener('click', onDocClick));

const label = computed(() => {
  const h = currentHost.value;
  return h ? `${h.name} (${h.host})` : '—';
});
</script>

<template>
  <header class="host-bar">
    <div class="left">
      <span class="title">{{ t('hostbar.title') }}</span>
      <div class="dropdown" ref="rootRef">
        <button class="host-select" @click="toggleOpen" type="button" :disabled="isConnected">
          <span class="host-name">{{ label }}</span>
          <svg class="chev" :class="{ open }" width="12" height="12" viewBox="0 0 24 24" fill="none">
            <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <ul v-if="open" class="menu">
          <li
            v-for="h in state.hosts"
            :key="h.id"
            :class="{ active: h.id === state.currentHostId }"
            @click="choose(h.id)"
          >
            <span class="m-name">{{ h.name }}</span>
            <span class="m-host">{{ h.host }}</span>
          </li>
        </ul>
      </div>
      <input
        class="name-input"
        type="text"
        v-model="hostName"
        :placeholder="t('hostbar.namePlaceholder')"
        :disabled="isConnected"
      />
    </div>

    <div class="center">
      <StatusDot :state="state.connState" />
    </div>

    <div class="right">
      <button class="btn" type="button" @click="onSave" :disabled="isConnected">{{ t('common.save') }}</button>
      <button class="btn danger" type="button" @click="deleteHost" :disabled="isConnected">{{ t('common.delete') }}</button>
      <LangSwitcher />
      <ThemeSwitcher />
    </div>
  </header>
</template>

<style scoped>
.host-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--gap);
  padding: 10px 14px;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--panel-shadow);
}
.left { display: flex; align-items: center; gap: 12px; min-width: 0; }
.title { font-size: 13px; color: var(--text-dim); white-space: nowrap; }

.dropdown { position: relative; }
.host-select {
  display: flex; align-items: center; gap: 8px;
  min-width: 220px;
  padding: 6px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text);
  font-size: 13px;
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.host-select:hover { border-color: var(--accent); }
.host-select:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.host-name { flex: 1; text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.chev { color: var(--text-dim); transition: transform 0.15s; }
.chev.open { transform: rotate(180deg); }

.menu {
  position: absolute; z-index: 20; margin-top: 4px;
  min-width: 240px;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
  max-height: 240px; overflow-y: auto;
}
.menu li {
  display: flex; align-items: center; justify-content: space-between;
  gap: 10px; padding: 8px 12px; cursor: pointer; font-size: 13px;
}
.menu li:hover { background: var(--accent-soft); }
.menu li.active { color: var(--accent); }
.m-host { color: var(--text-dim); font-size: 12px; }

.center { display: flex; align-items: center; }
.right { display: flex; gap: 8px; }
.btn {
  padding: 6px 14px; font-size: 13px;
  background: var(--bg-input); color: var(--text);
  border: 1px solid var(--border); border-radius: var(--radius);
  cursor: pointer; transition: all 0.15s;
}
.btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.btn:active:not(:disabled) { transform: translateY(1px); }
.btn:disabled { opacity: 0.45; cursor: not-allowed; }
.btn.danger:hover:not(:disabled) { border-color: var(--error); color: var(--error); }
.host-select:disabled { opacity: 0.45; cursor: not-allowed; }
.name-input {
  width: 140px;
  padding: 6px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text);
  font-size: 13px;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.name-input:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.name-input:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
