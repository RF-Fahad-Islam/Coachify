-- Migration 0025: Workstation Pairing and Device Management

-- 1. Extend devices table with LAN IP, OS metadata, and revocation fields
ALTER TABLE devices ADD COLUMN ip_address TEXT;
ALTER TABLE devices ADD COLUMN os_version TEXT;
ALTER TABLE devices ADD COLUMN blocked_at TEXT;
ALTER TABLE devices ADD COLUMN revoked_at TEXT;
ALTER TABLE devices ADD COLUMN revocation_reason TEXT;

-- Index for fast status and token lookups
CREATE INDEX IF NOT EXISTS idx_devices_status ON devices(status);
CREATE INDEX IF NOT EXISTS idx_devices_last_seen ON devices(last_seen_at DESC);

-- 2. Ephemeral pairing sessions table (managed by host)
CREATE TABLE IF NOT EXISTS lan_pairing_sessions (
    id TEXT PRIMARY KEY, -- UUIDv7
    code_hash TEXT NOT NULL, -- SHA256(code || ':' || salt)
    salt TEXT NOT NULL,      -- 16-byte random hex salt
    expires_at TEXT NOT NULL,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending' 
        CHECK (status IN ('pending', 'used', 'expired', 'cancelled', 'locked')),
    created_by_user_id TEXT NOT NULL REFERENCES users(id),
    paired_device_id TEXT REFERENCES devices(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pairing_lookup 
    ON lan_pairing_sessions(status, expires_at);

-- 3. Immutable device audit events table
CREATE TABLE IF NOT EXISTS device_events (
    id TEXT PRIMARY KEY, -- UUIDv7
    device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL 
        CHECK (event_type IN ('paired', 'renamed', 'blocked', 'unblocked', 'revoked', 'token_reissued')),
    actor_user_id TEXT REFERENCES users(id),
    remote_ip TEXT,
    details_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_device_events_device 
    ON device_events(device_id, created_at DESC);

-- Prevent manual update or deletion of immutable device events
CREATE TRIGGER IF NOT EXISTS trg_prevent_modify_device_events
BEFORE UPDATE ON device_events
BEGIN
    SELECT RAISE(ABORT, 'device_events records are append-only and cannot be modified');
END;

CREATE TRIGGER IF NOT EXISTS trg_prevent_delete_device_events
BEFORE DELETE ON device_events
BEGIN
    SELECT RAISE(ABORT, 'device_events records cannot be deleted');
END;
