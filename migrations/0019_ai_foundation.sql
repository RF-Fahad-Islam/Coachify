-- Migration 0019: AI Foundation, Credit Wallet, Privacy Center, and Copilot Shell
-- Compliant with Spec 0035

-- 1. Privacy Settings (singleton row per institution)
CREATE TABLE IF NOT EXISTS ai_privacy_settings (
    id TEXT PRIMARY KEY NOT NULL DEFAULT 'default',
    allow_student_names INTEGER NOT NULL DEFAULT 0,
    allow_phone_numbers INTEGER NOT NULL DEFAULT 0,
    allow_guardian_info INTEGER NOT NULL DEFAULT 0,
    allow_financial_figures INTEGER NOT NULL DEFAULT 0,
    allow_exam_marks INTEGER NOT NULL DEFAULT 0,
    allow_staff_names INTEGER NOT NULL DEFAULT 0,
    custom_redaction_rules TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);

-- Seed conservative default privacy settings
INSERT OR IGNORE INTO ai_privacy_settings (
    id, allow_student_names, allow_phone_numbers, allow_guardian_info,
    allow_financial_figures, allow_exam_marks, allow_staff_names, updated_at
) VALUES ('default', 0, 0, 0, 0, 0, 0, datetime('now'));

-- 2. AI Credit Wallet (singleton row)
CREATE TABLE IF NOT EXISTS ai_credit_wallets (
    id TEXT PRIMARY KEY NOT NULL DEFAULT 'default',
    balance_credits INTEGER NOT NULL DEFAULT 0,
    gateway_url TEXT NOT NULL DEFAULT 'https://ai.coachify.cloud/v1',
    token_dpapi_blob BLOB,
    last_synced_at TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Seed wallet with 500 promotional starter credits
INSERT OR IGNORE INTO ai_credit_wallets (
    id, balance_credits, gateway_url, created_at, updated_at
) VALUES ('default', 500, 'https://ai.coachify.cloud/v1', datetime('now'), datetime('now'));

-- 3. AI Credit Ledger (append-only auditable transaction log)
CREATE TABLE IF NOT EXISTS ai_credit_transactions (
    id TEXT PRIMARY KEY NOT NULL,
    wallet_id TEXT NOT NULL REFERENCES ai_credit_wallets(id),
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('purchase', 'usage', 'refund', 'promotional', 'sync_adjustment')),
    amount INTEGER NOT NULL,
    balance_after INTEGER NOT NULL,
    reference_id TEXT,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_ai_credit_tx_wallet ON ai_credit_transactions(wallet_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ai_credit_tx_created ON ai_credit_transactions(created_at DESC);

-- Record promotional grant transaction in ledger for initial 500 credits
INSERT OR IGNORE INTO ai_credit_transactions (
    id, wallet_id, transaction_type, amount, balance_after, reference_id, description, created_at
) VALUES (
    '019213ab-7c82-7000-8000-000000000019',
    'default',
    'promotional',
    500,
    500,
    'init_promotional_grant',
    'Starter AI credit allowance for institution trial',
    datetime('now')
);

-- 4. Copilot Conversations and Message History
CREATE TABLE IF NOT EXISTS ai_copilot_conversations (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_conv_user ON ai_copilot_conversations(user_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS ai_copilot_messages (
    id TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL REFERENCES ai_copilot_conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system', 'tool')),
    content TEXT NOT NULL,
    tool_calls_json TEXT,
    tool_result_json TEXT,
    credits_deducted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_messages_conv ON ai_copilot_messages(conversation_id, created_at ASC);

-- 5. AI Request Audit Log
CREATE TABLE IF NOT EXISTS ai_audit_logs (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL REFERENCES users(id),
    action_type TEXT NOT NULL CHECK (action_type IN ('prompt', 'tool_call', 'privacy_redaction', 'credit_sync')),
    prompt_summary TEXT NOT NULL,
    tools_invoked TEXT NOT NULL DEFAULT '[]',
    categories_allowed TEXT NOT NULL DEFAULT '[]',
    categories_redacted TEXT NOT NULL DEFAULT '[]',
    redaction_count INTEGER NOT NULL DEFAULT 0,
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    credits_used INTEGER NOT NULL DEFAULT 0,
    latency_ms INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('success', 'error', 'credits_exhausted', 'rate_limited', 'gateway_offline')),
    error_message TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_audit_created ON ai_audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ai_audit_user ON ai_audit_logs(user_id, created_at DESC);
