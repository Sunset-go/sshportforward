<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { usePrefs } from '../composables/usePrefs';
import type { ThemeMode } from '../composables/usePrefs';

const { theme, setTheme, t } = usePrefs();

const open = ref(false);
const rootRef = ref<HTMLElement | null>(null);

const options: { value: ThemeMode }[] = [{ value: 'dark' }, { value: 'light' }];

function toggleOpen() {
  open.value = !open.value;
}
function choose(m: ThemeMode) {
  setTheme(m);
  open.value = false;
}
function onDocClick(e: MouseEvent) {
  if (rootRef.value && !rootRef.value.contains(e.target as Node)) open.value = false;
}
onMounted(() => document.addEventListener('click', onDocClick));
onUnmounted(() => document.removeEventListener('click', onDocClick));
</script>

<template>
  <div class="theme-switcher" ref="rootRef">
    <button class="sw-btn" type="button" :title="t('theme.title')" @click="toggleOpen">
      <svg v-if="theme === 'dark'" width="15" height="15" viewBox="0 0 24 24" fill="none">
        <path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8z" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.6" />
        <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
      </svg>
      <span>{{ theme === 'dark' ? t('theme.dark') : t('theme.light') }}</span>
      <svg class="chev" :class="{ open }" width="12" height="12" viewBox="0 0 24 24" fill="none">
        <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <ul v-if="open" class="menu">
      <li
        v-for="o in options"
        :key="o.value"
        :class="{ active: theme === o.value }"
        @click="choose(o.value)"
      >
        <svg v-if="o.value === 'dark'" width="15" height="15" viewBox="0 0 24 24" fill="none">
          <path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8z" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.6" />
          <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        <span>{{ o.value === 'dark' ? t('theme.dark') : t('theme.light') }}</span>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.theme-switcher { position: relative; }
.sw-btn {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 6px 10px; font-size: 13px;
  background: var(--bg-input); color: var(--text);
  border: 1px solid var(--border); border-radius: var(--radius);
  cursor: pointer; transition: all 0.15s;
}
.sw-btn:hover { border-color: var(--accent); color: var(--accent); }
.chev { color: var(--text-dim); transition: transform 0.15s; }
.chev.open { transform: rotate(180deg); }
.menu {
  position: absolute; right: 0; z-index: 20; margin-top: 4px;
  min-width: 132px;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--modal-shadow);
  padding: 4px 0;
}
.menu li {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px; cursor: pointer; font-size: 13px;
}
.menu li:hover { background: var(--accent-hover); }
.menu li.active { color: var(--accent); }
</style>
