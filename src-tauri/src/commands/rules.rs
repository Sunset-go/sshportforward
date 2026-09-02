//! 转发规则 CRUD 命令。

use tauri::State;

use crate::db::{db_err, gen_id, AppDb};
use crate::models::{ForwardRuleOut, RuleInput, RuleRow};

const SELECT_RULE_SQL: &str =
    "SELECT id, rule_type, enabled, listen_addr, target_addr, note FROM forward_rules WHERE id = ? AND host_id = ?";

async fn get_rule(db: &AppDb, rule_id: &str, host_id: &str) -> Result<ForwardRuleOut, String> {
    let row = sqlx::query_as::<_, RuleRow>(SELECT_RULE_SQL)
        .bind(rule_id)
        .bind(host_id)
        .fetch_one(&db.0)
        .await
        .map_err(|e| db_err(e, "读取规则失败"))?;
    Ok(row.into_out())
}

/// 新增规则（自动生成 `r-` 前缀的字符串 id）
#[tauri::command]
pub async fn add_rule(
    db: State<'_, AppDb>,
    host_id: String,
    input: RuleInput,
) -> Result<ForwardRuleOut, String> {
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM hosts WHERE id = ?")
        .bind(&host_id)
        .fetch_one(&db.0)
        .await
        .map_err(|e| db_err(e, "校验主机失败"))?;
    if exists == 0 {
        return Err("主机不存在".to_string());
    }

    let id = match input.id {
        Some(uid) if !uid.is_empty() => uid,
        _ => gen_id("r"),
    };
    sqlx::query(
        "INSERT INTO forward_rules (id, host_id, rule_type, enabled, listen_addr, target_addr, note) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&host_id)
    .bind(&input.rule_type)
    .bind(input.enabled as i64)
    .bind(&input.listen_addr)
    .bind(&input.target_addr)
    .bind(&input.note)
    .execute(&db.0)
    .await
    .map_err(|e| db_err(e, "新增规则失败"))?;
    get_rule(&db, &id, &host_id).await
}

/// 更新规则
#[tauri::command]
pub async fn update_rule(
    db: State<'_, AppDb>,
    host_id: String,
    rule_id: String,
    input: RuleInput,
) -> Result<ForwardRuleOut, String> {
    let target_id = if !rule_id.is_empty() {
        rule_id
    } else {
        input.id.clone().unwrap_or_default()
    };
    if target_id.is_empty() {
        return Err("缺少规则 id".to_string());
    }

    let r = sqlx::query(
        "UPDATE forward_rules SET rule_type = ?, enabled = ?, listen_addr = ?, target_addr = ?, note = ?, updated_at = strftime('%s','now') WHERE id = ? AND host_id = ?",
    )
    .bind(&input.rule_type)
    .bind(input.enabled as i64)
    .bind(&input.listen_addr)
    .bind(&input.target_addr)
    .bind(&input.note)
    .bind(&target_id)
    .bind(&host_id)
    .execute(&db.0)
    .await
    .map_err(|e| db_err(e, "更新规则失败"))?;
    if r.rows_affected() == 0 {
        return Err("规则不存在".to_string());
    }
    get_rule(&db, &target_id, &host_id).await
}

/// 删除规则
#[tauri::command]
pub async fn delete_rule(
    db: State<'_, AppDb>,
    host_id: String,
    rule_id: String,
) -> Result<bool, String> {
    let r = sqlx::query("DELETE FROM forward_rules WHERE id = ? AND host_id = ?")
        .bind(&rule_id)
        .bind(&host_id)
        .execute(&db.0)
        .await
        .map_err(|e| db_err(e, "删除规则失败"))?;
    Ok(r.rows_affected() > 0)
}