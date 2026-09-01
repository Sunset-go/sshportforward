// SSH 隧道状态管理（单例 composable）
// 项目未安装 Pinia，故用模块级 reactive 单例实现全局状态。

import { computed, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { t } from './usePrefs';
import { describeRule, genId } from '../utils';
import type { ConnState, ForwardRule, HostProfile, LogEntry, LogLevel, RuleType } from '../types';

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
  /** 已修改但尚未持久化的规则 id 集合 */
  dirtyRules: Set<string>;
  /** 初始化是否完成 */
  initialized: boolean;
}

const state = reactive<TunnelState>({
  hosts: [],
  currentHostId: '',
  connState: 'idle',
  activeRuleType: 'local',
  showPassword: false,
  logs: [],
  logSeq: 0,
  errors: {},
  editingRuleId: null,
  dirtyRules: new Set(),
  initialized: false,
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

/** 追加一条日志（双写：本地内存 + 后端 DB） */
async function addLog(level: LogLevel, message: string): Promise<void> {
  const entry: LogEntry = { id: nextLogId(), time: nowTime(), level, message };
  state.logs.push(entry);
  try {
    await invoke('add_log', { level, message });
  } catch (err) {
    // 后端写入失败不影响前端显示，仅输出错误信息
    console.error('[add_log] 写入后端日志失败:', err);
  }
}

/** 创建一个空的默认主机（未持久化） */
function createDefaultHost(): HostProfile {
  return {
    id: genId('host'),
    name: t('common.unnamed'),
    host: '',
    port: 22,
    username: '',
    password: '',
    keyPath: '',
    rules: [],
  };
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

/** 切换当前主机 */
function selectHost(id: string): void {
  if (id === state.currentHostId) return;
  state.currentHostId = id;
  state.errors = {};
  const h = currentHost.value;
  addLog('INFO', t('logmsg.switchHost', { name: h?.name ?? t('common.unknown'), host: h?.host ?? '' }));
}

/** 新增一台主机，持久化到后端并设为当前主机 */
async function addHost(): Promise<void> {
  const draft = createDefaultHost();
  try {
    const saved = await invoke<HostProfile>('save_host', {
      input: {
        id: undefined,
        name: draft.name,
        host: draft.host,
        port: draft.port,
        username: draft.username,
        password: draft.password,
        keyPath: draft.keyPath,
      },
    });
    state.hosts.push(saved);
    state.currentHostId = saved.id;
    addLog('INFO', t('logmsg.addHost', { name: saved.name }));
  } catch (err) {
    addLog('ERROR', `添加主机失败: ${err}`);
  }
}

/** 保存当前主机配置 */
async function saveHost(): Promise<void> {
  const h = currentHost.value;
  if (!h) return;

  // 同步 dirty 规则到后端
  for (const ruleId of state.dirtyRules) {
    const rule = h.rules.find((r) => r.id === ruleId);
    if (!rule) continue;
    try {
      await invoke('update_rule', {
        hostId: h.id,
        ruleId: rule.id,
        input: {
          type: rule.type,
          enabled: rule.enabled,
          listenAddr: rule.listenAddr,
          targetAddr: rule.targetAddr,
          note: rule.note,
        },
      });
    } catch (err) {
      addLog('ERROR', `保存规则失败: ${err}`);
      return;
    }
  }
  state.dirtyRules.clear();

  // 保存主机基本信息
  try {
    await invoke('save_host', {
      input: {
        id: h.id,
        name: h.name,
        host: h.host,
        port: h.port,
        username: h.username,
        password: h.password,
        keyPath: h.keyPath,
      },
    });
    addLog('SUCCESS', t('logmsg.saveHost', { name: h.name }));
  } catch (err) {
    addLog('ERROR', `保存主机失败: ${err}`);
  }
}

/** 删除当前主机（先删后端，成功再删本地） */
async function deleteHost(): Promise<void> {
  const h = currentHost.value;
  if (!h) return;

  try {
    await invoke('delete_host', { hostId: h.id });
  } catch (err) {
    addLog('ERROR', `删除主机失败: ${err}`);
    return;
  }

  const idx = state.hosts.findIndex((x) => x.id === h.id);
  state.hosts.splice(idx, 1);
  state.currentHostId = state.hosts[0]?.id ?? '';
  addLog('WARN', t('logmsg.deleteHost', { name: h.name }));
}

/** 切换 Tab */
function setActiveRuleType(type: RuleType): void {
  state.activeRuleType = type;
}

/** 新增一条规则（后端生成 ID，成功后加入列表） */
async function addRule(): Promise<void> {
  const h = currentHost.value;
  if (!h) return;

  try {
    const saved = await invoke<ForwardRule>('add_rule', {
      hostId: h.id,
      input: {
        type: state.activeRuleType,
        enabled: true,
        listenAddr: '127.0.0.1:',
        targetAddr: state.activeRuleType === 'dynamic' ? '' : '127.0.0.1:',
        note: '',
      },
    });
    h.rules.push(saved);
    addLog('INFO', t('logmsg.addRule', { desc: describeRule(t('logmsg.draft'), saved.type, saved.listenAddr, saved.targetAddr) }));
  } catch (err) {
    addLog('ERROR', `新增规则失败: ${err}`);
  }
}

/** 删除一条规则（先删后端，成功再删本地） */
async function removeRule(id: string): Promise<void> {
  const h = currentHost.value;
  if (!h) return;

  const rule = h.rules.find((r) => r.id === id);
  if (!rule) return;

  try {
    await invoke('delete_rule', { hostId: h.id, ruleId: id });
  } catch (err) {
    addLog('ERROR', `删除规则失败: ${err}`);
    return;
  }

  state.dirtyRules.delete(id);
  h.rules.splice(h.rules.findIndex((r) => r.id === id), 1);
  addLog('WARN', t('logmsg.removeRule', { desc: describeRule('', rule.type, rule.listenAddr, rule.targetAddr) }));
}

/** 切换规则启用状态（仅更新本地，标记 dirty） */
function toggleRule(id: string): void {
  const h = currentHost.value;
  if (!h) return;
  const rule = h.rules.find((r) => r.id === id);
  if (!rule) return;
  rule.enabled = !rule.enabled;
  state.dirtyRules.add(id);
  addLog('INFO', t('logmsg.toggleRule', { desc: describeRule(rule.enabled ? t('logmsg.enabled') : t('logmsg.disabled'), rule.type, rule.listenAddr, rule.targetAddr) }));
}

/** 行内编辑写入（失焦触发，构造完整 RuleInput 调 update_rule） */
async function commitRuleEdit(id: string, field: 'listenAddr' | 'targetAddr' | 'note', value: string): Promise<void> {
  const h = currentHost.value;
  if (!h) return;
  const rule = h.rules.find((r) => r.id === id);
  if (!rule) return;
  if (rule[field] === value) return;
  rule[field] = value;
  state.dirtyRules.add(id);
  try {
    await invoke('update_rule', {
      hostId: h.id,
      ruleId: id,
      input: {
        type: rule.type,
        enabled: rule.enabled,
        listenAddr: rule.listenAddr,
        targetAddr: rule.targetAddr,
        note: rule.note,
      },
    });
  } catch (err) {
    addLog('ERROR', `保存规则字段失败: ${err}`);
  }
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
async function saveRuleEdit(data: {
  type: RuleType;
  listenAddr: string;
  targetAddr: string;
  note: string;
}): Promise<void> {
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
  state.dirtyRules.add(rule.id);

  try {
    await invoke('update_rule', {
      hostId: h.id,
      ruleId: rule.id,
      input: {
        type: rule.type,
        enabled: rule.enabled,
        listenAddr: rule.listenAddr,
        targetAddr: rule.targetAddr,
        note: rule.note,
      },
    });
  } catch (err) {
    addLog('ERROR', `保存规则失败: ${err}`);
  }
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
  await addLog('INFO', t('logmsg.connecting', { host: h.host, port: h.port, username: h.username }));

  // 先同步 dirty 规则
  for (const ruleId of state.dirtyRules) {
    const rule = h.rules.find((r) => r.id === ruleId);
    if (!rule) continue;
    try {
      await invoke('update_rule', {
        hostId: h.id,
        ruleId: rule.id,
        input: {
          type: rule.type,
          enabled: rule.enabled,
          listenAddr: rule.listenAddr,
          targetAddr: rule.targetAddr,
          note: rule.note,
        },
      });
    } catch (err) {
      addLog('ERROR', `保存规则失败: ${err}`);
    }
  }
  state.dirtyRules.clear();

  try {
    await invoke('start_tunnel', { hostId: h.id, host: h.host, port: h.port, username: h.username });
    const connState = await invoke<string>('get_conn_state');
    state.connState = connState as ConnState;
  } catch (err) {
    state.connState = 'error';
    addLog('ERROR', `连接失败: ${err}`);
    return;
  }
  await addLog('SUCCESS', t('logmsg.connected', { host: h.host, port: h.port }));
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
  await addLog('INFO', t('logmsg.closing'));

  try {
    await invoke('stop_tunnel', { hostId: h.id });
    const connState = await invoke<string>('get_conn_state');
    state.connState = connState as ConnState;
  } catch (err) {
    state.connState = 'error';
    addLog('ERROR', `断开失败: ${err}`);
    return;
  }
  await addLog('SUCCESS', t('logmsg.disconnected', { host: h.host }));
}

/** 选择私钥文件 */
async function pickKeyFile(): Promise<void> {
  const h = currentHost.value;
  if (!h) return;
  try {
    const path = await invoke<string>('pick_key_file');
    if (path) {
      h.keyPath = path;
      await addLog('INFO', t('logmsg.pickKey', { path }));
    }
  } catch (err) {
    addLog('ERROR', `选择私钥失败: ${err}`);
  }
}

/** 读取后端日志（手动拉取历史日志） */
async function readLogs(): Promise<void> {
  try {
    const lines = await invoke<string[]>('read_logs');
    for (const line of lines) {
      state.logs.push({ id: nextLogId(), time: nowTime(), level: 'INFO', message: line });
    }
  } catch (err) {
    addLog('ERROR', `读取后端日志失败: ${err}`);
  }
}

/** 清空日志 */
function clearLogs(): void {
  state.logs.splice(0, state.logs.length);
}

/** 拼接全部日志文本（用于复制） */
function logsText(): string {
  return state.logs.map((l) => `[${l.time}] ${l.level} ${l.message}`).join('\n');
}

/** 从后端加载主机列表（初始化用）。若数据库为空则自动创建一台默认主机并持久化。 */
async function loadHosts(): Promise<void> {
  try {
    const hosts = await invoke<HostProfile[]>('list_hosts');
    if (hosts.length === 0) {
      const draft = createDefaultHost();
      const saved = await invoke<HostProfile>('save_host', {
        input: {
          id: undefined,
          name: draft.name,
          host: draft.host,
          port: draft.port,
          username: draft.username,
          password: draft.password,
          keyPath: draft.keyPath,
        },
      });
      state.hosts = [saved];
      state.currentHostId = saved.id;
      await addLog('INFO', t('logmsg.createdDefaultHost', { name: saved.name }));
    } else {
      state.hosts = hosts;
      state.currentHostId = hosts[0]?.id ?? '';
    }
    state.initialized = true;
    await addLog('INFO', t('logmsg.loaded', { n: state.hosts.length }));
    await addLog('INFO', t('logmsg.waiting'));
  } catch (err) {
    state.initialized = true;
    await addLog('ERROR', `加载主机列表失败: ${err}`);
    await addLog('INFO', t('logmsg.waiting'));
  }
}

// 初始化：加载主机列表
loadHosts();

/** 单例 hook */
export function useSshTunnel() {
  return {
    state,
    currentHost,
    currentRules,
    selectHost,
    addHost,
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