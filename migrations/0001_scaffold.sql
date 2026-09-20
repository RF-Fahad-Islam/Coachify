-- Initial scaffold migration placeholder
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (1, datetime('now'));
