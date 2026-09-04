// SSH 隧道状态管理（单例 composable）
// 项目未安装 Pinia，故用模块级 reactive 单例实现全局状态。

import { computed, reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { t } from './usePrefs';
import { describeRule, genId } from '../utils';
import type { ConnState, ForwardRule, HostProfile, LogEntry, LogLevel, RuleType, TrafficStat } from '../types';

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
  /** 尚未持久化到后端的主机 id 集合（新建主机的草稿） */
  dirtyHosts: Set<string>;
  /** 各转发规则的实时流量速率（字节/秒，规则 id 为键，仅连接期间有值） */
  traffic: Record<string, TrafficStat>;
  /** 主机名称输入框内容（HostBar 编辑，点击保存时写入当前主机） */
  hostName: string;
  /** 初始化是否完成 */
  initialized: boolean;
  /** 三张卡片的折叠状态（true=收起）：新建主机时展开配置并收起其余两张，
   *  启动连接时收起配置与日志、展开转发规则，便于聚焦当前操作 */
  cardCollapsed: { config: boolean; rules: boolean; logs: boolean };
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
  dirtyHosts: new Set(),
  traffic: {},
  hostName: '',
  initialized: false,
  cardCollapsed: { config: false, rules: false, logs: false },
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

/** 内存日志最大条数（超出后裁剪最旧条目，防止无界增长） */
const MAX_LOGS = 500;

/** 追加一条日志（双写：本地内存 + 后端 DB） */
async function addLog(level: LogLevel, message: string): Promise<void> {
  const entry: LogEntry = { id: nextLogId(), time: nowTime(), level, message };
  state.logs.push(entry);
  if (state.logs.length > MAX_LOGS) {
    state.logs.splice(0, state.logs.length - MAX_LOGS);
  }
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

/** 当前主机是否为尚未保存的草稿 */
const currentHostIsDraft = computed(
  () => !!currentHost.value && state.dirtyHosts.has(currentHost.value.id),
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

/** 切换当前主机（离开时丢弃未保存的草稿主机） */
function selectHost(id: string): void {
  if (id === state.currentHostId) return;

  const prev = currentHost.value;
  if (prev && state.dirtyHosts.has(prev.id)) {
    const idx = state.hosts.findIndex((x) => x.id === prev.id);
    if (idx >= 0) state.hosts.splice(idx, 1);
    state.dirtyHosts.delete(prev.id);
    addLog('WARN', t('logmsg.discardHost', { name: prev.name }));
  }

  state.currentHostId = id;
  state.errors = {};
  const h = currentHost.value;
  addLog('INFO', t('logmsg.switchHost', { name: h?.name ?? t('common.unknown'), host: h?.host ?? '' }));
}

/** 新增一台主机（仅前端草稿，保存时才持久化到后端） */
function addHost(): void {
  const draft = createDefaultHost();
  state.hosts.push(draft);
  state.currentHostId = draft.id;
  state.dirtyHosts.add(draft.id);
  // 新建主机时聚焦配置：展开配置卡片，收起转发规则与日志
  state.cardCollapsed = { config: false, rules: true, logs: true };
  addLog('INFO', t('logmsg.addHost', { name: draft.name }));
}

/** 保存当前主机配置（草稿主机在此持久化并换用后端正式 id） */
async function saveHost(): Promise<void> {
  const h = currentHost.value;
  if (!h) return;
  // 名称输入框内容写入当前主机
  h.name = state.hostName.trim() || t('common.unnamed');
  const isNew = state.dirtyHosts.has(h.id);

  // 草稿主机：持久化后替换为后端返回的正式记录
  if (isNew) {
    try {
      const saved = await invoke<HostProfile>('save_host', {
        input: {
          id: undefined,
          name: h.name,
          host: h.host,
          port: h.port,
          username: h.username,
          password: h.password,
          keyPath: h.keyPath,
        },
      });
      const idx = state.hosts.findIndex((x) => x.id === h.id);
      if (idx >= 0) state.hosts.splice(idx, 1, saved);
      state.currentHostId = saved.id;
      state.dirtyHosts.delete(h.id);
      addLog('SUCCESS', t('logmsg.saveHost', { name: saved.name }));
    } catch (err) {
      addLog('ERROR', `${t('logmsg.err.saveHost')}: ${err}`);
    }
    return;
  }

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
      addLog('ERROR', `${t('logmsg.err.saveRule')}: ${err}`);
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
    addLog('ERROR', `${t('logmsg.err.saveHost')}: ${err}`);
  }
}

/** 删除当前主机（草稿主机直接移除，已持久化的先删后端） */
async function deleteHost(): Promise<void> {
  const h = currentHost.value;
  if (!h) return;

  if (!state.dirtyHosts.has(h.id)) {
    try {
      await invoke('delete_host', { hostId: h.id });
    } catch (err) {
      addLog('ERROR', `${t('logmsg.err.deleteHost')}: ${err}`);
      return;
    }
  }

  const idx = state.hosts.findIndex((x) => x.id === h.id);
  state.hosts.splice(idx, 1);
  state.currentHostId = state.hosts[0]?.id ?? '';
  state.dirtyHosts.delete(h.id);
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
  if (state.dirtyHosts.has(h.id)) {
    addLog('WARN', t('logmsg.saveHostFirst'));
    return;
  }

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
    addLog('ERROR', `${t('logmsg.err.addRule')}: ${err}`);
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
    addLog('ERROR', `${t('logmsg.err.removeRule')}: ${err}`);
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
    addLog('ERROR', `${t('logmsg.err.updateRule')}: ${err}`);
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
    addLog('ERROR', `${t('logmsg.err.saveRule')}: ${err}`);
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
  if (state.dirtyHosts.has(h.id)) {
    addLog('WARN', t('logmsg.saveHostFirst'));
    return;
  }
  // 启动连接时聚焦转发规则：收起配置与日志，展开转发规则
  state.cardCollapsed = { config: true, rules: false, logs: true };
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
      addLog('ERROR', `${t('logmsg.err.saveRule')}: ${err}`);
    }
  }
  state.dirtyRules.clear();

  try {
    // 凭据由后端从加密库读取，前端只传主机 id
    await invoke('start_tunnel', { hostId: h.id });
    const connState = await invoke<string>('get_conn_state');
    state.connState = connState as ConnState;
  } catch (err) {
    state.connState = 'error';
    addLog('ERROR', `${t('logmsg.err.connect')}: ${err}`);
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
    await invoke('stop_tunnel');
    const connState = await invoke<string>('get_conn_state');
    state.connState = connState as ConnState;
  } catch (err) {
    state.connState = 'error';
    addLog('ERROR', `${t('logmsg.err.disconnect')}: ${err}`);
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
    addLog('ERROR', `${t('logmsg.err.pickKey')}: ${err}`);
  }
}

/** 清空日志（同步清内存与后端 DB） */
async function clearLogs(): Promise<void> {
  state.logs.splice(0, state.logs.length);
  try {
    await invoke('clear_logs');
  } catch (err) {
    console.error('[clear_logs] 清空后端日志失败:', err);
  }
}

/** 拼接全部日志文本（用于复制） */
function logsText(): string {
  return state.logs.map((l) => `[${l.time}] ${l.level} ${l.message}`).join('\n');
}

/** 从后端加载主机列表（初始化用）。若数据库为空则创建一台默认草稿主机。 */
async function loadHosts(): Promise<void> {
  try {
    const hosts = await invoke<HostProfile[]>('list_hosts');
    if (hosts.length === 0) {
      const draft = createDefaultHost();
      state.hosts = [draft];
      state.currentHostId = draft.id;
      state.dirtyHosts.add(draft.id);
      await addLog('INFO', t('logmsg.createdDefaultHost', { name: draft.name }));
    } else {
      state.hosts = hosts;
      state.currentHostId = hosts[0]?.id ?? '';
    }
    state.initialized = true;
    await addLog('INFO', t('logmsg.loaded', { n: state.hosts.length }));
    await addLog('INFO', t('logmsg.waiting'));
  } catch (err) {
    state.initialized = true;
    await addLog('ERROR', `${t('logmsg.err.loadHosts')}: ${err}`);
    await addLog('INFO', t('logmsg.waiting'));
  }
}

// 切换主机时同步名称输入框
watch(
  () => state.currentHostId,
  () => {
    state.hostName = currentHost.value?.name ?? '';
  },
  { immediate: true },
);

// 初始化：加载主机列表
loadHosts();

// 连接期间每秒采样各规则的累计流量并换算为实时速率；断开后停止轮询并清零。
// 后端 get_traffic 返回的是累计字节数，速率 = 相邻两次采样差值 / 实际间隔。
let trafficTimer: ReturnType<typeof setInterval> | null = null;
let lastSamples: Record<string, TrafficStat> = {};
let lastSampleAt = 0;

watch(
  () => state.connState,
  (v) => {
    if (v === 'connected' && trafficTimer === null) {
      lastSamples = {};
      lastSampleAt = 0;
      trafficTimer = setInterval(async () => {
        try {
          const cur = await invoke<Record<string, TrafficStat>>('get_traffic');
          const now = Date.now();
          const dt = (now - lastSampleAt) / 1000;
          const rates: Record<string, TrafficStat> = {};
          for (const [id, s] of Object.entries(cur)) {
            const prev = lastSamples[id];
            // 无上次采样或计数器重置（重连）时速率记 0
            if (prev && dt > 0 && s.up >= prev.up && s.down >= prev.down) {
              rates[id] = { up: (s.up - prev.up) / dt, down: (s.down - prev.down) / dt };
            } else {
              rates[id] = { up: 0, down: 0 };
            }
          }
          lastSamples = cur;
          lastSampleAt = now;
          state.traffic = rates;
        } catch {
          /* 查询失败时保留上次数据 */
        }
      }, 1000);
    } else if (v !== 'connected' && trafficTimer !== null) {
      clearInterval(trafficTimer);
      trafficTimer = null;
      lastSamples = {};
      lastSampleAt = 0;
      state.traffic = {};
    }
  },
);

/** 单例 hook */
export function useSshTunnel() {
  return {
    state,
    currentHost,
    currentHostIsDraft,
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
    clearLogs,
    logsText,
  };
}