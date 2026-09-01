# CLAUDE.md —— src-tauri（Tauri 后端）维护指南

本文件面向维护该 Tauri 后端（Rust / Tauri 2 / SQLite）的 AI 助手与开发者。
前端位于仓库根目录 `src/`。

## 职责边界

- 后端语义：通过 `tauri::command` 暴露结构化 API（CRUD / 隧道 / 日志 / 密钥选择），
  数据持久化于 SQLite，连接状态保存在进程内存。
- 不要实现前端逻辑：数据模型 CRUD 的 UI 状态、mock 回退、命令编排在前端
  `src/composables/useSshTunnel.ts` / `usePrefs.ts` 中完成。
- 当前隧道为 **mock**（仅模拟握手耗时与状态流转），未建立真实 SSH 通道。

## 技术栈

- Tauri 2（`tauri = "2"`）
- `sqlx = "0.8"`（`runtime-tokio-rustls` + `sqlite` + `macros`），直接持有 `SqlitePool`
- `tauri-plugin-dialog = "2"`：系统文件对话框（选择私钥）
- `tauri-plugin-opener = "2"`：打开外部链接
- `tokio`（仅 `time` feature，隧道 mock 用 `sleep`）、`chrono`（UNIX 秒 → `HH:mm:ss`）

## 目录结构

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── capabilities/default.json   # IPC 权限（自定义命令无需额外声明）
├── db/migrations/001_init.sql  # 建表 SQL（启动时由 db::init_db 执行）
└── src/
    ├── lib.rs          # Builder：注册插件、setup 初始化 DB 与状态、invoke_handler
    ├── main.rs         # 入口，调用 lib::run()
    ├── db.rs           # SqlitePool 初始化 / AppDb / AppConnState / gen_id / db_err
    ├── models.rs       # 行结构(FromRow) + DTO(Serialize/Deserialize) + 转换
    └── commands/
        ├── mod.rs
        ├── hosts.rs    # 主机 CRUD（list/get/save/delete_host）
        ├── rules.rs    # 规则 CRUD（list/add/update/delete_rule）
        ├── logs.rs     # 日志（add/list/clear_logs、read_logs）
        └── tunnel.rs   # 隧道（start/stop_tunnel）、连接状态机、pick_key_file
```

## 前端契约（必须保持同步）

前端类型定义在 `src/types/index.ts`，invoke 用法见 `src/composables/useSshTunnel.ts`。

| 类型 | 字段 / 约束 |
| --- | --- |
| `HostProfile.id` | **string**（如 `'host-pc'`），TEXT 主键 |
| `ForwardRule.id` | **string**（如 `'r-1'`），TEXT 主键 |
| `ForwardRule.type` | `'local' \| 'remote' \| 'dynamic'` |
| `ForwardRule.enabled` | boolean；DB 存 0/1，`RuleRow::into_out` 转 bool |
| `LogLevel` | **全大写**：`'INFO' \| 'SUCCESS' \| 'WARN' \| 'ERROR'` |
| `LogEntry.time` | `HH:mm:ss`（UNIX 秒经 chrono 格式化） |
| `read_logs` 返回值 | `string[]`：`"[HH:mm:ss] LEVEL message"` |

对外字段一律 **camelCase**（`#[serde(rename_all = "camelCase")]`）；
命令参数 Rust 侧为 snake_case，Tauri 自动映射（前端传 `hostId` → 参数 `host_id`）。

## 后端命令清单（lib.rs invoke_handler 注册）

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `list_hosts` | – | `HostProfileOut[]`（内嵌 rules） |
| `get_host` | `hostId` | `HostProfileOut \| null` |
| `save_host` | `SaveHostInput`（id 空则新建） | `HostProfileOut` |
| `delete_host` | `hostId` | `bool`（级联删规则） |
| `list_rules` | `hostId` | `ForwardRuleOut[]` |
| `add_rule` | `hostId`, `RuleInput` | `ForwardRuleOut` |
| `update_rule` | `hostId`, `ruleId`, `RuleInput` | `ForwardRuleOut` |
| `delete_rule` | `hostId`, `ruleId` | `bool` |
| `add_log` | `level`, `message` | `LogEntryOut` |
| `list_logs` | `limit?` | `LogEntryOut[]`（升序） |
| `clear_logs` | – | `bool` |
| `read_logs` | – | `string[]`（预格式化） |
| `get_conn_state` | – | `string`（idle/connecting/connected/disconnecting/error） |
| `set_conn_state` | `state` | `bool` |
| `start_tunnel` | `hostId`, `host`, `port`, `username` | `()`（mock，写日志+状态） |
| `stop_tunnel` | `hostId` | `()`（mock） |
| `pick_key_file` | – | `string`（空串=取消） |

## 数据库

- 文件：`app_data_dir()/ssh_tunnel.db`，启动时自动建表
  （`tauri::async_runtime::block_on(db::init_db(...))`，setup 中执行）。
- 迁移：`db/migrations/001_init.sql` 由 `db::init_db` 按分号分割逐条执行（`run_migration_sql`，兼容 SQLite 多语句限制）；
  变更结构时**直接修改该文件**（本地应用，无版本化迁移）。
- `PRAGMA foreign_keys = ON` 由 `SqliteConnectOptions::foreign_keys(true)` 开启，
  保证 `ON DELETE CASCADE` 生效。
- 新增出入参模型：先在 `models.rs` 定义（行结构 + DTO），
  命令模块内 `query_as::<_, Row>` 读取，`Row::into_out` 转前端结构。

## 共享状态（app.manage 注册）

- `AppDb(pub SqlitePool)`：所有命令通过 `State<'_, AppDb>` 获取连接池。
- `AppConnState(RwLock<String>)`：内存连接状态机，不落库，
  隧道命令与 `get/set_conn_state` 共用。

## 构建与验证

```sh
cd src-tauri
cargo check        # 快速编译校验
npm run tauri dev  # 在仓库根目录启动开发态应用
```

新增命令的标准流程：
1. `commands/<域>.rs` 写 `#[tauri::command]` 函数；
2. `lib.rs` 的 `tauri::generate_handler![...]` 追加注册；
3. `models.rs` 补充 DTO，并用 `cargo check` 验证。

## 注意事项

- 不要改 `Cargo.toml` 里 tauri 2 全家桶的版本策略（固定 major 版本）。
- 密码明文存于 DB（本地工具可接受）；后续如需增强改为系统钥匙串。
- 命令返回错误统一 `Err(String)`，中文文案，db_err 前缀标注数据库上下文。
- 新增/修改 SQL 后必须同步 `001_init.sql`，保持“建表即可运行”一致。