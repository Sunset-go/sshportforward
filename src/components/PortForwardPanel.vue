<script setup lang="ts">
import { computed, ref } from 'vue';
import { useSshTunnel } from '../composables/useSshTunnel';
import { t } from '../composables/usePrefs';
import type { RuleType } from '../types';
import RuleTable from './RuleTable.vue';
import RuleEditModal from './RuleEditModal.vue';

const { state, currentHostIsDraft, setActiveRuleType, addRule } = useSshTunnel();
const isLocked = computed(() => state.connState === 'connected' || currentHostIsDraft.value);

const collapsed = ref(false);

const tabs: { type: RuleType; flag: string }[] = [
  { type: 'local', flag: '-L' },
  { type: 'remote', flag: '-R' },
  { type: 'dynamic', flag: '-D' },
];
</script>

<template>
  <section class="card" :style="collapsed ? 'flex: 0 0 auto' : undefined">
    <div class="card-head" :class="{ collapsed }" @click="collapsed = !collapsed">
      <h2 class="title">{{ t('pf.title') }}</h2>
      <div class="tabs" @click.stop>
        <button
          v-for="tab in tabs"
          :key="tab.type"
          class="tab"
          :class="{ active: state.activeRuleType === tab.type }"
          type="button"
          @click="setActiveRuleType(tab.type)"
        >
          {{ t('common.' + tab.type) }} <span class="flag">{{ tab.flag }}</span>
        </button>
      </div>
      <div class="spacer"></div>
      <svg class="chevron" :class="{ down: !collapsed }" width="14" height="14" viewBox="0 0 24 24" fill="none">
        <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </div>

    <template v-if="!collapsed">
    <div class="panel-body">
      <RuleTable />
    </div>

    <div class="panel-foot">
      <button class="add-btn" type="button" :disabled="isLocked" @click="async () => await addRule()">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
          <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
        {{ t('pf.add') }}
      </button>
    </div>
    </template>

    <RuleEditModal />
  </section>
</template>

<style scoped>
.card {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--panel-shadow);
  display: flex; flex-direction: column;
  min-height: 0;
}
.card-head {
  display: flex; align-items: center; gap: 16px;
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  cursor: pointer; user-select: none;
}
.card-head.collapsed { border-bottom-color: transparent; }
.card-head h2 { font-size: 13px; font-weight: 600; color: var(--text); margin: 0; white-space: nowrap; }
.card-head .title { cursor: inherit; }

.tabs { display: flex; gap: 4px; }
.tab {
  display: inline-flex; align-items: center; gap: 4px;
  padding: 6px 12px; font-size: 13px;
  background: transparent; color: var(--text-dim);
  border: none; border-bottom: 2px solid transparent;
  cursor: pointer; transition: color 0.15s, border-color 0.15s;
}
.tab:hover { color: var(--text); }
.tab.active { color: var(--accent); border-bottom-color: var(--accent); }
.flag { font-size: 11px; opacity: 0.7; }
.spacer { flex: 1; }
.chevron { color: var(--text-dim); transition: transform 0.2s; flex-shrink: 0; }
.chevron.down { transform: rotate(180deg); }

.panel-body { flex: 1; min-height: 0; overflow: hidden; }
.panel-foot { padding: 8px 14px; border-top: 1px solid var(--border); }
.add-btn {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 6px 12px; font-size: 13px;
  background: var(--bg-input); color: var(--text-dim);
  border: 1px dashed var(--border); border-radius: var(--radius);
  cursor: pointer; transition: all 0.15s;
}
.add-btn:hover { color: var(--accent); border-color: var(--accent); }
.add-btn:disabled { opacity: 0.3; cursor: default; pointer-events: none; }
</style>
