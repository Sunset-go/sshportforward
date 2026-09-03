// 应用偏好单例（语言 + 主题）
// 项目未安装 Pinia / vue-i18n，故用模块级 reactive 单例 + 手写字典实现轻量 i18n 与主题切换。

import { computed, reactive, watch } from 'vue';

export type Locale = 'zh' | 'en';
export type ThemeMode = 'dark' | 'light';

const LOCALE_KEY = 'sshpf:locale';
const THEME_KEY = 'sshpf:theme';

function readStorage(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function writeStorage(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* 忽略存储失败 */
  }
}

function readLocale(): Locale {
  return readStorage(LOCALE_KEY) === 'en' ? 'en' : 'zh';
}

function readTheme(): ThemeMode {
  return readStorage(THEME_KEY) === 'light' ? 'light' : 'dark';
}

const state = reactive({
  locale: readLocale(),
  theme: readTheme(),
});

/** 中英文案字典 */
const messages: Record<Locale, Record<string, string>> = {
  zh: {
    'common.save': '保存',
    'common.delete': '删除',
    'common.cancel': '取消',
    'common.close': '关闭',
    'common.local': '本地',
    'common.remote': '远端',
    'common.dynamic': '动态',
    'common.note': '备注',
    'common.actions': '操作',
    'common.enable': '启用',
    'common.listenAddr': '监听地址',
    'common.targetAddr': '目标地址',
    'common.remoteAddr': '远端地址',
    'common.localAddr': '本地地址',
    'common.unnamed': '未命名主机',
    'common.unknown': '未知',

    'hostbar.title': '已保存主机',
    'hostbar.namePlaceholder': '主机名称',
    'hostbar.addHost': '添加主机',

    'lang.title': '语言',
    'lang.zh': '简体中文',
    'lang.en': 'English',

    'theme.title': '主题',
    'theme.dark': '深色',
    'theme.light': '浅色',

    'conn.title': 'SSH 连接信息',
    'conn.subtitle': '填写远端主机凭据以建立隧道',
    'conn.host': '主机 Host',
    'conn.port': '端口 Port',
    'conn.username': '用户名',
    'conn.password': '密码',
    'conn.keyPath': '私钥路径',
    'conn.keyPathPlaceholder': '可选，留空使用密码认证',
    'conn.browse': '浏览',
    'conn.start': '启动',
    'conn.stop': '关闭',
    'conn.connecting': '连接中…',
    'conn.show': '显示',
    'conn.hide': '隐藏',

    'pf.title': '端口转发规则',
    'pf.add': '添加规则',

    'rt.targetRemote': '远端地址 → 本地地址',
    'rt.arrow': '远端 → 本地',
    'rt.upload': '上传',
    'rt.download': '下载',
    'rt.empty.local': '暂无本地转发规则',
    'rt.empty.remote': '暂无远端转发规则',
    'rt.empty.dynamic': '暂无动态转发规则',
    'rt.edit': '修改',
    'rt.delete': '删除',

    'rm.title': '修改转发规则',
    'rm.type': '转发类型',
    'rm.notePlaceholder': '可选备注',

    'log.title': '运行日志',
    'log.count': '{n} 条',
    'log.copy': '复制全部',
    'log.copied': '已复制',
    'log.clear': '清空',
    'log.empty': '暂无日志',

    'status.idle': '未连接',
    'status.connecting': '连接中',
    'status.connected': '已连接',
    'status.error': '连接失败',
    'status.disconnecting': '断开中',

    'logmsg.switchHost': '已切换主机：{name}（{host}）',
    'logmsg.saveHost': '已保存主机「{name}」配置',
    'logmsg.deleteHost': '已删除主机「{name}」',
    'logmsg.addRule': '已新增规则 {desc}',
    'logmsg.draft': '草稿',
    'logmsg.removeRule': '已删除规则 {desc}',
    'logmsg.toggleRule': '规则 {desc}',
    'logmsg.enabled': '已启用',
    'logmsg.disabled': '已停用',
    'logmsg.updateRule': '规则已更新 {desc}（{field}）',
    'logmsg.modifyRule': '规则已修改 {desc}',
    'logmsg.noHost': '未选择主机',
    'logmsg.hostRequired': '主机地址不能为空',
    'logmsg.portInvalid': '端口需为 1-65535',
    'logmsg.usernameRequired': '用户名不能为空',
    'logmsg.validateFail': '表单校验失败，请检查必填项',
    'logmsg.connecting': '正在连接 {host}:{port}（用户 {username}）…',
    'logmsg.connected': '已连接到 {host}:{port}',
    'logmsg.noEnabledRule': '当前没有启用的转发规则',
    'logmsg.established': '已建立转发 {desc}',
    'logmsg.closing': '正在关闭所有转发通道…',
    'logmsg.disconnected': '已断开与 {host} 的连接',
    'logmsg.pickKey': '已选择私钥：{path}',
    'logmsg.loading': '载入主机配置…',
    'logmsg.loaded': '已载入 {n} 个主机配置',
    'logmsg.waiting': '等待连接…',
    'logmsg.createdDefaultHost': '已自动创建默认主机「{name}」',
    'logmsg.addHost': '已添加主机「{name}」',
    'logmsg.saveHostFirst': '请先保存主机配置',
    'logmsg.discardHost': '已丢弃未保存的主机「{name}」',

    'logmsg.err.addHost': '添加主机失败',
    'logmsg.err.saveRule': '保存规则失败',
    'logmsg.err.saveHost': '保存主机失败',
    'logmsg.err.deleteHost': '删除主机失败',
    'logmsg.err.addRule': '新增规则失败',
    'logmsg.err.removeRule': '删除规则失败',
    'logmsg.err.updateRule': '保存规则字段失败',
    'logmsg.err.connect': '连接失败',
    'logmsg.err.disconnect': '断开失败',
    'logmsg.err.pickKey': '选择私钥失败',
    'logmsg.err.loadHosts': '加载主机列表失败',

    'tray.show': '显示主窗口',
    'tray.closeToTray': '关闭时最小化到托盘',
    'tray.quit': '退出',

    'closeDlg.title': '关闭程序',
    'closeDlg.message': '是否将程序最小化到系统托盘？此选择将被记住，之后关闭窗口不再询问；可随时在托盘菜单中修改。',
    'closeDlg.toTray': '最小化到托盘',
    'closeDlg.exit': '退出程序',

    'startDlg.title': '开启最小化到托盘？',
    'startDlg.message': '当前未开启"关闭时最小化到托盘"。开启后，点击关闭窗口时程序将收容到托盘，隧道保持运行。是否开启？',
    'startDlg.enable': '开启',
  },
  en: {
    'common.save': 'Save',
    'common.delete': 'Delete',
    'common.cancel': 'Cancel',
    'common.close': 'Close',
    'common.local': 'Local',
    'common.remote': 'Remote',
    'common.dynamic': 'Dynamic',
    'common.note': 'Note',
    'common.actions': 'Actions',
    'common.enable': 'Enable',
    'common.listenAddr': 'Listen Address',
    'common.targetAddr': 'Target Address',
    'common.remoteAddr': 'Remote Address',
    'common.localAddr': 'Local Address',
    'common.unnamed': 'Untitled Host',
    'common.unknown': 'Unknown',

    'hostbar.title': 'Saved Hosts',
    'hostbar.namePlaceholder': 'Host name',
    'hostbar.addHost': 'Add Host',

    'lang.title': 'Language',
    'lang.zh': '简体中文',
    'lang.en': 'English',

    'theme.title': 'Theme',
    'theme.dark': 'Dark',
    'theme.light': 'Light',

    'conn.title': 'SSH Connection',
    'conn.subtitle': 'Fill in remote host credentials to establish the tunnel',
    'conn.host': 'Host',
    'conn.port': 'Port',
    'conn.username': 'Username',
    'conn.password': 'Password',
    'conn.keyPath': 'Private Key Path',
    'conn.keyPathPlaceholder': 'Optional, leave blank to use password auth',
    'conn.browse': 'Browse',
    'conn.start': 'Start',
    'conn.stop': 'Stop',
    'conn.connecting': 'Connecting…',
    'conn.show': 'Show',
    'conn.hide': 'Hide',

    'pf.title': 'Port Forward Rules',
    'pf.add': 'Add Rule',

    'rt.targetRemote': 'Remote → Local',
    'rt.arrow': 'Remote → Local',
    'rt.upload': 'Up',
    'rt.download': 'Down',
    'rt.empty.local': 'No local forward rules',
    'rt.empty.remote': 'No remote forward rules',
    'rt.empty.dynamic': 'No dynamic forward rules',
    'rt.edit': 'Edit',
    'rt.delete': 'Delete',

    'rm.title': 'Edit Forward Rule',
    'rm.type': 'Forward Type',
    'rm.notePlaceholder': 'Optional note',

    'log.title': 'Runtime Logs',
    'log.count': '{n} entries',
    'log.copy': 'Copy All',
    'log.copied': 'Copied',
    'log.clear': 'Clear',
    'log.empty': 'No logs',

    'status.idle': 'Idle',
    'status.connecting': 'Connecting',
    'status.connected': 'Connected',
    'status.error': 'Connection Failed',
    'status.disconnecting': 'Disconnecting',

    'logmsg.switchHost': 'Switched host: {name} ({host})',
    'logmsg.saveHost': 'Saved host "{name}" config',
    'logmsg.deleteHost': 'Deleted host "{name}"',
    'logmsg.addRule': 'Added rule {desc}',
    'logmsg.draft': 'draft',
    'logmsg.removeRule': 'Removed rule {desc}',
    'logmsg.toggleRule': 'Rule {desc}',
    'logmsg.enabled': 'enabled',
    'logmsg.disabled': 'disabled',
    'logmsg.updateRule': 'Rule updated {desc} ({field})',
    'logmsg.modifyRule': 'Rule modified {desc}',
    'logmsg.noHost': 'No host selected',
    'logmsg.hostRequired': 'Host address cannot be empty',
    'logmsg.portInvalid': 'Port must be 1-65535',
    'logmsg.usernameRequired': 'Username cannot be empty',
    'logmsg.validateFail': 'Validation failed, please check required fields',
    'logmsg.connecting': 'Connecting to {host}:{port} (user {username})…',
    'logmsg.connected': 'Connected to {host}:{port}',
    'logmsg.noEnabledRule': 'No enabled forward rules',
    'logmsg.established': 'Established forward {desc}',
    'logmsg.closing': 'Closing all forward channels…',
    'logmsg.disconnected': 'Disconnected from {host}',
    'logmsg.pickKey': 'Selected private key: {path}',
    'logmsg.loading': 'Loading host config…',
    'logmsg.loaded': 'Loaded {n} host configs',
    'logmsg.waiting': 'Waiting for connection…',
    'logmsg.createdDefaultHost': 'Created default host "{name}"',
    'logmsg.addHost': 'Added host "{name}"',
    'logmsg.saveHostFirst': 'Please save the host config first',
    'logmsg.discardHost': 'Discarded unsaved host "{name}"',

    'logmsg.err.addHost': 'Failed to add host',
    'logmsg.err.saveRule': 'Failed to save rule',
    'logmsg.err.saveHost': 'Failed to save host',
    'logmsg.err.deleteHost': 'Failed to delete host',
    'logmsg.err.addRule': 'Failed to add rule',
    'logmsg.err.removeRule': 'Failed to remove rule',
    'logmsg.err.updateRule': 'Failed to update rule field',
    'logmsg.err.connect': 'Connection failed',
    'logmsg.err.disconnect': 'Disconnect failed',
    'logmsg.err.pickKey': 'Failed to select private key',
    'logmsg.err.loadHosts': 'Failed to load hosts',

    'tray.show': 'Show Main Window',
    'tray.closeToTray': 'Minimize to Tray on Close',
    'tray.quit': 'Quit',

    'closeDlg.title': 'Close Application',
    'closeDlg.message': 'Minimize to the system tray instead of exiting? This choice will be remembered and you will not be asked again; you can change it anytime from the tray menu.',
    'closeDlg.toTray': 'Minimize to Tray',
    'closeDlg.exit': 'Exit',

    'startDlg.title': 'Enable Minimize to Tray?',
    'startDlg.message': '"Minimize to Tray on Close" is not enabled yet. Once enabled, closing the window keeps the app in the tray and the tunnel running. Enable it now?',
    'startDlg.enable': 'Enable',
  },
};

/** 翻译函数，支持 {name} 插值 */
export function t(key: string, params?: Record<string, string | number>): string {
  const template = messages[state.locale]?.[key] ?? messages.zh[key] ?? key;
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_m, name: string) =>
    params[name] !== undefined ? String(params[name]) : '',
  );
}

/** 将主题写入 <html data-theme="...">，供 CSS 变量切换 */
function applyTheme(theme: ThemeMode): void {
  document.documentElement.setAttribute('data-theme', theme);
}

// 模块加载即应用持久化主题，避免闪烁
applyTheme(state.theme);

watch(
  () => state.theme,
  (v) => {
    writeStorage(THEME_KEY, v);
    applyTheme(v);
  },
);
watch(
  () => state.locale,
  (v) => writeStorage(LOCALE_KEY, v),
);

function setLocale(locale: Locale): void {
  state.locale = locale;
}

function setTheme(theme: ThemeMode): void {
  state.theme = theme;
}

/** 单例 hook */
export function usePrefs() {
  return {
    locale: computed(() => state.locale),
    theme: computed(() => state.theme),
    t,
    setLocale,
    setTheme,
  };
}
