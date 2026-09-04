<script setup lang="ts">
import { onMounted, ref } from 'vue';
import TopBar from './components/TopBar.vue';
import HostBar from './components/HostBar.vue';
import ConnectionForm from './components/ConnectionForm.vue';
import PortForwardPanel from './components/PortForwardPanel.vue';
import LogConsole from './components/LogConsole.vue';
import CloseBehaviorDialog from './components/CloseBehaviorDialog.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import { useSettings } from './composables/useSettings';

const { load } = useSettings();
const settingsOpen = ref(false);

// 首屏防闪由 index.html 内联脚本 + localStorage 镜像处理；
// 此处从后端加载完整偏好并应用（theme/locale/minimizeToTray/autoStart）
onMounted(() => { void load(); });
</script>

<template>
  <div class="shell">
    <TopBar @open-settings="settingsOpen = true" />
    <div class="app">
      <HostBar />
      <ConnectionForm />
      <PortForwardPanel class="grow" />
      <LogConsole class="grow log-grow" />
    </div>
    <CloseBehaviorDialog />
    <SettingsPanel v-model:open="settingsOpen" />
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}
.app {
  display: flex;
  flex-direction: column;
  gap: var(--gap);
  flex: 1 1 0;
  min-height: 0;
  padding: var(--gap);
  box-sizing: border-box;
  overflow: hidden;
}
.grow { flex: 1 1 0; min-height: 0; }
.log-grow { flex: 1.1 1 0; }
</style>
