<script setup lang="ts">
import { computed } from 'vue';
import { useSshTunnel } from '../composables/useSshTunnel';
import { t } from '../composables/usePrefs';

const { state, currentRules, toggleRule, removeRule, commitRuleEdit, openEditModal } = useSshTunnel();

const isRemote = () => state.activeRuleType === 'remote';
const isDynamic = () => state.activeRuleType === 'dynamic';
/** 连接已建立时锁定规则编辑 */
const isLocked = computed(() => state.connState === 'connected');

/** 行内编辑失焦包装函数 */
async function handleCommitEdit(id: string, field: 'listenAddr' | 'targetAddr' | 'note', value: string) {
  await commitRuleEdit(id, field, value);
}
</script>

<template>
  <div class="rule-table">
    <div class="thead">
      <div class="th c-enable">{{ t('common.enable') }}</div>
      <div class="th c-listen">{{ isRemote() ? t('common.localAddr') : t('common.listenAddr') }}</div>
      <div class="th c-target">{{ isRemote() ? t('rt.targetRemote') : t('common.targetAddr') }}</div>
      <div class="th c-note">{{ t('common.note') }}</div>
      <div class="th c-op">{{ t('common.actions') }}</div>
    </div>

    <div class="tbody">
      <div v-if="currentRules.length === 0" class="empty">{{ isDynamic() ? t('rt.empty.dynamic') : isRemote() ? t('rt.empty.remote') : t('rt.empty.local') }}</div>

      <div v-for="r in currentRules" :key="r.id" class="tr">
        <div class="td c-enable">
          <label class="check" :class="{ locked: isLocked }">
            <input type="checkbox" :checked="r.enabled" :disabled="isLocked" @change="toggleRule(r.id)" />
            <span class="box"></span>
          </label>
        </div>

        <div class="td c-listen">
          <input
            type="text"
            class="cell-input"
            :class="{ locked: isLocked }"
            :value="r.listenAddr"
            :readonly="isLocked"
            placeholder="127.0.0.1:3080"
            @blur="handleCommitEdit(r.id, 'listenAddr', ($event.target as HTMLInputElement).value)"
          />
        </div>

        <div class="td c-target">
          <template v-if="isDynamic()">
            <span class="muted">—</span>
          </template>
          <template v-else>
            <span v-if="isRemote()" class="arrow">{{ t('rt.arrow') }}</span>
            <input
              type="text"
              class="cell-input"
              :class="{ locked: isLocked }"
              :value="r.targetAddr"
              :readonly="isLocked"
              placeholder="127.0.0.1:3080"
              @blur="handleCommitEdit(r.id, 'targetAddr', ($event.target as HTMLInputElement).value)"
            />
          </template>
        </div>

        <div class="td c-note">
          <input
            type="text"
            class="cell-input note"
            :class="{ locked: isLocked }"
            :value="r.note"
            :readonly="isLocked"
            placeholder="—"
            @blur="handleCommitEdit(r.id, 'note', ($event.target as HTMLInputElement).value)"
          />
        </div>

        <div class="td c-op">
          <button class="op-btn edit" type="button" :title="t('rt.edit')" :disabled="isLocked" @click="openEditModal(r.id)">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
              <path d="M12 20h9M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
          <button class="op-btn del" type="button" :title="t('rt.delete')" :disabled="isLocked" @click="async () => await removeRule(r.id)">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
              <path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2m-9 0v12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.rule-table { display: flex; flex-direction: column; height: 100%; min-height: 0; font-size: 13px; }
.thead {
  display: grid; grid-template-columns: 56px 1fr 1fr 1fr 84px;
  align-items: center; padding: 8px 14px;
  background: var(--bg-app);
  border-bottom: 1px solid var(--border);
  position: sticky; top: 0; z-index: 1;
}
.th { color: var(--text-dim); font-size: 12px; font-weight: 500; }
.tbody { flex: 1; min-height: 0; overflow-y: auto; }
.tr {
  display: grid; grid-template-columns: 56px 1fr 1fr 1fr 84px;
  align-items: center; padding: 6px 14px;
  border-bottom: 1px solid var(--border-soft);
  transition: background 0.12s;
}
.tr:hover { background: var(--accent-hover); }
.td { min-width: 0; padding-right: 8px; }
.cell-input {
  width: 100%;
  padding: 4px 8px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: var(--text); font-size: 13px;
  font-family: ui-monospace, Consolas, monospace;
  transition: border-color 0.12s, background 0.12s, box-shadow 0.12s;
}
.cell-input:hover { background: var(--bg-input); border-color: var(--border); }
.cell-input:focus { outline: none; background: var(--bg-input); border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.cell-input.note { font-family: inherit; }
.cell-input.locked { cursor: default; color: var(--text-dim); }
.cell-input.locked:hover { background: transparent; border-color: transparent; }
.muted, .arrow { color: var(--text-dim); font-size: 12px; }

.check { display: inline-flex; align-items: center; cursor: pointer; }
.check input { display: none; }
.box {
  width: 16px; height: 16px; border-radius: 4px;
  border: 1px solid var(--border); background: var(--bg-input);
  transition: all 0.12s; position: relative;
}
.check input:checked + .box { background: var(--accent); border-color: var(--accent); }
.check input:checked + .box::after {
  content: ''; position: absolute; left: 4px; top: 1px;
  width: 5px; height: 9px; border: solid #fff; border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}
.check.locked { cursor: default; }
.check input:disabled + .box { opacity: 0.4; cursor: default; }

.c-op { display: flex; align-items: center; gap: 4px; }
.op-btn {
  display: inline-flex; align-items: center; justify-content: center;
  width: 26px; height: 26px; border-radius: 4px;
  background: transparent; border: 1px solid transparent;
  color: var(--text-dim); cursor: pointer; transition: all 0.12s;
}
.op-btn.edit:hover { color: var(--accent); border-color: var(--accent); background: rgba(47, 129, 247, 0.1); }
.op-btn.del:hover { color: var(--error); border-color: var(--error); background: rgba(248, 81, 73, 0.1); }
.op-btn:disabled { opacity: 0.3; cursor: default; pointer-events: none; }

.empty { padding: 24px; text-align: center; color: var(--text-dim); font-size: 13px; }
</style>
