-- Migration 0026: Mobile App and LAN QR Pairing

-- 1. Track paired devices with mobile platform and pairing origin
ALTER TABLE devices ADD COLUMN paired_via TEXT NOT NULL DEFAULT 'code' 
    CHECK (paired_via IN ('code', 'qr', 'manual'));

ALTER TABLE devices ADD COLUMN platform TEXT;
ALTER TABLE devices ADD COLUMN device_type TEXT;

-- 2. Ensure pairing sessions track QR secret hashes
ALTER TABLE lan_pairing_sessions ADD COLUMN qr_secret_hash TEXT;

-- Create unique index for instant QR secret lookup and uniqueness enforcement
CREATE UNIQUE INDEX IF NOT EXISTS idx_lan_pairing_sessions_qr_hash 
    ON lan_pairing_sessions(qr_secret_hash);

-- 3. Designated host receipt printer for mobile print routing
ALTER TABLE lan_host_config ADD COLUMN receipt_printer_name TEXT;

-- 4. Staff PIN hash for mobile staff quick PIN login
ALTER TABLE users ADD COLUMN pin_hash TEXT;
