//! 数据模型：数据库行结构（snake_case）+ 对外 DTO（camelCase）。
//!
//! 字段命名与前端 `src/types/index.ts` 对齐：
//! - `HostProfile.id` / `ForwardRule.id` 为**字符串**（如 `'host-pc'` / `'r-1'`）
//! - 日志级别 `LogLevel` 为全大写（`INFO` / `SUCCESS` / `WARN` / `ERROR`）

use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// `hosts` 表行
#[derive(Debug, Clone, FromRow)]
pub struct HostRow {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub key_path: String,
}

/// `forward_rules` 表行
#[derive(Debug, Clone, FromRow)]
pub struct RuleRow {
    pub id: String,
    pub rule_type: String,
    pub enabled: i64,
    pub listen_addr: String,
    pub target_addr: String,
    pub note: String,
}

/// `logs` 表行
#[derive(Debug, Clone, FromRow)]
pub struct LogRow {
    pub id: i64,
    pub level: String,
    pub message: String,
    pub created_at: i64,
}

/// 主机（含规则），返回给前端
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostProfileOut {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub key_path: String,
    pub rules: Vec<ForwardRuleOut>,
}

/// 转发规则，返回给前端
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardRuleOut {
    pub id: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub enabled: bool,
    pub listen_addr: String,
    pub target_addr: String,
    pub note: String,
}

/// 日志条目，返回给前端（`time` 为 `HH:mm:ss`）
#[derive(Debug, Clone, Serialize)]
pub struct LogEntryOut {
    pub id: i64,
    pub time: String,
    pub level: String,
    pub message: String,
}

/// 保存主机入参（upsert：`id` 存在则更新，否则新建）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveHostInput {
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub key_path: String,
}

/// 规则保存入参（`add_rule` / `update_rule` 共用）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleInput {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub enabled: bool,
    pub listen_addr: String,
    pub target_addr: String,
    pub note: String,
}

impl HostRow {
    pub fn into_out(self, rules: Vec<ForwardRuleOut>) -> HostProfileOut {
        HostProfileOut {
            id: self.id,
            name: self.name,
            host: self.host,
            port: self.port,
            username: self.username,
            password: self.password,
            key_path: self.key_path,
            rules,
        }
    }
}

impl RuleRow {
    pub fn into_out(self) -> ForwardRuleOut {
        ForwardRuleOut {
            id: self.id,
            rule_type: self.rule_type,
            enabled: self.enabled != 0,
            listen_addr: self.listen_addr,
            target_addr: self.target_addr,
            note: self.note,
        }
    }
}

impl LogRow {
    pub fn into_out(self) -> LogEntryOut {
        let time = Local
            .timestamp_opt(self.created_at, 0)
            .single()
            .map(|d| d.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "00:00:00".to_string());
        LogEntryOut {
            id: self.id,
            time,
            level: self.level,
            message: self.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_row_formats_time() {
        let row = LogRow {
            id: 1,
            level: "INFO".into(),
            message: "hello".into(),
            created_at: 0,
        };
        let out = row.into_out();
        assert_eq!(out.time.len(), 8); // HH:mm:ss
        assert_eq!(out.level, "INFO");
    }

    #[test]
    fn rule_row_enabled_bool_conversion() {
        let on = RuleRow {
            id: "r-1".into(),
            rule_type: "local".into(),
            enabled: 1,
            listen_addr: "127.0.0.1:8080".into(),
            target_addr: "localhost:80".into(),
            note: String::new(),
        };
        assert!(on.into_out().enabled);

        let off = RuleRow {
            id: "r-2".into(),
            rule_type: "dynamic".into(),
            enabled: 0,
            listen_addr: "127.0.0.1:1080".into(),
            target_addr: String::new(),
            note: String::new(),
        };
        assert!(!off.into_out().enabled);
    }
}