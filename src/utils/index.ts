// 通用工具函数

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