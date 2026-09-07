<script setup lang="ts">
// 抽屉式侧边栏：品牌行 + 折叠、搜索、已保存主机列表、添加主机、底部偏好。
// data-tauri-drag-region 只加在空白容器上，按钮不加，避免点击被拖动吞掉。
import { onMounted, onUnmounted, ref } from 'vue';
import { useSshTunnel } from '../../composables/useSshTunnel';
import { useSettings } from '../../composables/useSettings';
import { t } from '../../composables/usePrefs';
import type { Locale } from '../../composables/useSettings';
import HostList from './HostList.vue';

defineProps<{ collapsed: boolean }>();
const emit = defineEmits<{
  (e: 'toggle'): void;
  (e: 'add-host'): void;
  (e: 'open-settings'): void;
}>();

const { state, selectHost } = useSshTunnel();
const { settings, toggleTheme, setLocale } = useSettings();

const VERSION = 'v0.1.0';

const search = ref('');
const langOpen = ref(false);
const langBtnRef = ref<HTMLElement | null>(null);

const langOptions: { value: Locale; label: string }[] = [
  { value: 'zh', label: '简体中文' },
  { value: 'en', label: 'English' },
];

function chooseLang(l: Locale): void {
  setLocale(l);
  langOpen.value = false;
}

function onDocClick(e: MouseEvent): void {
  if (langBtnRef.value && !langBtnRef.value.contains(e.target as Node)) langOpen.value = false;
}

onMounted(() => document.addEventListener('click', onDocClick));
onUnmounted(() => document.removeEventListener('click', onDocClick));
</script>

<template>
  <aside class="sidebar" :class="{ collapsed }" data-tauri-drag-region>
    <!-- 品牌行 -->
    <div class="brand" :class="{ collapsed }">
      <div class="brand-left" data-tauri-drag-region>
        <svg class="logo" width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
          <path d="M12 8a2 2 0 1 1 0 4 2 2 0 0 1 0-4z" stroke="currentColor" stroke-width="1.5" />
        </svg>
        <span v-if="!collapsed" class="app-name">sshportforward</span>
      </div>
      <button
        class="fold"
        type="button"
        :title="collapsed ? t('sidebar.expand') : t('sidebar.collapse')"
        @click="emit('toggle')"
      >
        <svg class="fold-ico" :class="{ flipped: collapsed }" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M11 7l-5 5 5 5M13 7l5 5-5 5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </div>

    <!-- 搜索框 -->
    <div v-if="!collapsed" class="search">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle cx="11" cy="11" r="7" stroke="currentColor" stroke-width="1.5" />
        <path d="M20 20l-3.5-3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      <input v-model="search" type="text" :placeholder="t('sidebar.searchPlaceholder')" />
    </div>

    <!-- 分组标题 -->
    <div v-if="!collapsed" class="group-title">
      <span class="group-label">{{ t('sidebar.savedHosts') }}</span>
      <span class="group-right">
        <span class="badge">{{ state.hosts.length }}</span>
        <button class="add-mini" type="button" :title="t('sidebar.addHost')" @click="emit('add-host')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
          </svg>
        </button>
      </span>
    </div>

    <!-- 主机列表 -->
    <div v-if="!collapsed" class="host-scroll">
      <HostList :filter="search" />
    </div>
    <div v-else class="host-scroll collapsed-hosts">
      <button
        v-for="h in state.hosts"
        :key="h.id"
        class="host-dot"
        :class="{ active: h.id === state.currentHostId }"
        type="button"
        :title="h.name"
        @click="selectHost(h.id)"
      >
        <span class="dot" :class="{ on: state.connState === 'connected' && h.id === state.currentHostId }"></span>
      </button>
    </div>

    <!-- 添加主机 -->
    <div class="add-host">
      <button v-if="!collapsed" class="add-dashed" type="button" @click="emit('add-host')">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
        </svg>
        {{ t('sidebar.addHost') }}
      </button>
      <button v-else class="add-circle" type="button" :title="t('sidebar.addHost')" @click="emit('add-host')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <!-- 底部：偏好 + 版本号 -->
    <div class="foot">
      <div class="divider"></div>
      <div class="prefs" :class="{ vertical: collapsed }">
        <!-- 语言 -->
        <div class="btn-wrap" ref="langBtnRef">
          <button class="icon-btn" type="button" :title="t('sidebar.language')" @click="langOpen = !langOpen">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
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

        <!-- 主题 -->
        <button class="icon-btn" type="button" :title="t('sidebar.theme')" @click="toggleTheme">
          <svg v-if="settings.theme === 'dark'" width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
          </svg>
          <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.5" />
            <path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M19.1 4.9L17 7M7 17l-2.1 2.1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>

        <!-- 设置 -->
        <button class="icon-btn" type="button" :title="t('sidebar.settings')" @click="emit('open-settings')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z" stroke="currentColor" stroke-width="1.5" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
          </svg>
        </button>
      </div>
      <span v-if="!collapsed" class="version">{{ VERSION }}</span>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  flex: 0 0 auto;
  width: 212px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--side-bg);
  border-right: 1px solid var(--border);
  transition: width 0.18s ease;
  overflow: hidden;
  user-select: none;
}
.sidebar.collapsed { width: 56px; }

/* 品牌行 */
.brand {
  height: 44px;
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px 0 14px;
  gap: 8px;
}
.brand.collapsed {
  flex-direction: column;
  justify-content: center;
  padding: 0;
  gap: 6px;
  height: auto;
  padding-top: 8px;
}
.brand-left { display: flex; align-items: center; gap: 8px; color: var(--text); }
.brand.collapsed .brand-left { justify-content: center; }
.logo { color: var(--blue); flex-shrink: 0; }
.app-name { font-size: 13px; font-weight: 600; letter-spacing: 0.2px; white-space: nowrap; }

.fold {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--fold-bg);
  border: 1px solid var(--border);
  border-radius: 50%;
  color: var(--icon);
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}
.fold:hover { background: var(--btn-bg); border-color: var(--fold-hover-border); color: var(--text); }
.fold:active { border-color: var(--blue); color: #fff; }
.fold:focus-visible { outline: 2px solid var(--blue); outline-offset: 1px; }
.fold-ico { transition: transform 0.18s ease; }
.fold-ico.flipped { transform: scaleX(-1); }

/* 搜索 */
.search {
  flex: 0 0 auto;
  height: 30px;
  margin: 0 12px 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  background: var(--input);
  border: 1px solid var(--border);
  border-radius: var(--radius-input);
  color: var(--text-3);
}
.search:focus-within { border-color: var(--blue); }
.search input {
  flex: 1;
  min-width: 0;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text);
  font-size: 13px;
}
.search input:focus { outline: none; }
.search input::placeholder { color: var(--text-3); }

/* 分组标题 */
.group-title {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 14px 6px;
  font-size: 11px;
  color: var(--text-3);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.badge {
  min-width: 18px;
  height: 16px;
  padding: 0 5px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--btn-bg);
  border-radius: 8px;
  color: var(--text-2);
  font-size: 11px;
}
.group-right {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.add-mini {
  width: 18px;
  height: 18px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--btn-border);
  border-radius: 5px;
  color: var(--text-2);
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}
.add-mini:hover { background: var(--btn-bg); border-color: var(--blue); color: var(--text); }
.add-mini:focus-visible { outline: 2px solid var(--blue); outline-offset: 1px; }

/* 主机列表滚动区 */
.host-scroll {
  flex: 1 1 0;
  min-height: 0;
  overflow-y: auto;
  padding: 0 8px;
}
.collapsed-hosts {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}
.host-dot {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 7px;
  cursor: pointer;
}
.host-dot:hover { background: var(--row); }
.host-dot.active { background: var(--sel); border-color: var(--btn-border); }
.host-dot .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-3);
}
.host-dot .dot.on { background: var(--green); }

/* 添加主机 */
.add-host {
  flex: 0 0 auto;
  padding: 10px 12px;
}
.add-dashed {
  width: 100%;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: var(--dashed-bg);
  border: 1px dashed var(--dashed-border);
  border-radius: var(--radius-btn);
  color: var(--text-2);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}
.add-dashed:hover { background: #22232b; border-color: #4a4d59; color: var(--text); }
.add-dashed:focus-visible { outline: 2px solid var(--blue); outline-offset: 1px; }

.add-circle {
  width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--dashed-bg);
  border: 1px dashed var(--dashed-border);
  border-radius: 50%;
  color: var(--text-2);
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}
.add-circle:hover { background: #22232b; border-color: #4a4d59; color: var(--text); }
.add-circle:focus-visible { outline: 2px solid var(--blue); outline-offset: 1px; }

/* 底部 */
.foot {
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 12px 12px;
}
.divider { height: 1px; background: var(--border); }
.prefs { display: flex; gap: 4px; }
.prefs.vertical { flex-direction: column; align-items: center; }

.btn-wrap { position: relative; display: flex; }
.icon-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-btn);
  color: var(--icon);
  cursor: pointer;
  transition: background 0.12s, color 0.12s, border-color 0.12s;
}
.icon-btn:hover { background: var(--btn-bg); border-color: var(--btn-border); color: var(--text); }
.icon-btn:focus-visible { outline: 2px solid var(--blue); outline-offset: 1px; }

.dropdown {
  position: absolute;
  left: 0;
  bottom: calc(100% + 4px);
  z-index: 1100;
  min-width: 140px;
  list-style: none;
  margin: 0;
  padding: 4px 0;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: var(--modal-shadow);
}
.dropdown li {
  padding: 8px 14px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
}
.dropdown li:hover { background: var(--accent-hover); }
.dropdown li.active { color: var(--blue); font-weight: 600; }

.version {
  font-size: 11px;
  color: var(--text-3);
  text-align: center;
}
</style>
