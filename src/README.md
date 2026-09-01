# SSH 端口转发可视化工具（前端）

`src/` 目录下的 Vue 3 前端应用：基于 Tauri 2 + Vite 构建的 SSH 端口映射可视化工具，提供主机管理、连接信息表单、端口转发规则编辑与运行时日志查看，支持中英双语与深浅主题。

> 本文件仅描述 `src/` **前端**部分的使用与架构，不涉及 `src-tauri/` 后端与打包发布。

## 技术栈

- Vue 3（`<script setup lang="ts">`）+ TypeScript 严格模式 + Vite
- Tauri 2（`@tauri-apps/api`，仅用于 `invoke` 调用后端命令，失败时回落到 mock）
- 无 UI 组件库、无图标库：控件与图标全部手写（`scoped` CSS + 内联 SVG）
- 无 Pinia / vue-i18n：用模块级 `reactive` 单例 + 手写字典实现状态与 i18n

## 目录结构

```
src/
├── App.vue                  # 应用入口与布局，定义全局 CSS 变量
├── main.ts                  # Vue 挂载入口
├── types/
│   └── index.ts             # 全局类型定义
├── composables/
│   ├── useSshTunnel.ts      # 隧道/主机/规则/日志状态单例
│   └── usePrefs.ts          # 语言与主题偏好单例（i18n 字典 + 持久化）
├── mock/
│   └── mockData.ts          # 假数据与假日志生成器（后端不可用时的回落）
└── components/
    ├── HostBar.vue          # 顶部：主机选择 + 状态灯 + 操作按钮 + 语言/主题切换
    ├── ConnectionForm.vue   # SSH 连接信息表单（启动/关闭）
    ├── PortForwardPanel.vue # 端口转发规则区（Tab + 规则列表）
    ├── RuleTable.vue        # 规则表格
    ├── RuleEditModal.vue    # 规则编辑弹窗
    ├── LogConsole.vue       # 底部日志输出区
    ├── StatusDot.vue        # 状态指示灯
    ├── LangSwitcher.vue     # 语言切换下拉
    └── ThemeSwitcher.vue    # 主题切换下拉
```

## 运行

```bash
npm install          # 安装依赖
npm run dev          # 启动 Vite 开发服务器（纯前端，可脱离后端运行）
npm run build        # 类型检查（vue-tsc --noEmit）+ 生产构建
npm run tauri dev    # 带 Tauri 壳启动（需要 src-tauri/ 与 Rust 环境）
```

## 架构说明

### 状态管理

前端未引入 Pinia，改用两个「模块级 `reactive` 单例」的 composable 共享全局状态：

- **`useSshTunnel()`**：管理主机列表、当前主机、连接状态、规则与日志。
  - 状态字段：`hosts` / `currentHostId` / `connState` / `activeRuleType` / `showPassword` / `logs` / `errors` / `editingRuleId`。
  - 导出方法：`selectHost` / `saveHost` / `deleteHost` / `setActiveRuleType` / `addRule` / `removeRule` / `toggleRule` / `commitRuleEdit` / `openEditModal` / `closeEditModal` / `saveRuleEdit` / `toggleShowPassword` / `startTunnel` / `stopTunnel` / `pickKeyFile` / `readLogs` / `clearLogs` / `logsText`。
- **`usePrefs()`**：管理 `locale`（`zh`/`en`）与 `theme`（`dark`/`light`），并导出翻译函数 `t`。

由于都是模块级单例，任何组件调用同一 hook 都共享同一份状态，无需 Prop 层层透传。

### 组件关系

```
App.vue
├── HostBar.vue
│   ├── StatusDot.vue
│   ├── LangSwitcher.vue
│   └── ThemeSwitcher.vue
├── ConnectionForm.vue
├── PortForwardPanel.vue
│   ├── RuleTable.vue
│   └── RuleEditModal.vue
└── LogConsole.vue
```

### 数据模型

见 `src/types/index.ts`：

- `ConnState`：`'idle' | 'connecting' | 'connected' | 'error' | 'disconnecting'`
- `RuleType`：`'local' | 'remote' | 'dynamic'`（对应 `-L` / `-R` / `-D`）
- `LogLevel`：`'INFO' | 'SUCCESS' | 'WARN' | 'ERROR'`
- `ForwardRule`：`id / type / enabled / listenAddr / targetAddr / note`
- `HostProfile`：`id / name / host / port / username / password / keyPath / rules`
- `LogEntry`：`id / time / level / message`

### 多语言（i18n）

- 词典位于 `usePrefs.ts` 的 `messages`，按 `locale` 分 `zh` / `en` 两组，键名点分（如 `common.save`、`logmsg.connected`）。
- 翻译函数 `t(key, params?)` 支持 `{name}` 插值：`t('logmsg.connected', { host, port })`。
- 组件内既可 `const { t } = usePrefs()` 解构，也可直接 `import { t } from '../composables/usePrefs'`。
- 偏好持久化到 `localStorage`：`sshpf:locale`、`sshpf:theme`。

### 主题

- 主题写入 `<html data-theme="...">`，`App.vue` 的全局样式通过 `:root`（深色默认）与 `[data-theme='light']`（浅色覆盖）两套 CSS 变量切换。
- 所有组件样式只引用变量，不写死颜色。

### Tauri 接入与 mock 回落

后端调用统一封装在 `useSshTunnel.ts`，形如 `try { await invoke(...) } catch { 回落到 mockData }`：

| 方法 | 后端命令 | 回落行为 |
| --- | --- | --- |
| `startTunnel` | `start_tunnel` | `mockConnectSequence` 模拟握手/认证 |
| `stopTunnel` | `stop_tunnel` | `mockDisconnectSequence` 模拟断开 |
| `pickKeyFile` | `pick_key_file` | `pickMockKeyPath` 回填假路径 |
| `readLogs` | `read_logs` | 无额外日志 |

替换点均以 `// TODO: 对接 src-tauri 命令 XXX` 中文注释标出。

### 样式体系

- 全局变量与基础样式在 `App.vue` 的非 scoped `<style>` 中定义；组件样式为 `scoped`。
- 主要变量：`--bg-app` / `--bg-panel` / `--bg-input` / `--border` / `--text` / `--text-dim` / `--accent` / `--success` / `--warn` / `--error` / `--radius` / `--gap` 等。

## 扩展指南

- **新增文案**：在 `usePrefs.ts` 的 `zh` 与 `en` 两组字典中各加一条同名 key，再用 `t('key')` 取用。
- **新增语言**：扩展 `Locale` 类型、`messages` 字典、`readLocale` 兜底逻辑，并在 `LangSwitcher.vue` 中增加选项。
- **新增主题**：扩展 `ThemeMode`，在 `App.vue` 增加对应 `[data-theme='...']` 变量集。
- **对接真实后端**：实现 `src-tauri/` 中对应的 `start_tunnel` / `stop_tunnel` / `pick_key_file` / `read_logs` 命令即可；命令不可用时前端自动回落 mock，不影响开发。
