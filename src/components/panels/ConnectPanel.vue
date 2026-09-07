<script setup lang="ts">
// 连接页签：SSH 连接信息 + 隧道概览两张卡片。主机选择由侧边栏统一承担，此处只负责编辑与保存/删除。
import { computed, onUnmounted, ref, watch } from 'vue';
import { useSshTunnel } from '../../composables/useSshTunnel';
import { t } from '../../composables/usePrefs';
import { formatBytes, hostDraftToAuthInput, hostToDraft } from '../../utils';
import HostForm from '../common/HostForm.vue';
import type { HostDraft } from '../../types';

const { state, currentHost, saveHost, deleteHost } = useSshTunnel();

const draft = ref<HostDraft>({
  id: '', name: '', host: '', port: 22, username: '', method: 'password', password: '', keyPath: '', passphrase: '',
});

watch(
  currentHost,
  (h) => {
    draft.value = h ? hostToDraft(h) : { ...draft.value, id: '', name: '', host: '', port: 22, username: '', method: 'password', password: '', keyPath: '', passphrase: '' };
  },
  { immediate: true },
);

const isConnected = computed(() => state.connState === 'connected');

async function save(): Promise<void> {
  const h = currentHost.value;
  if (!h) return;
  h.name = draft.value.name.trim() || t('common.unnamed');
  h.host = draft.value.host.trim();
  h.port = draft.value.port;
  h.username = draft.value.username;
  const auth = hostDraftToAuthInput(draft.value);
  h.password = auth.password;
  h.keyPath = auth.keyPath;
  h.passphrase = auth.passphrase;
  await saveHost();
}

async function remove(): Promise<void> {
  await deleteHost();
}

// 隧道概览
const ruleCount = computed(() => currentHost.value?.rules.length ?? 0);
const activeTunnelCount = computed(() => {
  if (!isConnected.value || !currentHost.value) return 0;
  return currentHost.value.rules.filter((r) => r.enabled).length;
});
const totalTraffic = computed(() => formatBytes(state.totalBytes));

const now = ref(Date.now());
let uptimeTimer: ReturnType<typeof setInterval> | null = null;
watch(
  () => state.connState,
  (v) => {
    if (v === 'connected' && uptimeTimer === null) {
      uptimeTimer = setInterval(() => (now.value = Date.now()), 1000);
    } else if (v !== 'connected' && uptimeTimer !== null) {
      clearInterval(uptimeTimer);
      uptimeTimer = null;
    }
  },
  { immediate: true },
);
onUnmounted(() => {
  if (uptimeTimer) clearInterval(uptimeTimer);
});

const uptime = computed(() => {
  const ms = state.connectedAt ? Math.max(0, now.value - state.connectedAt) : 0;
  const s = Math.floor(ms / 1000);
  const p = (n: number) => String(n).padStart(2, '0');
  return `${p(Math.floor(s / 3600))}:${p(Math.floor((s % 3600) / 60))}:${p(s % 60)}`;
});

const stats = computed(() => [
  { label: t('conn.rules'), value: String(ruleCount.value) },
  { label: t('conn.activeTunnels'), value: String(activeTunnelCount.value) },
  { label: t('conn.totalTraffic'), value: totalTraffic.value },
  { label: t('conn.uptime'), value: uptime.value },
]);
</script>

<template>
  <div class="connect-panel">
    <!-- SSH 连接信息 -->
    <section class="card">
      <div class="card-head">
        <div class="head-left">
          <h2>{{ t('conn.title') }}</h2>
          <span v-if="currentHost" class="hostname">· {{ currentHost.name || t('common.unnamed') }}</span>
        </div>
        <div class="head-actions">
          <button class="act" type="button" :disabled="!currentHost || isConnected" @click="save">
            {{ t('common.save') }}
          </button>
          <button class="act" type="button" :disabled="!currentHost || isConnected" @click="remove">
            {{ t('common.delete') }}
          </button>
        </div>
      </div>

      <div class="card-body">
        <template v-if="currentHost">
          <HostForm v-model="draft" variant="connect" :disabled="isConnected" />
        </template>
        <p v-else class="empty">{{ t('logmsg.noHost') }}</p>
      </div>
    </section>

    <!-- 隧道概览 -->
    <section class="card overview">
      <div class="card-head">
        <h2>{{ t('conn.overview') }}</h2>
      </div>
      <div class="stats">
        <div v-for="s in stats" :key="s.label" class="stat">
          <span class="stat-label">{{ s.label }}</span>
          <span class="stat-value">{{ s.value }}</span>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.connect-panel {
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
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.head-left {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}
.card-head h2 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
}
.hostname {
  font-size: 12px;
  color: var(--text-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.head-actions { display: flex; gap: 8px; flex-shrink: 0; }

.act {
  height: 30px;
  padding: 0 14px;
  background: var(--btn-bg);
  border: 1px solid var(--btn-border);
  border-radius: var(--radius-btn);
  color: var(--text);
  font-size: 13px;
  cursor: pointer;
  transition: filter 0.12s, opacity 0.12s;
}
.act:hover:not(:disabled) { filter: brightness(1.08); }
.act:disabled { opacity: 0.45; cursor: not-allowed; }
.act:focus-visible { outline: 2px solid var(--blue); outline-offset: 1px; }

.card-body { padding: 16px; }
.empty { margin: 0; font-size: 13px; color: var(--text-3); padding: 8px 0; }

.stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
}
.stat {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 16px;
  border-right: 1px solid var(--border);
}
.stat:last-child { border-right: none; }
.stat-label { font-size: 11px; color: var(--text-3); }
.stat-value { font-size: 16px; color: var(--text); font-weight: 600; }
</style>
