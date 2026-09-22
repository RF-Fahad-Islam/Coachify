# AI foundation (Slice 10, minimal AI for early customers)

Back to [index](index.md). A minimal AI layer so early customers can try natural language interaction. The full AI feature set (morning brief, natural language reports, communication writer, anomaly guard, etc.) is deferred to 1.1. The architectural rule holds: institution data lives locally; the cloud AI gateway handles only the AI inference, never stores customer data.

### 54. AI copilot shell · needs a decision · GA
Natural language assistant that queries Coachify through controlled tools. The copilot can read (never write) Coachify data through a defined tool set. It answers questions about students, fees, attendance, and batches using the institution's own data.
**Done when:** the copilot panel opens from the app shell; a natural language question like "how many students in Physics A have dues over 5000" returns the correct answer; the copilot never has direct SQL access; every query is logged; the copilot works through the cloud AI gateway.
- [ ] Design it (spec): `/architect AI copilot shell`

### 55. Cloud AI gateway · needs a decision · GA
Coachify's hosted gateway that proxies AI requests. The institution's data is sent to the gateway for inference and never stored. The AI provider credentials (OpenAI, Anthropic, etc.) live on Coachify's server, never inside the desktop app. Rate limiting, usage tracking, and credit deduction happen at the gateway.
**Done when:** the desktop app sends AI requests to the gateway with the institution's credit token; the gateway proxies to the AI provider and returns the response; no AI provider keys exist on the desktop; usage is tracked per request; the gateway rejects requests when credits are exhausted.
- [ ] Design it (spec): `/architect cloud AI gateway`

### 56. AI credit wallet & ledger · needs a decision · GA
Institutions buy Coachify AI credits from the website. Credits are an auditable ledger of purchase, usage, refund, and promotional transactions. The desktop app shows the current balance and links to the purchase page.
**Done when:** the desktop app displays the current AI credit balance; each AI request deducts credits and logs the transaction; the ledger shows purchase, usage, and refund history; a "buy more" button opens the Coachify web account; credits sync automatically after purchase.
- [ ] Design it (spec): `/architect AI credit wallet & ledger`

### 57. AI privacy center · needs a decision · GA
Institution controls what categories of data may leave the PC for cloud AI processing. The owner configures which data types (student names, phone numbers, financial figures, exam scores) the copilot may send to the cloud gateway. Unchecked categories are redacted before sending.
**Done when:** the privacy settings screen shows every data category with a toggle; unchecked categories are redacted from AI requests before they leave the PC; the default configuration is conservative (financial data and personal identifiers off); the audit log records what categories were sent for each AI request.
- [ ] Design it (spec): `/architect AI privacy center`
