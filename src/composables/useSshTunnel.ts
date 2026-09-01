// SSH 隧道状态管理（单例 composable）
// 项目未安装 Pinia，故用模块级 reactive 单例实现全局状态。

import { computed, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { t } from './usePrefs';
import type { ConnState, ForwardRule, HostProfile, LogEntry, LogLevel, RuleType } from '../types';
import {
  describeRule,
  mockConnectSequence,
  mockDisconnectSequence,
  mockHosts,
  pickMockKeyPath,
} from '../mock/mockData';

interface TunnelState {
  hosts: HostProfile[];
  currentHostId: string;
  connState: ConnState;
  activeRuleType: RuleType;
  showPassword: boolean;
  logs: LogEntry[];
  logSeq: number;
  /** 表单字段错误提示 key -> message */
  errors: Record<string, string>;
  /** 当前正在编辑的规则 id（null 表示弹窗关闭） */
  editingRuleId: string | null;
}

const state = reactive<TunnelState>({
  hosts: mockHosts,
  currentHostId: mockHosts[0]?.id ?? '',
  connState: 'idle',
  activeRuleType: 'local',
  showPassword: false,
  logs: [],
  logSeq: 0,
  errors: {},
  editingRuleId: null,
});

/** 生成自增日志 id */
function nextLogId(): number {
  return ++state.logSeq;
}

/** 当前时间 HH:mm:ss */
function nowTime(): string {
  const d = new Date();
  const p = (n: number) => n.toString().padStart(2, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

/** 追加一条日志 */
function addLog(level: LogLevel, message: string): void {
  state.logs.push({ id: nextLogId(), time: nowTime(), level, message });
}

/** 当前选中的主机（响应式，列表为空时为 undefined） */
const currentHost = computed<HostProfile | undefined>(
  () => state.hosts.find((h) => h.id === state.currentHostId) ?? state.hosts[0],
);

/** 当前 Tab 类型下的规则列表 */
const currentRules = computed<ForwardRule[]>(
  () => currentHost.value?.rules.filter((r) => r.type === state.activeRuleType) ?? [],
);

/** 当前正在编辑的规则（跨所有类型查找） */
const editingRule = computed<ForwardRule | undefined>(() =>
  state.editingRuleId
    ? currentHost.value?.rules.find((r) => r.id === state.editingRuleId)
    : undefined,
);

/** 生成新规则 id */
function genRuleId(): string {
  return 'r-' + Math.random().toString(36).slice(2, 9);
}

/** 切换当前主机 */
function selectHost(id: string): void {
  if (id === state.currentHostId) return;
  state.currentHostId = id;
  state.errors = {};
  const h = currentHost.value;
  addLog('INFO', t('logmsg.switchHost', { name: h?.name ?? t('common.unknown'), host: h?.host ?? '' }));
}

/** 保存当前主机配置 */
function saveHost(): void {
  const h = currentHost.value;
  if (!h) return;
  addLog('SUCCESS', t('logmsg.saveHost', { name: h.name }));
}

/** 删除当前主机（允许删到空列表） */
function deleteHost(): void {
  const h = currentHost.value;
  if (!h) return;
  const idx = state.hosts.findIndex((x) => x.id === h.id);
  state.hosts.splice(idx, 1);
  state.currentHostId = state.hosts[0]?.id ?? '';
  addLog('WARN', t('logmsg.deleteHost', { name: h.name }));
}

/** 切换 Tab */
function setActiveRuleType(type: RuleType): void {
  state.activeRuleType = type;
}

/** 新增一条规则（属于当前 Tab 类型） */
function addRule(): void {
  const h = currentHost.value;
  if (!h) return;
  const rule: ForwardRule = {
    id: genRuleId(),
    type: state.activeRuleType,
    enabled: true,
    listenAddr: '127.0.0.1:',
    targetAddr: state.activeRuleType === 'dynamic' ? '' : '127.0.0.1:',
    note: '',
  };
  h.rules.push(rule);
  addLog('INFO', t('logmsg.addRule', { desc: describeRule(t('logmsg.draft'), rule.type, rule.listenAddr, rule.targetAddr) }));
}

/** 删除一条规则 */
function removeRule(id: string): void {
  const h = currentHost.value;
  if (!h) return;
  const idx = h.rules.findIndex((r) => r.id === id);
  if (idx < 0) return;
  const [removed] = h.rules.splice(idx, 1);
  addLog('WARN', t('logmsg.removeRule', { desc: describeRule('', removed.type, removed.listenAddr, removed.targetAddr) }));
}

/** 切换规则启用状态 */
function toggleRule(id: string): void {
  const h = currentHost.value;
  if (!h) return;
  const rule = h.rules.find((r) => r.id === id);
  if (!rule) return;
  rule.enabled = !rule.enabled;
  addLog('INFO', t('logmsg.toggleRule', { desc: describeRule(rule.enabled ? t('logmsg.enabled') : t('logmsg.disabled'), rule.type, rule.listenAddr, rule.targetAddr) }));
}

/** 行内编辑写入（失焦触发，记录日志） */
function commitRuleEdit(id: string, field: 'listenAddr' | 'targetAddr' | 'note', value: string): void {
  const h = currentHost.value;
  if (!h) return;
  const rule = h.rules.find((r) => r.id === id);
  if (!rule) return;
  if (rule[field] === value) return;
  rule[field] = value;
  const fieldLabel = { listenAddr: t('common.listenAddr'), targetAddr: t('common.targetAddr'), note: t('common.note') }[field];
  addLog('INFO', t('logmsg.updateRule', { desc: describeRule('', rule.type, rule.listenAddr, rule.targetAddr), field: fieldLabel }));
}

/** 打开规则编辑弹窗 */
function openEditModal(ruleId: string): void {
  state.editingRuleId = ruleId;
}

/** 关闭规则编辑弹窗 */
function closeEditModal(): void {
  state.editingRuleId = null;
}

/** 保存规则编辑（含转发类型变更，类型变更后自动切到对应 Tab） */
function saveRuleEdit(data: {
  type: RuleType;
  listenAddr: string;
  targetAddr: string;
  note: string;
}): void {
  if (!state.editingRuleId) return;
  const h = currentHost.value;
  if (!h) return;
  const rule = h.rules.find((r) => r.id === state.editingRuleId);
  if (!rule) return;
  const typeChanged = rule.type !== data.type;
  rule.type = data.type;
  rule.listenAddr = data.listenAddr.trim();
  rule.targetAddr = data.type === 'dynamic' ? '' : data.targetAddr.trim();
  rule.note = data.note.trim();
  if (typeChanged) state.activeRuleType = data.type;
  addLog('INFO', t('logmsg.modifyRule', { desc: describeRule('', rule.type, rule.listenAddr, rule.targetAddr) }));
  state.editingRuleId = null;
}

/** 切换密码明文显示 */
function toggleShowPassword(): void {
  state.showPassword = !state.showPassword;
}

/** 表单校验 */
function validate(): boolean {
  const h = currentHost.value;
  state.errors = {};
  if (!h) {
    state.errors.host = t('logmsg.noHost');
    return false;
  }
  if (!h.host.trim()) state.errors.host = t('logmsg.hostRequired');
  if (!h.port || h.port <= 0 || h.port > 65535) state.errors.port = t('logmsg.portInvalid');
  if (!h.username.trim()) state.errors.username = t('logmsg.usernameRequired');
  return Object.keys(state.errors).length === 0;
}

/** 启动隧道（连接） */
async function startTunnel(): Promise<void> {
  if (state.connState !== 'idle' && state.connState !== 'error') return;
  if (!validate()) {
    addLog('ERROR', t('logmsg.validateFail'));
    return;
  }
  const h = currentHost.value;
  if (!h) return;
  state.connState = 'connecting';
  addLog('INFO', t('logmsg.connecting', { host: h.host, port: h.port, username: h.username }));
  try {
    // TODO: 对接 src-tauri 命令 start_tunnel
    await invoke('start_tunnel', { hostId: h.id, host: h.host, port: h.port, username: h.username });
  } catch {
    // 回落到 mockData：模拟连接过程
    await mockConnectSequence(h, addLog);
  }
  state.connState = 'connected';
  addLog('SUCCESS', t('logmsg.connected', { host: h.host, port: h.port }));
  const enabled = h.rules.filter((r) => r.enabled);
  if (enabled.length === 0) {
    addLog('WARN', t('logmsg.noEnabledRule'));
    return;
  }
  for (const r of enabled) {
    addLog('SUCCESS', t('logmsg.established', { desc: describeRule('', r.type, r.listenAddr, r.targetAddr) }));
  }
}

/** 关闭隧道（断开） */
async function stopTunnel(): Promise<void> {
  if (state.connState !== 'connected') return;
  const h = currentHost.value;
  if (!h) return;
  state.connState = 'disconnecting';
  addLog('INFO', t('logmsg.closing'));
  try {
    // TODO: 对接 src-tauri 命令 stop_tunnel
    await invoke('stop_tunnel', { hostId: h.id });
  } catch {
    // 回落到 mockData：模拟断开过程
    await mockDisconnectSequence(h, addLog);
  }
  state.connState = 'idle';
  addLog('SUCCESS', t('logmsg.disconnected', { host: h.host }));
}

/** 选择私钥文件 */
async function pickKeyFile(): Promise<void> {
  const h = currentHost.value;
  if (!h) return;
  try {
    // TODO: 对接 src-tauri 命令 pick_key_file（或 dialog::open）
    const path = await invoke<string>('pick_key_file');
    if (path) {
      h.keyPath = path;
      addLog('INFO', t('logmsg.pickKey', { path }));
    }
  } catch {
    // 回落到 mockData：回填一个假路径
    const path = pickMockKeyPath();
    h.keyPath = path;
    addLog('INFO', t('logmsg.pickKeyMock', { path }));
  }
}

/** 读取后端日志（拉取额外日志并合并） */
async function readLogs(): Promise<void> {
  try {
    // TODO: 对接 src-tauri 命令 read_logs
    const lines = await invoke<string[]>('read_logs');
    for (const line of lines) addLog('INFO', line);
  } catch {
    // 回落到 mockData：无额外日志可读
  }
}

/** 清空日志 */
function clearLogs(): void {
  state.logs.splice(0, state.logs.length);
  // addLog('INFO', '日志已清空');
}

/** 拼接全部日志文本（用于复制） */
function logsText(): string {
  return state.logs.map((l) => `[${l.time}] ${l.level} ${l.message}`).join('\n');
}

// 初始化日志
addLog('INFO', t('logmsg.loading'));
addLog('INFO', t('logmsg.loaded', { n: state.hosts.length }));
addLog('INFO', t('logmsg.waiting'));

/** 单例 hook */
export function useSshTunnel() {
  return {
    state,
    currentHost,
    currentRules,
    selectHost,
    saveHost,
    deleteHost,
    setActiveRuleType,
    addRule,
    removeRule,
    toggleRule,
    commitRuleEdit,
    openEditModal,
    closeEditModal,
    saveRuleEdit,
    editingRule,
    toggleShowPassword,
    startTunnel,
    stopTunnel,
    pickKeyFile,
    readLogs,
    clearLogs,
    logsText,
  };
}

