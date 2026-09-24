# AI foundation (Slice 10, minimal AI for early customers)

Back to [index](index.md). A minimal AI layer so early customers can try natural language interaction. The full AI feature set (morning brief, natural language reports, communication writer, anomaly guard, etc.) is deferred to 1.1. The architectural rule holds: institution data lives locally; the cloud AI gateway handles only the AI inference, never stores customer data.

### 54. AI copilot shell · GA
Natural language assistant that queries Coachify through controlled tools. The copilot can read (never write) Coachify data through a defined tool set. It answers questions about students, fees, attendance, and batches using the institution's own data.
**Done when:** the copilot panel opens from the app shell; a natural language question like "how many students in Physics A have dues over 5000" returns the correct answer; the copilot never has direct SQL access; every query is logged; the copilot works through the cloud AI gateway.
spec [0036](../specs/0036-ai-foundation/0036-ai-copilot-shell.md)
- [x] Design it (spec): `/architect AI copilot shell`
- [x] Build it: `/develop AI copilot shell`
  - [x] Migration 0020 `ai_copilot_conversations` and `ai_copilot_messages` schema, covers AC-1, AC-5
  - [x] 6 controlled read-only tools querying students, dues, attendance, batches, exams, covers AC-3, AC-4
  - [x] Core `ai.ask_copilot` conversational orchestrator with tool execution loop, covers AC-2, AC-3
  - [x] App shell `CopilotDrawer` UI with `Ctrl+J` shortcut listener and message stream, covers AC-1, AC-2
- [x] Verify it: `/check verify AI copilot shell`
- [x] Test it: `/test AI copilot shell`
- [x] Review it (fresh model): `/check review AI copilot shell`
- [x] Document it: `/document AI copilot shell`

### 55. Cloud AI gateway · GA
Coachify's hosted gateway that proxies AI requests. The institution's data is sent to the gateway for inference and never stored. The AI provider credentials (OpenAI, Anthropic, etc.) live on Coachify's server, never inside the desktop app. Rate limiting, usage tracking, and credit deduction happen at the gateway.
**Done when:** the desktop app sends AI requests to the gateway with the institution's credit token; the gateway proxies to the AI provider and returns the response; no AI provider keys exist on the desktop; usage is tracked per request; the gateway rejects requests when credits are exhausted.
spec [0036](../specs/0036-ai-foundation/0036-cloud-ai-gateway.md)
- [x] Design it (spec): `/architect cloud AI gateway`
- [x] Build it: `/develop cloud AI gateway`
  - [x] HTTPS client adapter and request envelope with bearer token header, covers AC-1, AC-4
  - [x] Windows DPAPI encryption wrapper for institution gateway token, covers AC-6
  - [x] Zero retention compliance and usage token accounting from gateway responses, covers AC-2, AC-3, AC-5
  - [x] Headless `MockAiGateway` test harness with programmable responses and latency injection, covers AC-7
- [x] Verify it: `/check verify cloud AI gateway`
- [x] Test it: `/test cloud AI gateway`
- [x] Review it (fresh model): `/check review cloud AI gateway`
- [x] Document it: `/document cloud AI gateway`

### 56. AI credit wallet & ledger · GA
Institutions buy Coachify AI credits from the website. Credits are an auditable ledger of purchase, usage, refund, and promotional transactions. The desktop app shows the current balance and links to the purchase page.
**Done when:** the desktop app displays the current AI credit balance; each AI request deducts credits and logs the transaction; the ledger shows purchase, usage, and refund history; a "buy more" button opens the Coachify web account; credits sync automatically after purchase.
spec [0036](../specs/0036-ai-foundation/0036-ai-credit-wallet-and-ledger.md)
- [x] Design it (spec): `/architect AI credit wallet & ledger`
- [x] Build it: `/develop AI credit wallet & ledger`
  - [x] Migration 0020 `ai_credit_wallets` and `ai_credit_transactions` ledger tables, covers AC-1
  - [x] Atomic credit deduction and zero-overdraft pre-flight check in `ai::wallet`, covers AC-2, AC-3
  - [x] `ai.sync_credits` remote synchronization handler and balance query RPCs, covers AC-4, AC-6
  - [x] `AiWalletView` UI with balance indicator, transaction history table, and web checkout link, covers AC-4, AC-5, AC-7
- [x] Verify it: `/check verify AI credit wallet & ledger`
- [x] Test it: `/test AI credit wallet & ledger`
- [x] Review it (fresh model): `/check review AI credit wallet & ledger`
- [x] Document it: `/document AI credit wallet & ledger`

### 57. AI privacy center · GA
Institution controls what categories of data may leave the PC for cloud AI processing. The owner configures which data types (student names, phone numbers, financial figures, exam scores) the copilot may send to the cloud gateway. Unchecked categories are redacted before sending.
**Done when:** the privacy settings screen shows every data category with a toggle; unchecked categories are redacted from AI requests before they leave the PC; the default configuration is conservative (financial data and personal identifiers off); the audit log records what categories were sent for each AI request.
spec [0036](../specs/0036-ai-foundation/0036-ai-privacy-center.md)
- [x] Design it (spec): `/architect AI privacy center`
- [x] Build it: `/develop AI privacy center`
  - [x] Migration 0020 `ai_privacy_settings` schema with conservative defaults, covers AC-1, AC-2
  - [x] Client-side `PrivacyRedactionEngine` token pseudonym transformer, covers AC-3
  - [x] Ephemeral in-memory de-anonymization dictionary for local UI re-hydration, covers AC-4
  - [x] `ai_audit_logs` persistence and `AiPrivacyCenterView` settings management UI, covers AC-5, AC-6, AC-7
- [x] Verify it: `/check verify AI privacy center`
- [x] Test it: `/test AI privacy center`
- [x] Review it (fresh model): `/check review AI privacy center`
- [x] Document it: `/document AI privacy center`
