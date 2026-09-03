<script setup lang="ts">
import HostBar from './components/HostBar.vue';
import ConnectionForm from './components/ConnectionForm.vue';
import PortForwardPanel from './components/PortForwardPanel.vue';
import LogConsole from './components/LogConsole.vue';
import TraySync from './components/TraySync.vue';
import { usePrefs } from './composables/usePrefs';

// 初始化语言与主题偏好（模块加载即应用 data-theme，避免闪烁）
usePrefs();
</script>

<template>
  <div class="app">
    <HostBar />
    <ConnectionForm />
    <PortForwardPanel class="grow" />
    <LogConsole class="grow log-grow" />
    <TraySync />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  gap: var(--gap);
  height: 100vh;
  padding: var(--gap);
  box-sizing: border-box;
  overflow: hidden;
}
.grow { flex: 1 1 0; min-height: 0; }
.log-grow { flex: 1.1 1 0; }
</style>

<style>
:root {
  --bg-app: #0d1117;
  --bg-panel: #161b22;
  --bg-input: #0b0f14;
  --border: #26303d;
  --text: #c9d1d9;
  --text-dim: #7d8590;
  --accent: #2f81f7;
  --accent-soft: rgba(47, 129, 247, 0.15);
  --success: #3fb950;
  --warn: #d29922;
  --error: #f85149;
  --radius: 6px;
  --gap: 12px;

  --border-soft: rgba(38, 48, 61, 0.5);
  --accent-hover: rgba(47, 129, 247, 0.06);
  --panel-shadow: 0 0 12px rgba(0, 0, 0, 0.25);
  --modal-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  --log-info: #6e7b8b;
  --log-success-text: #b6f0c0;
  --log-warn-text: #f0d39a;
  --log-error-text: #ffb3b0;

  font-family: "Microsoft YaHei", "PingFang SC", "Segoe UI", Roboto, system-ui, sans-serif;
  font-size: 14px;
  color: var(--text);
  background-color: var(--bg-app);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 浅色主题变量（覆盖深色默认值） */
[data-theme='light'] {
  --bg-app: #f6f8fa;
  --bg-panel: #ffffff;
  --bg-input: #ffffff;
  --border: #d0d7de;
  --text: #1f2328;
  --text-dim: #59636e;
  --accent: #0969da;
  --accent-soft: rgba(9, 105, 218, 0.12);
  --success: #1a7f37;
  --warn: #9a6700;
  --error: #cf222e;

  --border-soft: rgba(208, 215, 222, 0.65);
  --accent-hover: rgba(9, 105, 218, 0.08);
  --panel-shadow: 0 1px 3px rgba(31, 35, 40, 0.12), 0 1px 2px rgba(31, 35, 40, 0.06);
  --modal-shadow: 0 8px 32px rgba(31, 35, 40, 0.2);
  --log-info: #57606a;
  --log-success-text: #1a7f37;
  --log-warn-text: #7d4e00;
  --log-error-text: #cf222e;
}

* { box-sizing: border-box; }
html, body, #app { height: 100%; margin: 0; padding: 0; }
body { background: var(--bg-app); overflow: hidden; }

input, button, select { font-family: inherit; }
input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button { opacity: 0.4; }

/* 全局细滚动条 */
*::-webkit-scrollbar { width: 8px; height: 8px; }
*::-webkit-scrollbar-track { background: transparent; }
*::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 8px;
}
*::-webkit-scrollbar-thumb:hover { background: var(--text-dim); }
* { scrollbar-width: thin; scrollbar-color: var(--border) transparent; }
</style>
