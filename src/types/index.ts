// 全局类型定义

/** 连接状态机 */
export type ConnState = 'idle' | 'connecting' | 'connected' | 'error' | 'disconnecting';

/** 转发规则类型：本地 -L / 远端 -R / 动态 -D */
export type RuleType = 'local' | 'remote' | 'dynamic';

/** 日志级别 */
export type LogLevel = 'INFO' | 'SUCCESS' | 'WARN' | 'ERROR';

/** 单条转发规则 */
export interface ForwardRule {
  id: string;
  type: RuleType;
  enabled: boolean;
  /** 监听地址，如 127.0.0.1:3080 */
  listenAddr: string;
  /** 目标地址，如 127.0.0.1:3080 或 192.168.1.100:80 */
  targetAddr: string;
  /** 备注 */
  note: string;
}

/** 主机配置档案 */
export interface HostProfile {
  id: string;
  /** 显示名称，如 PC */
  name: string;
  host: string;
  port: number;
  username: string;
  password: string;
  /** 私钥文件路径 */
  keyPath: string;
  rules: ForwardRule[];
}

/** 单条规则的流量统计（字节） */
export interface TrafficStat {
  up: number;
  down: number;
}

/** 单条日志 */
export interface LogEntry {
  id: number;
  /** HH:mm:ss */
  time: string;
  level: LogLevel;
  message: string;
}
