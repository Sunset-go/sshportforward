-- 应用设置键值表：持久化用户偏好（如关闭行为 close_to_tray）
CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
