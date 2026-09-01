# 角色

你是一名资深桌面端前端工程师，精通 Vue 3 + TypeScript + Vite + Tauri 2。你的产出要求是：完整、可直接运行、不写"其余代码省略"。

# 任务

在我现有的 Tauri 2 + Vue 3 项目的 `src/` 目录下，从零还原一张参考截图中的「SSH 端口映射可视化工具」界面。要求：布局结构、分区顺序、控件种类与截图一致，视觉风格为深色科技风，界面文案使用简体中文。

# 技术栈约束

- Vue 3 + `<script setup lang="ts">` + TypeScript 严格模式 + Vite
- 状态管理用 Pinia；若项目尚未安装，则改用 `reactive` 组合式函数（composable）实现，不要引入 Redux 等额外方案
- 样式：纯手写 `scoped` CSS + CSS 变量，**不要引入任何 UI 组件库**，所有控件（输入框、下拉、Tab、复选框、按钮、滚动条）手写实现，以便精确控制深色风格
- 图标：使用内联 SVG，不要引入图标库依赖
- 不修改 `src-tauri/` 目录，不安装新依赖（除非项目已存在）

# 文件结构（按此创建）

src/
types/index.ts # 类型定义
stores/sshStore.ts # 或 composables/useSshTunnel.ts
mock/mockData.ts # 假数据 + 假日志生成器
components/
HostBar.vue # 顶部：已保存主机 + 状态 + 操作按钮
ConnectionForm.vue # SSH 连接信息表单
PortForwardPanel.vue # 端口转发规则区（含 Tab 与规则列表）
RuleTable.vue # 规则表格
LogConsole.vue # 底部日志输出区
StatusDot.vue # 状态指示灯小组件
App.vue

# 布局规格（自上而下，单列纵向布局，占满窗口高度）

## 1. 顶部主机栏 HostBar

- 左侧标题：「已保存主机」，后接主机选择器（下拉/输入框）显示 `PC (100.122.12.38)`
- 中部状态区：状态圆点 + 文案（未连接 / 连接中 / 已连接 / 连接失败），颜色分别为 灰 / 琥珀 / 绿 / 红
- 右侧按钮组：「保存」「删除」

## 2. SSH 连接信息区 ConnectionForm（卡片式容器，带标题栏）

- 两列表单网格，字段依次为：
  - 主机 Host（文本输入，占位 `192.168.1.100`）
  - 端口 Port（数字输入，默认 22）
  - 用户名（文本输入，示例值 Administrator）
  - 密码（密码输入，可切换明文显示）
  - 私钥路径（文本输入 + 右侧「浏览」按钮，点击用 Tauri dialog 打开，未接入时用 mock 回填路径）
- 卡片右侧纵向按钮：「启动」（主色）、「关闭」（禁用态当未连接）
- 连接中：启动按钮显示 loading 并禁用；已连接：启动变禁用、关闭可点

## 3. 端口转发规则区 PortForwardPanel

- 顶部 Tab：本地 -L / 远端 -R / 动态 -D，选中项有下划线或高亮底
- 规则表格列：启用（复选框） | 监听地址 | 目标地址（远端 Tab 下为「远端地址 → 本地地址」） | 备注 | 操作（删除图标）
- 示例数据：`127.0.0.1:3080` 等若干条本地映射，含启用/未启用混合状态
- 表格底部：「+ 添加规则」按钮；表格内容超出高度时内部滚动，表头固定
- 行内地址支持直接编辑，失焦即写入状态；删除行无需二次确认但要在日志中记录

## 4. 底部日志区 LogConsole

- 终端风格：等宽字体、逐行追加、自动滚到底部（用户手动上滚时暂停自动滚动）
- 每行格式：`[HH:mm:ss] 级别 消息`，级别配色：INFO 蓝灰 / SUCCESS 绿 / WARN 琥珀 / ERROR 红
- 初始载入日志：「载入主机配置…」「等待连接…」
- 右上角：「清空」「复制全部」按钮
- 高度占满剩余空间，随窗口缩放自适应；窗口很窄时不横向破版

# 视觉规范（写入 CSS 变量统一复用）

--bg-app: #0d1117; --bg-panel: #161b22; --bg-input: #0b0f14;
--border: #26303d; --text: #c9d1d9; --text-dim: #7d8590;
--accent: #2f81f7; --accent-soft: rgba(47,129,247,.15);
--success: #3fb950; --warn: #d29922; --error: #f85149;
--radius: 6px; --gap: 12px;
字体：系统无衬线（中文用 "Microsoft YaHei"/"PingFang SC"），日志区用 ui-monospace/Consolas 等宽字体。
细节：面板 1px 边框 + 极淡外发光；输入框聚焦时 accent 色描边与 3px 柔和 focus ring；按钮 hover 提亮、active 轻微下沉；自定义细滚动条；禁用态降低透明度并禁用 hover。

# 状态与数据模型

type ConnState = 'idle' | 'connecting' | 'connected' | 'error' | 'disconnecting';
type RuleType = 'local' | 'remote' | 'dynamic';
interface ForwardRule { id: string; type: RuleType; enabled: boolean; listenAddr: string; targetAddr: string; note: string; }
interface HostProfile { id: string; name: string; host: string; port: number; username: string; password: string; keyPath: string; rules: ForwardRule[]; }
状态机：未连接→（点启动、校验必填项）→连接中→已连接/连接失败；已连接时自动启用所有勾选规则并写日志；关闭连接回到未连接并把状态写回日志。表单校验失败时在对应字段下方显示红色提示。

# Tauri 接入点（先 mock，保留接口形状）

在 store 中统一封装 `startTunnel` / `stopTunnel` / `pickKeyFile` / `readLogs` 四个方法：内部用 `try { await invoke(...) } catch { 回落到 mockData }` 的形式，保证无后端也能跑通全部交互；用 `// TODO: 对接 src-tauri 命令` 注释标出替换位置，注释用中文。

# 验收标准（完成后逐条自检）

1. `pnpm tauri dev`（或 `npm run tauri dev`）可直接启动，无 TS 报错、无控制台报错
2. 四个分区顺序与截图一致，窗口缩放时不塌陷、不出现横向滚动条
3. Tab 切换只显示对应类型的规则；勾选、新增、删除规则即时生效并写日志
4. 启动/关闭能走通完整状态机，按钮禁用逻辑正确
5. 日志自动追加、自动滚底、清空与复制可用
6. 所有文件给出完整代码，禁止出现 `// ...省略` 之类占位

# 输出

先给出文件树，再按文件逐个输出完整代码，最后附一段「如何对接真实 Tauri 后端」的中文说明。
