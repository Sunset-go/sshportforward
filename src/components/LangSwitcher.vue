<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { usePrefs } from '../composables/usePrefs';
import type { Locale } from '../composables/usePrefs';

const { locale, setLocale, t } = usePrefs();

const open = ref(false);
const rootRef = ref<HTMLElement | null>(null);

const options: { value: Locale; label: string }[] = [
  { value: 'zh', label: '简体中文' },
  { value: 'en', label: 'English' },
];

function toggleOpen() {
  open.value = !open.value;
}
function choose(l: Locale) {
  setLocale(l);
  open.value = false;
}
function onDocClick(e: MouseEvent) {
  if (rootRef.value && !rootRef.value.contains(e.target as Node)) open.value = false;
}
onMounted(() => document.addEventListener('click', onDocClick));
onUnmounted(() => document.removeEventListener('click', onDocClick));
</script>

<template>
  <div class="lang-switcher" ref="rootRef">
    <button class="sw-btn" type="button" :title="t('lang.title')" @click="toggleOpen">
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.6" />
        <path d="M3 12h18M12 3a15.5 15.5 0 0 1 0 18M12 3a15.5 15.5 0 0 0 0 18" stroke="currentColor" stroke-width="1.6" />
      </svg>
      <span>{{ locale === 'zh' ? '中' : 'EN' }}</span>
      <svg class="chev" :class="{ open }" width="12" height="12" viewBox="0 0 24 24" fill="none">
        <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <ul v-if="open" class="menu">
      <li
        v-for="o in options"
        :key="o.value"
        :class="{ active: locale === o.value }"
        @click="choose(o.value)"
      >
        {{ o.label }}
      </li>
    </ul>
  </div>
</template>

<style scoped>
.lang-switcher { position: relative; }
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
