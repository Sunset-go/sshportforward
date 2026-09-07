-- 003_passphrase.sql: 为 hosts 表新增私钥密码（passphrase）字段。
-- 与 password / key_path 一样，落库前经 crypto::encrypt 加密；空串原样存储。
ALTER TABLE hosts ADD COLUMN passphrase TEXT NOT NULL DEFAULT '';
