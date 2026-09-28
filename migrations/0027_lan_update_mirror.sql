-- Migration 0027: LAN Host Update Mirror Configuration & Staged Releases

CREATE TABLE IF NOT EXISTS lan_update_mirror_config (
    id TEXT PRIMARY KEY, -- 'default'
    mirror_enabled INTEGER NOT NULL DEFAULT 1 CHECK (mirror_enabled IN (0, 1)),
    storage_path TEXT, -- relative or absolute directory, defaults to app data update_mirror
    max_cached_releases INTEGER NOT NULL DEFAULT 2,
    auto_stage_on_host_update INTEGER NOT NULL DEFAULT 1 CHECK (auto_stage_on_host_update IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS lan_staged_updates (
    id TEXT PRIMARY KEY, -- release version '1.1.0'
    version TEXT UNIQUE NOT NULL,
    channel TEXT NOT NULL DEFAULT 'stable' CHECK (channel IN ('internal', 'beta', 'stable')),
    artifact_filename TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    updater_signature TEXT NOT NULL,
    updater_key_id TEXT NOT NULL DEFAULT 'u2026a',
    notes TEXT NOT NULL DEFAULT '',
    min_upgrade_from TEXT,
    staged_at TEXT NOT NULL,
    downloads_count INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
);

-- Trigger: Prevent deletion of default configuration
CREATE TRIGGER IF NOT EXISTS prevent_delete_lan_update_mirror_config
BEFORE DELETE ON lan_update_mirror_config
BEGIN
    SELECT RAISE(ABORT, 'LAN update mirror configuration record cannot be deleted');
END;
