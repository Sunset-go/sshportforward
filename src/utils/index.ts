// 通用工具函数

import type { HostDraft, HostProfile } from '../types';

/** 为一个规则生成简短描述（用于日志）
 * @param prefix 前缀文本，如 "已启用"、"已禁用"、"草稿"
 * @param type 规则类型：'local' | 'remote' | 'dynamic'
 * @param listen 监听地址
 * @param target 目标地址
 */
export function describeRule(
  prefix: string,
  type: string,
  listen: string,
  target: string,
): string {
  const flag = type === 'local' ? '-L' : type === 'remote' ? '-R' : '-D';
  return `${prefix} [${flag}] ${listen}${target ? ' → ' + target : ''}`;
}

/** 生成随机 ID */
export function genId(prefix: string): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 9)}`;
}

/** 字节数格式化为可读文本（B/KB/MB/GB/TB），统一保留 1 位小数，
 *  值每跨过 1024 阈值单位自动进位 */
export function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return '0.0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let v = n;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  return `${v.toFixed(1)} ${units[i]}`;
}

/** 速率（字节/秒）格式化为可读文本，如 "12.3 KB/s" */
export function formatSpeed(bytesPerSec: number): string {
  return `${formatBytes(bytesPerSec)}/s`;
}

/** 主机表单是否通过必填与端口校验（校验规则只在此写一份） */
export function isHostDraftValid(d: HostDraft): boolean {
  return (
    !!d.name.trim() &&
    !!d.host.trim() &&
    !!d.username.trim() &&
    Number.isFinite(d.port) &&
    d.port > 0 &&
    d.port <= 65535
  );
}

/** 将认证方式映射为扁平凭据：密码模式清空私钥字段，私钥模式清空密码 */
export function hostDraftToAuthInput(d: HostDraft): {
  password: string;
  keyPath: string;
  passphrase: string;
} {
  const isKey = d.method === 'privateKey';
  return {
    password: isKey ? '' : d.password,
    keyPath: isKey ? d.keyPath : '',
    passphrase: isKey ? d.passphrase : '',
  };
}

/** 主机档案转为表单草稿（method 由 keyPath 是否为空推导） */
export function hostToDraft(h: HostProfile): HostDraft {
  return {
    id: h.id,
    name: h.name,
    host: h.host,
    port: h.port,
    username: h.username,
    method: h.keyPath ? 'privateKey' : 'password',
    password: h.password,
    keyPath: h.keyPath,
    passphrase: h.passphrase,
  };
}