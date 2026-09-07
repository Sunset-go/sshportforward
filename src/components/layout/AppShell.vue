<script setup lang="ts">
// 两栏骨架：侧边栏（212/56）+ 主区（页签栏 44 + 内容区）。
// 内容区用 KeepAlive 保活，切页签不丢表单填写状态。
import { markRaw, onMounted, ref, type Component } from 'vue';
import SideBar from './SideBar.vue';
import TabBar from './TabBar.vue';
import AddHostDialog from '../dialog/AddHostDialog.vue';
import ConnectPanel from '../panels/ConnectPanel.vue';
import PortForwardPanel from '../PortForwardPanel.vue';
import LogConsole from '../LogConsole.vue';
import SettingsPanel from '../panels/SettingsPanel.vue';
import CloseBehaviorDialog from '../CloseBehaviorDialog.vue';
import { useSettings } from '../../composables/useSettings';
import type { PanelKey } from '../../types';

const { load } = useSettings();

const panels: Record<PanelKey, Component> = {
  connect: markRaw(ConnectPanel),
  forward: markRaw(PortForwardPanel),
  log: markRaw(LogConsole),
  settings: markRaw(SettingsPanel),
};

const activeTab = ref<PanelKey>('connect');
const collapsed = ref(false);
const showAddHost = ref(false);

const LS_COLLAPSED = 'sshpf:sidebarCollapsed';

onMounted(() => {
  collapsed.value = localStorage.getItem(LS_COLLAPSED) === '1';
  void load();
});

function toggleCollapsed(): void {
  collapsed.value = !collapsed.value;
  localStorage.setItem(LS_COLLAPSED, collapsed.value ? '1' : '0');
}
</script>

<template>
  <div class="shell">
    <SideBar
      :collapsed="collapsed"
      @toggle="toggleCollapsed"
      @add-host="showAddHost = true"
      @open-settings="activeTab = 'settings'"
    />

    <main class="main">
      <TabBar :active="activeTab" @change="activeTab = $event" />
      <div class="content">
        <KeepAlive>
          <component :is="panels[activeTab]" :key="activeTab" class="panel-fill" />
        </KeepAlive>
      </div>
    </main>

    <CloseBehaviorDialog />
    <AddHostDialog v-model:open="showAddHost" @saved="activeTab = 'connect'" />
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: var(--win-bg);
}
.main {
  flex: 1 1 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.content {
  flex: 1 1 0;
  min-height: 0;
  padding: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.panel-fill {
  flex: 1 1 0;
  min-height: 0;
}
</style>
