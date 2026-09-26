-- Migration 0024: LAN Host Service and Discovery Configuration

CREATE TABLE IF NOT EXISTS lan_host_config (
    id TEXT PRIMARY KEY, -- 'default'
    is_enabled INTEGER NOT NULL DEFAULT 0 CHECK (is_enabled IN (0, 1)),
    port INTEGER NOT NULL DEFAULT 50443 CHECK (port BETWEEN 1024 AND 65535),
    bind_address TEXT NOT NULL DEFAULT '0.0.0.0',
    tls_cert_pem TEXT NOT NULL,
    tls_key_pem TEXT NOT NULL,
    tls_fingerprint_sha256 TEXT NOT NULL,
    mdns_enabled INTEGER NOT NULL DEFAULT 1 CHECK (mdns_enabled IN (0, 1)),
    auto_start_on_boot INTEGER NOT NULL DEFAULT 0 CHECK (auto_start_on_boot IN (0, 1)),
    prevent_sleep INTEGER NOT NULL DEFAULT 1 CHECK (prevent_sleep IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Audit trigger: prevent deletion of host configuration
CREATE TRIGGER IF NOT EXISTS prevent_delete_lan_host_config
BEFORE DELETE ON lan_host_config
BEGIN
    SELECT RAISE(ABORT, 'LAN host configuration record cannot be deleted');
END;
