//! Tauri 后端命令模块。
//!
//! 每个子模块对应用前端的业务域：
//! - `hosts`：主机 CRUD（`HostProfile`）
//! - `rules`：转发规则 CRUD（`ForwardRule`）
//! - `logs`：日志读写（`LogEntry` / `read_logs`）
//! - `tunnel`：隧道接通/断开、连接状态机、私钥选择
//! - `settings`：应用设置读写（app_settings 表）、窗口隐藏与退出

pub mod hosts;
pub mod logs;
pub mod rules;
pub mod settings;
pub mod tunnel;