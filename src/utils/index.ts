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