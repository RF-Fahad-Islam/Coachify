-- Migration 0023: License Deactivation State for Portal Device Management
-- Compliant with Spec 0043 (Feature 68)

ALTER TABLE license_refresh_state ADD COLUMN deactivated_at TEXT;
ALTER TABLE license_refresh_state ADD COLUMN deactivation_reason TEXT;
