# 0035. AI credit wallet & ledger

> Feature 56 (Slice 10: AI foundation)

## Summary

This specification defines the AI Credit Wallet and Ledger system. AI usage across Coachify is metered through institutional credits purchased via the Coachify website. 

To maintain total transparency and financial integrity, credit transactions follow the immutable ledger pattern established in Slice 2: every credit deduction, recharge, refund, and promotional credit is recorded as an immutable row in an append-only SQLite ledger table (`ai_credit_transactions`). The desktop app provides an instant balance indicator in the app shell, links directly to the web checkout portal, and syncs balances automatically.

## Requirements

**User stories**:
* As an institution owner, I want to see our remaining AI credits at a glance on the copilot panel and settings screen so that I know when we need to recharge.
* As an accountant, I want an itemized history of every credit deduction showing which user asked the question, the date/time, and how many credits were consumed.
* As an owner, I want a single-click "Buy More Credits" button that opens our authenticated Coachify web account where I can pay via bKash, Nagad, or card.
* As an administrator who just purchased credits online, I want our local desktop balance to refresh immediately without restarting the application.

**Acceptance criteria**:
* **AC-1**: `ai_credit_wallets` table maintains the active balance, gateway endpoint, and token reference; initialization seeds a starting balance (e.g. 500 promotional starter credits for early access).
* **AC-2**: Every copilot interaction deducts an integer amount of credits matching the gateway response token calculation and logs a transaction in `ai_credit_transactions` with `transaction_type = 'usage'`, linking the `reference_id` to the copilot `request_id`.
* **AC-3**: Local balance is guarded against negative overdrafts: if `balance_credits <= 0`, outgoing AI requests fail immediately with `credits_exhausted` before dispatching network requests.
* **AC-4**: Query `ai.get_wallet_balance` delivers current balance, active status, last sync timestamp, and gateway configuration status in under 10 ms.
* **AC-5**: Query `ai.get_credit_ledger` returns paginated historical transactions (`purchase`, `usage`, `refund`, `promotional`, `sync_adjustment`) with user attribution, amounts, and post-transaction balances.
* **AC-6**: Command `ai.sync_credits` issues an authenticated request to `GET /v1/wallet/balance` at the Cloud Gateway, reconciles new cloud purchases into local transactions, and updates the wallet balance.
* **AC-7**: The UI provides a "Buy Credits" button opening the browser to `https://coachify.cloud/account/credits?token=...` with the institution's license context.

## Inline rationale

Local ledger tracking guarantees that credit usage is completely auditable even if the coaching center PC is offline or the internet fluctuates mid-day. Mirroring the append-only ledger pattern from Slice 2 ensures that balances cannot be tampered with or silently decremented without leaving a permanent audit trail.

## Data model details

```sql
-- Wallet balance snapshot
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

-- Append-only credit ledger
CREATE TABLE IF NOT EXISTS ai_credit_transactions (
    id TEXT PRIMARY KEY NOT NULL,
    wallet_id TEXT NOT NULL REFERENCES ai_credit_wallets(id),
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('purchase', 'usage', 'refund', 'promotional', 'sync_adjustment')),
    amount INTEGER NOT NULL, -- positive for recharges, negative for usage
    balance_after INTEGER NOT NULL,
    reference_id TEXT, -- request_id or external invoice id
    description TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id)
);
```

## API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `ai.get_wallet_balance` | query | _None_ | `AiWalletBalance` (balance_credits, is_active, last_synced_at, gateway_configured) | Any authenticated | _None_ |
| `ai.get_credit_ledger` | query | `limit: Option<u32>`, `before_time: Option<String>` | `Vec<CreditTransactionItem>` | Owner, Admin, Cashier | `invalid_parameters` |
| `ai.sync_credits` | command | _None_ | `AiWalletBalance` | Owner, Admin | `gateway_unreachable`, `gateway_auth_failed` |
| `ai.record_credit_grant` | command | `amount: i64`, `description: String` | `CreditTransactionItem` | System / Migration / Owner | `negative_grant` |

## Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| `get_wallet_balance` | current credit balance | Read from `ai_credit_wallets.balance_credits` |
| `get_credit_ledger` | transaction history list | Read from `ai_credit_transactions` ordered by `created_at DESC` |
| `sync_credits` | updated balance and remote recharges | Fetched from Cloud Gateway `/v1/wallet/balance` |
| `ask_copilot` | credit deduction transaction | Inserted into `ai_credit_transactions` on completed response |

## Build tasks

1. Implement `crates/core/src/ai/wallet.rs` with atomic balance updates and transaction logging within SQLite transactions, satisfies **AC-1**, **AC-2**, **AC-3**
2. Implement `ai.sync_credits` contacting the Cloud AI Gateway to reconcile remote purchases and sync local balance, satisfies **AC-6**
3. Wire `ai.get_wallet_balance` and `ai.get_credit_ledger` queries in `dispatcher.rs`, satisfies **AC-4**, **AC-5**
4. Build `AiWalletView` in React showing balance card, quick top-up web link, and transaction history table, satisfies **AC-4**, **AC-5**, **AC-7**
