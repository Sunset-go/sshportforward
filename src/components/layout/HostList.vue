<script setup lang="ts">
// 主机列表：选中项高亮 + 左侧蓝色指示条 + 状态点 + 悬浮「…」菜单（重命名/删除）。
import { computed, nextTick, ref } from 'vue';
import { useSshTunnel } from '../../composables/useSshTunnel';
import { t } from '../../composables/usePrefs';

const props = defineProps<{ filter: string }>();

const { state, selectHost, deleteHost, renameHost } = useSshTunnel();

const menuOpenId = ref<string | null>(null);
const renamingId = ref<string | null>(null);
const renameValue = ref('');
const renameInputRef = ref<HTMLInputElement | null>(null);

const filtered = computed(() => {
  const q = props.filter.trim().toLowerCase();
  if (!q) return state.hosts;
  return state.hosts.filter(
    (h) => h.name.toLowerCase().includes(q) || h.host.toLowerCase().includes(q),
  );
});

function isConnectedHost(id: string): boolean {
  return state.connState === 'connected' && id === state.currentHostId;
}

function toggleMenu(id: string): void {
  menuOpenId.value = menuOpenId.value === id ? null : id;
}

function startRename(id: string): void {
  const h = state.hosts.find((x) => x.id === id);
  if (!h) return;
  menuOpenId.value = null;
  renamingId.value = id;
  renameValue.value = h.name;
  void nextTick(() => renameInputRef.value?.focus());
}

function commitRename(): void {
  if (renamingId.value) void renameHost(renamingId.value, renameValue.value);
  renamingId.value = null;
}

function cancelRename(): void {
  renamingId.value = null;
}
</script>

<template>
  <ul class="host-list">
    <li
      v-for="h in filtered"
      :key="h.id"
      class="item"
      :class="{ active: h.id === state.currentHostId }"
      @click="selectHost(h.id)"
    >
      <span class="indicator"></span>
      <span class="dot" :class="{ on: isConnectedHost(h.id) }"></span>
      <div class="meta">
        <input
          v-if="renamingId === h.id"
          ref="renameInputRef"
          class="rename-input"
          type="text"
          v-model="renameValue"
          @click.stop
          @keyup.enter="commitRename"
          @keyup.esc="cancelRename"
          @blur="commitRename"
        />
        <span v-else class="name" :title="h.name">{{ h.name || t('common.unnamed') }}</span>
        <span class="addr mono">{{ h.host || '—' }}</span>
      </div>

      <div class="more-wrap">
        <button class="more" type="button" :title="t('common.actions')" @click.stop="toggleMenu(h.id)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <circle cx="5" cy="12" r="1.6" />
            <circle cx="12" cy="12" r="1.6" />
            <circle cx="19" cy="12" r="1.6" />
          </svg>
        </button>
        <div v-if="menuOpenId === h.id" class="menu" @click.stop>
          <button type="button" @click="startRename(h.id)">{{ t('common.rename') }}</button>
          <button type="button" class="danger" @click="deleteHost(h.id)">{{ t('common.delete') }}</button>
        </div>
      </div>
    </li>
  </ul>
</template>

<style scoped>
.host-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 46px;
  padding: 0 8px;
  border-radius: 7px;
  border: 1px solid transparent;
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s;
}
.item:hover { background: var(--row); }
.item.active {
  background: var(--sel);
  border-color: #33353f;
}
.indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 24px;
  border-radius: 2px;
  background: transparent;
}
.item.active .indicator { background: var(--blue); }

.dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-3);
}
.dot.on { background: var(--green); }

.meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.name {
  font-size: 13px;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.addr {
  font-size: 11px;
  color: var(--text-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mono { font-family: ui-monospace, "Cascadia Code", Consolas, monospace; }

.rename-input {
  width: 100%;
  height: 20px;
  padding: 0 4px;
  background: var(--input);
  border: 1px solid var(--blue);
  border-radius: 4px;
  color: var(--text);
  font-size: 13px;
}
.rename-input:focus { outline: none; }

.more-wrap { position: relative; flex-shrink: 0; }
.more {
  width: 22px;
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 5px;
  color: var(--text-3);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.12s, background 0.12s, color 0.12s;
}
.item:hover .more { opacity: 1; }
.more:hover { background: var(--btn-bg); color: var(--text); }
.more:focus-visible { outline: 2px solid var(--blue); opacity: 1; }

.menu {
  position: absolute;
  right: 0;
  top: calc(100% + 2px);
  z-index: 30;
  min-width: 96px;
  display: flex;
  flex-direction: column;
  padding: 4px;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: var(--modal-shadow);
}
.menu button {
  text-align: left;
  padding: 6px 8px;
  background: transparent;
  border: none;
  border-radius: 5px;
  color: var(--text);
  font-size: 12px;
  cursor: pointer;
}
.menu button:hover { background: var(--accent-hover); }
.menu button.danger { color: var(--error); }
</style>
