//! 主机 CRUD 命令。

use tauri::State;

use crate::db::{db_err, gen_id, AppDb};
use crate::models::{ForwardRuleOut, HostProfileOut, HostRow, RuleRow, SaveHostInput};

const SELECT_HOST_SQL: &str = "SELECT id, name, host, port, username, password, key_path FROM hosts";
const SELECT_RULES_SQL: &str =
    "SELECT id, rule_type, enabled, listen_addr, target_addr, note FROM forward_rules WHERE host_id = ?";

/// 查询某主机的全部转发规则（供 hosts / rules / tunnel 共用）
pub async fn load_rules(
    pool: &sqlx::SqlitePool,
    host_id: &str,
) -> Result<Vec<ForwardRuleOut>, String> {
    let rows = sqlx::query_as::<_, RuleRow>(SELECT_RULES_SQL)
        .bind(host_id)
        .fetch_all(pool)
        .await
        .map_err(|e| db_err(e, "查询规则失败"))?;
    Ok(rows.into_iter().map(RuleRow::into_out).collect())
}

async fn get_host_row(db: &AppDb, id: &str) -> Result<HostRow, String> {
    sqlx::query_as::<_, HostRow>(&format!("{SELECT_HOST_SQL} WHERE id = ?"))
        .bind(id)
        .fetch_one(&db.0)
        .await
        .map_err(|e| db_err(e, "读取主机失败"))
}

/// 列出全部主机（内嵌各自规则）
#[tauri::command]
pub async fn list_hosts(db: State<'_, AppDb>) -> Result<Vec<HostProfileOut>, String> {
    let rows = sqlx::query_as::<_, HostRow>(SELECT_HOST_SQL)
        .fetch_all(&db.0)
        .await
        .map_err(|e| db_err(e, "查询主机列表失败"))?;
    let mut hosts = Vec::with_capacity(rows.len());
    for row in rows {
        let rules = load_rules(&db.0, &row.id).await?;
        hosts.push(row.into_out(rules));
    }
    Ok(hosts)
}

/// 查询单个主机（含规则）
#[tauri::command]
pub async fn get_host(
    db: State<'_, AppDb>,
    host_id: String,
) -> Result<Option<HostProfileOut>, String> {
    let row = sqlx::query_as::<_, HostRow>(&format!("{SELECT_HOST_SQL} WHERE id = ?"))
        .bind(&host_id)
        .fetch_optional(&db.0)
        .await
        .map_err(|e| db_err(e, "查询主机失败"))?;
    match row {
        Some(r) => {
            let rules = load_rules(&db.0, &r.id).await?;
            Ok(Some(r.into_out(rules)))
        }
        None => Ok(None),
    }
}

/// 保存主机：`input.id` 非空且存在则更新，否则新建，返回完整主机（含规则）
#[tauri::command]
pub async fn save_host(
    db: State<'_, AppDb>,
    input: SaveHostInput,
) -> Result<HostProfileOut, String> {
    let id = match &input.id {
        Some(id) if !id.is_empty() => {
            sqlx::query(
                "UPDATE hosts SET name = ?, host = ?, port = ?, username = ?, password = ?, key_path = ?, updated_at = strftime('%s','now') WHERE id = ?",
            )
            .bind(&input.name)
            .bind(&input.host)
            .bind(input.port)
            .bind(&input.username)
            .bind(&input.password)
            .bind(&input.key_path)
            .bind(id)
            .execute(&db.0)
            .await
            .map_err(|e| db_err(e, "更新主机失败"))?;
            id.clone()
        }
        _ => {
            let id = gen_id("host");
            sqlx::query(
                "INSERT INTO hosts (id, name, host, port, username, password, key_path) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&input.name)
            .bind(&input.host)
            .bind(input.port)
            .bind(&input.username)
            .bind(&input.password)
            .bind(&input.key_path)
            .execute(&db.0)
            .await
            .map_err(|e| db_err(e, "保存主机失败"))?;
            id
        }
    };
    let out = get_host_row(&db, &id).await?;
    let rules = load_rules(&db.0, &id).await?;
    Ok(out.into_out(rules))
}

/// 删除主机（级联删除其全部规则）
#[tauri::command]
pub async fn delete_host(db: State<'_, AppDb>, host_id: String) -> Result<bool, String> {
    let r = sqlx::query("DELETE FROM hosts WHERE id = ?")
        .bind(&host_id)
        .execute(&db.0)
        .await
        .map_err(|e| db_err(e, "删除主机失败"))?;
    Ok(r.rows_affected() > 0)
}