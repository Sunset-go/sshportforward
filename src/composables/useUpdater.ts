// 自动更新单例：启动时若用户已开启 autoUpdate，则检查新版本并自动下载安装。
// 失败时静默记录到 console，不阻断应用启动。
// 依赖：@tauri-apps/plugin-updater（check / downloadAndInstall）
//       @tauri-apps/plugin-process（relaunch）

import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { settings } from './useSettings';

let checking = false;

/**
 * 若 settings.autoUpdate 为 true，检查是否有新版本。
 * 有新版本时自动下载安装并重启应用。
 * 全程 try-catch，任何错误均静默处理。
 */
export async function checkForUpdatesOnStartup(): Promise<void> {
  if (checking) return;
  if (!settings.autoUpdate) return;
  checking = true;
  try {
    const update = await check();
    if (update?.available) {
      console.info(`发现新版本 ${update.version}，开始下载安装...`);
      await update.downloadAndInstall();
      // Windows 上安装完成后会自动重启；macOS/Linux 需手动 relaunch
      await relaunch();
    }
  } catch (e) {
    // 更新检查失败不影响正常使用，仅记录日志
    console.warn('自动更新检查失败:', e);
  } finally {
    checking = false;
  }
}

/** 单例 hook */
export function useUpdater() {
  return { checkForUpdatesOnStartup };
}
