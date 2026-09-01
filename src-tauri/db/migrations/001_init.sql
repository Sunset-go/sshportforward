-- 001_init.sql: SSH 端口映射工具建表
-- 所有表保持 snake_case 内部命名，对外（前端）通过 serde rename 映射为 camelCase。
--
-- 注意：
-- 1. 主键统一为 TEXT（前端 HostProfile.id / ForwardRule.id 为字符串，如 'host-pc' / 'r-1'）
-- 2. enabled 以 0/1 整数存储，Rust 侧转 bool
-- 3. updated_at / created_at 为 UNIX 秒（strftime('%s','now')）
-- 4. 删除主机时经 FK ON DELETE CASCADE 级联删除规则（需开启 PRAGMA foreign_keys）

CREATE TABLE IF NOT EXISTS hosts (
    id          TEXT    PRIMARY KEY,          -- 字符串主键，如 'host-pc'
    name        TEXT    NOT NULL,
    host        TEXT    NOT NULL,
    port        INTEGER NOT NULL DEFAULT 22,
    username    TEXT    NOT NULL DEFAULT '',
    password    TEXT    NOT NULL DEFAULT '',
    key_path    TEXT    NOT NULL DEFAULT '',
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS forward_rules (
    id          TEXT    PRIMARY KEY,          -- 字符串主键，如 'r-1'
    host_id     TEXT    NOT NULL,
    rule_type   TEXT    NOT NULL,             -- 'local' | 'remote' | 'dynamic'
    enabled     INTEGER NOT NULL DEFAULT 1,   -- 0/1
    listen_addr TEXT    NOT NULL,
    target_addr TEXT    NOT NULL,
    note        TEXT    NOT NULL DEFAULT '',
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS logs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    level       TEXT    NOT NULL,             -- 'INFO' | 'SUCCESS' | 'WARN' | 'ERROR'
    message     TEXT    NOT NULL,
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE INDEX IF NOT EXISTS idx_rules_host ON forward_rules(host_id);