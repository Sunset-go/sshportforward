<script setup lang="ts">
// 顶部工具栏：应用标识 + 语言/主题/设置三按钮。
// data-tauri-drag-region 供无边框窗口拖动（原生标题栏下无副作用）；
// 按钮不带该属性，点击不被拖动吞掉。
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useSettings } from '../composables/useSettings';
import { t } from '../composables/usePrefs';
import type { Locale } from '../composables/useSettings';

const emit = defineEmits<{ (e: 'open-settings'): void }>();

const { settings, toggleTheme, setLocale } = useSettings();

const langOpen = ref(false);
const langBtnRef = ref<HTMLElement | null>(null);

const langOptions: { value: Locale; label: string }[] = [
  { value: 'zh', label: '简体中文' },
  { value: 'en', label: 'English' },
];

function toggleLang(): void {
  langOpen.value = !langOpen.value;
}

function chooseLang(l: Locale): void {
  setLocale(l);
  langOpen.value = false;
}

function onDocClick(e: MouseEvent): void {
  if (langBtnRef.value && !langBtnRef.value.contains(e.target as Node)) langOpen.value = false;
}

onMounted(() => document.addEventListener('click', onDocClick));
onUnmounted(() => document.removeEventListener('click', onDocClick));

const themeIcon = computed(() => settings.theme);
</script>

<template>
  <div class="topbar" data-tauri-drag-region>
    <div class="left" data-tauri-drag-region>
      <svg class="logo" width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
        <path d="M12 8a2 2 0 1 1 0 4 2 2 0 0 1 0-4z" stroke="currentColor" stroke-width="1.5" />
      </svg>
      <span class="app-name">sshportforward</span>
    </div>

    <div class="right">
      <!-- 语言：下拉 -->
      <div class="btn-wrap" ref="langBtnRef">
        <button class="icon-btn" type="button" :title="t('topbar.language')" @click="toggleLang">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5" />
            <path d="M3 12h18M12 3c2.5 2.5 2.5 15 0 18M12 3c-2.5 2.5-2.5 15 0 18" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
        <ul v-if="langOpen" class="dropdown" @click.stop>
          <li
            v-for="opt in langOptions"
            :key="opt.value"
            :class="{ active: settings.locale === opt.value }"
            @click="chooseLang(opt.value)"
          >
            {{ opt.label }}
          </li>
        </ul>
      </div>

      <!-- 主题：点击直接切换，图标随主题变化 -->
      <button class="icon-btn" type="button" :title="t('topbar.theme')" @click="toggleTheme">
        <svg v-if="themeIcon === 'dark'" width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
        </svg>
        <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.5" />
          <path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M19.1 4.9L17 7M7 17l-2.1 2.1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>

      <!-- 设置 -->
      <button class="icon-btn" type="button" :title="t('topbar.settings')" @click="emit('open-settings')">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z" stroke="currentColor" stroke-width="1.5" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.topbar {
  height: 42px;
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px 0 14px;
  background: var(--topbar-bg);
  border-bottom: 1px solid var(--topbar-border);
  user-select: none;
}
.left {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text);
}
.logo { color: var(--accent); }
.app-name {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.2px;
}

.right {
  display: flex;
  align-items: center;
  gap: 4px;
  position: relative;
}
/* 按钮组左侧竖分隔线 */
.right::before {
  content: '';
  display: block;
  width: 1px;
  height: 20px;
  margin-right: 10px;
  background: var(--border);
}

.btn-wrap { position: relative; display: flex; }
.icon-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 7px;
  color: var(--text-dim);
  cursor: pointer;
  transition: background 0.12s, color 0.12s, border-color 0.12s;
}
.icon-btn:hover {
  background: var(--accent-hover);
  color: var(--text);
}
.icon-btn:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
  border-color: transparent;
}

.dropdown {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  z-index: 1100;
  min-width: 140px;
  list-style: none;
  margin: 0;
  padding: 4px 0;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--modal-shadow);
}
.dropdown li {
  padding: 8px 14px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
}
.dropdown li:hover { background: var(--accent-hover); }
.dropdown li.active { color: var(--accent); font-weight: 600; }
</style>
