<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useSshTunnel } from '../composables/useSshTunnel';
import { t } from '../composables/usePrefs';

const { state, clearLogs, logsText } = useSshTunnel();

const bodyRef = ref<HTMLDivElement | null>(null);
const autoScroll = ref(true);
const copied = ref(false);
const collapsed = computed({
  get: () => state.cardCollapsed.logs,
  set: (v) => { state.cardCollapsed.logs = v; },
});

function onScroll() {
  const el = bodyRef.value;
  if (!el) return;
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
  autoScroll.value = atBottom;
}

watch(
  () => state.logs.length,
  () => {
    void nextTick(() => {
      if (autoScroll.value && bodyRef.value) {
        bodyRef.value.scrollTop = bodyRef.value.scrollHeight;
      }
    });
  },
);

async function copyAll() {
  const text = logsText();
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1200);
  } catch {
    /* 剪贴板不可用时静默忽略 */
  }
}

function levelClass(l: string) {
  return 'lvl-' + l.toLowerCase();
}
</script>

<template>
  <section class="log-card" :style="collapsed ? 'flex: 0 0 auto' : undefined">
    <div class="log-head" :class="{ collapsed }" @click="collapsed = !collapsed">
      <h2 class="title">{{ t('log.title') }}</h2>
      <span class="count">{{ t('log.count', { n: state.logs.length }) }}</span>
      <div class="spacer"></div>
      <svg class="chevron" :class="{ down: !collapsed }" width="14" height="14" viewBox="0 0 24 24" fill="none">
        <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <button class="tool" type="button" @click.stop="copyAll">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
          <rect x="9" y="9" width="11" height="11" rx="2" stroke="currentColor" stroke-width="1.6" />
          <path d="M5 15V5a2 2 0 0 1 2-2h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        {{ copied ? t('log.copied') : t('log.copy') }}
      </button>
      <button class="tool" type="button" @click.stop="clearLogs">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
          <path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2m-9 0v12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        {{ t('log.clear') }}
      </button>
    </div>

    <div class="log-body" ref="bodyRef" @scroll="onScroll" v-show="!collapsed">
      <div v-if="state.logs.length === 0" class="empty">{{ t('log.empty') }}</div>
      <div v-for="l in state.logs" :key="l.id" class="line" :class="levelClass(l.level)">
        <span class="time">[{{ l.time }}]</span>
        <span class="level">{{ l.level }}</span>
        <span class="msg">{{ l.message }}</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.log-card {
  display: flex; flex-direction: column;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--panel-shadow);
  min-height: 0; overflow: hidden;
}
.log-head {
  display: flex; align-items: center; gap: 10px;
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  cursor: pointer; user-select: none;
}
.log-head.collapsed { border-bottom-color: transparent; }
.log-head h2 { font-size: 13px; font-weight: 600; color: var(--text); margin: 0; }
.log-head .title { cursor: inherit; }
.count { font-size: 12px; color: var(--text-dim); }
.spacer { flex: 1; }
.chevron { color: var(--text-dim); transition: transform 0.2s; flex-shrink: 0; }
.chevron.down { transform: rotate(180deg); }
.tool {
  display: inline-flex; align-items: center; gap: 5px;
  padding: 5px 10px; font-size: 12px;
  background: var(--bg-input); color: var(--text-dim);
  border: 1px solid var(--border); border-radius: var(--radius);
  cursor: pointer; transition: all 0.12s;
}
.tool:hover { color: var(--accent); border-color: var(--accent); }

.log-body {
  flex: 1; min-height: 0; overflow-y: auto;
  padding: 8px 14px;
  font-family: ui-monospace, "Cascadia Code", Consolas, "Microsoft YaHei", monospace;
  font-size: 12.5px; line-height: 1.7;
}
.line { display: flex; gap: 8px; white-space: pre-wrap; word-break: break-word; }
.time { color: var(--text-dim); flex-shrink: 0; }
.level { flex-shrink: 0; width: 60px; font-weight: 600; }
.msg { color: var(--text); }

.lvl-info .level { color: var(--log-info); }
.lvl-success .level { color: var(--success); }
.lvl-success .msg { color: var(--log-success-text); }
.lvl-warn .level { color: var(--warn); }
.lvl-warn .msg { color: var(--log-warn-text); }
.lvl-error .level { color: var(--error); }
.lvl-error .msg { color: var(--log-error-text); }

.empty { color: var(--text-dim); padding: 16px 0; }
</style>
