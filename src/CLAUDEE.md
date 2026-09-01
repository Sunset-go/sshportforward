# 项目维护指南（SSH 端口映射可视化工具 · 前端）

本文档描述 `src/` 前端应用的**已实现现状**与**维护约定**，供后续开发者/协作者参考。初始的需求说明已落地为下述实现，请以代码为准；本文档随代码演进同步更新。

## 角色

维护者是资深桌面端前端工程师，掌握 Vue 3 + TypeScript + Vite + Tauri 2。产出要求：完整、可直接运行、不写"其余代码省略"。

## 技术栈（已落地）

- Vue 3 + `<script setup lang="ts">` + TypeScript 严格模式 + Vite
- 状态管理：未安装 Pinia，改用模块级 `reactive` 单例（composable）
- i18n：未安装 vue-i18n，改用 `usePrefs.ts` 中的手写字典 + `t()` 翻译函数
- 样式：纯手写 `scoped` CSS + CSS 变量，无 UI 组件库，控件全部手写
- 图标：内联 SVG，无图标库依赖
- 未修改 `src-tauri/`，未引入额外依赖

## 文件结构（现状）

```
src/
├── types/index.ts               # 类型定义
├── composables/
│   ├── useSshTunnel.ts          # 隧道/主机/规则/日志状态单例（即早期规划的 stores/sshStore.ts）
│   └── usePrefs.ts              # 语言与主题偏好单例（i18n 字典 + localStorage 持久化）
├── mock/mockData.ts             # 假数据 + 假日志生成器
├── components/
│   ├── HostBar.vue              # 顶部：主机选择 + 状态 + 操作按钮 + 语言/主题切换
│   ├── ConnectionForm.vue       # SSH 连接信息表单
│   ├── PortForwardPanel.vue     # 端口转发规则区（Tab + 规则列表）
│   ├── RuleTable.vue            # 规则表格
│   ├── RuleEditModal.vue        # 规则编辑弹窗
│   ├── LogConsole.vue           # 底部日志输出区
│   ├── StatusDot.vue            # 状态指示灯
│   ├── LangSwitcher.vue         # 语言切换下拉
│   └── ThemeSwitcher.vue        # 主题切换下拉
└── App.vue                      # 入口与布局，定义全局 CSS 变量
```

## 界面分区（自上而下单列纵向布局，占满窗口高度）

1. **HostBar**：标题「已保存主机」+ 主机选择器（下拉）；中部状态灯 + 文案（未连接/连接中/已连接/连接失败，灰/琥珀/绿/红）；右侧按钮组「保存」「删除」，以及语言与主题两个下拉。
2. **ConnectionForm**：卡片式容器；两列表单（主机/端口/用户名/密码/私钥路径）；私钥路径带「浏览」按钮（Tauri dialog 未接入时 mock 回填）；卡片右侧纵向「启动」（主色）「关闭」（未连接时禁用）。
3. **PortForwardPanel**：顶部 Tab 本地 -L / 远端 -R / 动态 -D；规则表格列：启用 | 监听地址 | 目标地址（远端 Tab 下为「远端地址 → 本地地址」）| 备注 | 操作；底部「+ 添加规则」；超出高度内部滚动、表头固定；地址支持行内编辑失焦写入；删除无需二次确认但写日志。
4. **LogConsole**：终端风格等宽字体，逐行追加自动滚底（手动上滚暂停）；格式 `[HH:mm:ss] 级别 消息`；级别配色 INFO 蓝灰 / SUCCESS 绿 / WARN 琥珀 / ERROR 红；右上「清空」「复制全部」；高度占满剩余空间、自适应。

## 多语言与主题（新增能力）

- 词典：`usePrefs.ts` 的 `messages`，键名点分；`t(key, params?)` 支持 `{name}` 插值。
- 语言：`zh` / `en`，由 `LangSwitcher.vue` 切换，持久化到 `localStorage` 键 `sshpf:locale`。
- 主题：`dark` / `light`，由 `ThemeSwitcher.vue` 切换，写入 `<html data-theme>`，持久化到 `sshpf:theme`。
- 新增文案/语言/主题的步骤见 `README.md` 的「扩展指南」。

## 视觉规范（CSS 变量，App.vue 全局样式）

深色默认（`:root`）：

```
--bg-app:#0d1117; --bg-panel:#161b22; --bg-input:#0b0f14;
--border:#26303d; --text:#c9d1d9; --text-dim:#7d8590;
--accent:#2f81f7; --accent-soft:rgba(47,129,247,.15);
--success:#3fb950; --warn:#d29922; --error:#f85149;
--radius:6px; --gap:12px;
```

浅色覆盖（`[data-theme='light']`）：在 App.vue 中逐项覆盖为明亮配色。

字体：系统无衬线（中文 "Microsoft YaHei"/"PingFang SC"），日志区等宽（ui-monospace/Consolas）。细节：面板 1px 边框 + 极淡外发光；输入框聚焦 accent 描边 + 3px 柔和 focus ring；按钮 hover 提亮、active 下沉；自定义细滚动条；禁用态降低透明度并禁用 hover。

## 状态与数据模型

```ts
type ConnState = 'idle' | 'connecting' | 'connected' | 'error' | 'disconnecting';
type RuleType = 'local' | 'remote' | 'dynamic';
interface ForwardRule { id: string; type: RuleType; enabled: boolean; listenAddr: string; targetAddr: string; note: string; }
interface HostProfile { id: string; name: string; host: string; port: number; username: string; password: string; keyPath: string; rules: ForwardRule[]; }
interface LogEntry { id: number; time: string; level: LogLevel; message: string; }
```

状态机：未连接 →（点启动、校验必填项）→ 连接中 → 已连接/连接失败；已连接时自动启用所有勾选规则并写日志；关闭连接回到未连接并写日志。表单校验失败在对应字段下方显示红色提示。

## Tauri 接入点（先 mock，保留接口形状）

在 `useSshTunnel.ts` 中统一封装 `startTunnel` / `stopTunnel` / `pickKeyFile` / `readLogs` 四个方法：内部用 `try { await invoke(...) } catch { 回落到 mockData }`，保证无后端也能跑通全部交互；替换位置以中文注释 `// TODO: 对接 src-tauri 命令 XXX` 标出。

| 方法 | 后端命令 |
| --- | --- |
| startTunnel | start_tunnel |
| stopTunnel | stop_tunnel |
| pickKeyFile | pick_key_file |
| readLogs | read_logs |

## 维护自检清单

1. `npm run dev` / `npm run tauri dev` 可直接启动，无 TS 报错、无控制台报错
2. `npm run build`（vue-tsc --noEmit + vite build）通过
3. 四个分区顺序一致，窗口缩放时不塌陷、无横向滚动条
4. Tab 切换只显示对应类型规则；勾选、新增、删除规则即时生效并写日志
5. 启动/关闭走通完整状态机，按钮禁用逻辑正确
6. 日志自动追加、自动滚底、清空与复制可用
7. 语言/主题切换即时生效，刷新后偏好保持
8. 所有文件给出完整代码，禁止 `// ...省略` 之类占位
