// 应用偏好单例（主题/语言/最小化到托盘/开机自启）。
// 持久化到后端 settings.json（theme/locale/autoStart）+ DB close_to_tray（minimizeToTray）；
// localStorage 镜像用于 index.html 首屏防闪烁脚本。
// 项目未安装 Pinia，沿用 useSshTunnel 的模块级 reactive 单例范式。

import { reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type Locale = 'zh' | 'en';
export type ThemeMode = 'dark' | 'light';

export interface Settings {
  theme: ThemeMode;
  locale: Locale;
  minimizeToTray: boolean;
  autoStart: boolean;
}

/** localStorage 镜像 key（与 index.html 防闪脚本约定一致） */
const LS_THEME = 'sshpf:theme';
const LS_LOCALE = 'sshpf:locale';

export const settings = reactive<Settings>({
  theme: 'dark',
  locale: 'zh',
  minimizeToTray: false,
  autoStart: false,
});

let saveTimer: ReturnType<typeof setTimeout> | null = null;
let trayListenerReady = false;

/**
 * 注册托盘菜单 → 前端的单向同步监听器（仅一次）。
 * 用户在系统托盘菜单中切换"关闭时最小化到托盘"时，后端 emit
 * `close-to-tray-changed` 事件，此处实时更新响应式 settings，使设置面板
 * 的复选框与托盘菜单勾选态保持一致。
 */
async function ensureTrayListener(): Promise<void> {
  if (trayListenerReady) return;
  try {
    const unlisten: UnlistenFn = await listen<boolean>('close-to-tray-changed', (e) => {
      settings.minimizeToTray = e.payload;
    });
    // 单例应用，监听器生命周期与进程一致；保留引用避免 lint 警告
    void unlisten;
    trayListenerReady = true;
  } catch (e) {
    console.error('注册托盘设置监听失败', e);
  }
}

function lsWrite(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* 忽略存储失败 */
  }
}

/** 应用到 DOM（data-theme / <html lang>）并镜像到 localStorage 供首屏防闪 */
function apply(): void {
  document.documentElement.setAttribute('data-theme', settings.theme);
  document.documentElement.lang = settings.locale === 'zh' ? 'zh-CN' : 'en';
  lsWrite(LS_THEME, settings.theme);
  lsWrite(LS_LOCALE, settings.locale);
}

/** 从后端读取偏好并应用 */
export async function load(): Promise<void> {
  try {
    const s = await invoke<Settings>('get_settings');
    settings.theme = (s.theme === 'light' ? 'light' : 'dark');
    settings.locale = (s.locale === 'en' ? 'en' : 'zh');
    settings.minimizeToTray = !!s.minimizeToTray;
    settings.autoStart = !!s.autoStart;
  } catch (e) {
    console.error('加载设置失败', e);
  }
  apply();
  // 注册托盘菜单变更监听（幂等，仅首次生效）
  void ensureTrayListener();
}

/** 局部更新：立即应用 + 防抖落盘（save_settings 会把 minimizeToTray 桥接到 DB） */
export function patch(partial: Partial<Settings>): void {
  Object.assign(settings, partial);
  apply();
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    invoke('save_settings', { settings: { ...settings } }).catch((e) =>
      console.error('保存设置失败', e),
    );
  }, 300);
}

/** 在深/浅主题间切换 */
export function toggleTheme(): void {
  patch({ theme: settings.theme === 'dark' ? 'light' : 'dark' });
}

/** 切换语言 */
export function setLocale(locale: Locale): void {
  patch({ locale });
}

/** 单例 hook */
export function useSettings() {
  return { settings, load, patch, toggleTheme, setLocale };
}
