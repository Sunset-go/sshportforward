// 假数据 + 假日志生成器
// 当 Tauri 后端命令不可用时，由 composable 回落到这里，保证前端交互可独立跑通。

import type { HostProfile, LogLevel } from '../types';
import { t } from '../composables/usePrefs';

/** 已保存的主机列表（示例数据） */
export const mockHosts: HostProfile[] = [
  {
    id: 'host-pc',
    name: 'PC',
    host: '100.122.12.38',
    port: 22,
    username: 'Administrator',
    password: '',
    keyPath: '',
    rules: [
      { id: 'r-1', type: 'local', enabled: true, listenAddr: '127.0.0.1:3080', targetAddr: '127.0.0.1:3080', note: 'K8s API' },
      { id: 'r-2', type: 'local', enabled: true, listenAddr: '127.0.0.1:8080', targetAddr: '192.168.1.100:80', note: 'Web 控制台' },
      { id: 'r-3', type: 'local', enabled: false, listenAddr: '127.0.0.1:5432', targetAddr: '10.0.0.5:5432', note: 'PostgreSQL' },
      { id: 'r-4', type: 'local', enabled: true, listenAddr: '127.0.0.1:6379', targetAddr: '10.0.0.6:6379', note: 'Redis' },
      { id: 'r-5', type: 'remote', enabled: false, listenAddr: '0.0.0.0:8081', targetAddr: '127.0.0.1:8081', note: '远端反推' },
      { id: 'r-6', type: 'dynamic', enabled: false, listenAddr: '127.0.0.1:1080', targetAddr: '', note: 'SOCKS5 代理' },
    ],
  },
  {
    id: 'host-build',
    name: '构建服务器',
    host: '192.168.1.100',
    port: 22,
    username: 'ci',
    password: '',
    keyPath: '',
    rules: [
      { id: 'rb-1', type: 'local', enabled: false, listenAddr: '127.0.0.1:9000', targetAddr: '127.0.0.1:9000', note: 'Jenkins' },
    ],
  },
  {
    id: 'host-nas',
    name: 'NAS',
    host: '10.0.0.2',
    port: 22,
    username: 'admin',
    password: '',
    keyPath: '',
    rules: [
      { id: 'rn-1', type: 'local', enabled: false, listenAddr: '127.0.0.1:5000', targetAddr: '127.0.0.1:5000', note: '群晖面板' },
      { id: 'rn-2', type: 'dynamic', enabled: false, listenAddr: '127.0.0.1:1080', targetAddr: '', note: 'SOCKS5' },
    ],
  },
];

/** 候选的私钥路径，用于 mock 回填 */
const mockKeyPaths = [
  'C:\\Users\\Administrator\\.ssh\\id_ed25519',
  'C:\\Users\\Administrator\\.ssh\\id_rsa',
  '/home/admin/.ssh/id_ed25519',
];

/** 随机返回一个假私钥路径 */
export function pickMockKeyPath(): string {
  return mockKeyPaths[Math.floor(Math.random() * mockKeyPaths.length)] ?? mockKeyPaths[0];
}

const wait = (ms: number): Promise<void> => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * 模拟连接过程：逐步输出握手/认证日志
 * @param host  目标主机
 * @param emit  日志发射回调
 */
export async function mockConnectSequence(
  host: HostProfile,
  emit: (level: LogLevel, message: string) => void,
): Promise<void> {
  await wait(400);
  emit('INFO', t('logmsg.resolveHost', { host: host.host }));
  await wait(500);
  emit('INFO', t('logmsg.handshake'));
  await wait(600);
  emit('INFO', t('logmsg.auth', { user: host.username }));
  await wait(500);
}

/** 模拟断开过程 */
export async function mockDisconnectSequence(
  host: HostProfile,
  emit: (level: LogLevel, message: string) => void,
): Promise<void> {
  await wait(300);
  emit('INFO', t('logmsg.disconnectingMock', { host: host.host }));
  await wait(400);
}

/** 为一个规则生成简短描述（用于日志） */
export function describeRule(prefix: string, type: string, listen: string, target: string): string {
  const flag = type === 'local' ? '-L' : type === 'remote' ? '-R' : '-D';
  return `${prefix} [${flag}] ${listen}${target ? ' → ' + target : ''}`;
}
