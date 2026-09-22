# 0024. Omnichannel outbox and provider adapters (Rationale)

## Context

In Bangladesh, messaging to guardians is often the single most visible operational feature for parents: receiving immediate confirmation that their tuition payment was received, an attendance alert when their child is absent from class, or monthly tuition fee due statements.

However, coaching centers operate under severe infrastructure constraints:
1. Unstable internet connectivity: Broadband connections routinely drop for hours due to localized fiber cuts, bad weather, or ISP outages.
2. Aggressive or unreliable local SMS gateways: Many local aggregators lack idempotency keys, rate limits are strict, and server responses frequently hang or time out even after the gateway has successfully queued the message for cellular transmission.
3. WhatsApp anti ban risks: Disagreeable blast bursts without natural human pacing trigger automated phone number bans from Meta.
4. Counter throughput sensitivity: Cashiers cannot wait 3 to 10 seconds for an external network request to finish while a long line of parents waits at the counter.

If software attempts to send messages synchronously during a payment transaction, any network hiccup causes the cashier counter to freeze. If the software blindly retries after a timeout, parents receive multiple duplicate messages and prepaid credit balances drain rapidly.

Coachify solves this with a durable, append only Transactional Outbox directly inside SQLite, combined with an explicit uncertain status state that prevents automated duplicate sends, quiet hours scheduling, and pluggable channel adapters for SMS, WhatsApp, and Email.

---

## Options Considered

### Option 1: Durable Transactional SQLite Outbox with Desktop Worker, Uncertain Timeout Handling, and Pluggable Adapters (Chosen)

Domain commands (such as payment collection or attendance saving) insert outbox records in the same atomic database transaction.
- The business operation commits in under 50 ms without waiting on the internet or external gateways.
- A desktop background worker claims pending messages and dispatches them asynchronously every 30 seconds.
- If a request drops connection before receiving an acknowledgment, it is flagged uncertain and never retried automatically.
- Bundled OpenWA child process provides free rich WhatsApp messaging with persistent sessions and randomized 3 to 6 second anti ban pacing.
- Pluggable adapters allow switching between Greenweb, BulkSMS BD, Generic HTTP, OpenWA, and SMTP with credentials protected via Windows DPAPI.

Pros:
- High cashier speed: payment and attendance saves are completely decoupled from external network latency.
- Guaranteed delivery on reconnect: messages queued during an outage send automatically when connectivity returns.
- Zero duplicate billing: timeouts never trigger automated resends.
- Omnichannel flexibility: institutions choose the right channel for each message type.
- Secrets security: credentials encrypted at rest with DPAPI, never logged.

Cons:
- Delivery is near real time rather than instantaneous.
- Requires staff to periodically check the outbox UI to review any uncertain messages.

### Option 2: Synchronous Inline HTTP Dispatch During Domain Command

Trigger network requests directly inside the payment or attendance command before returning a response to the UI.

Pros:
- Immediate confirmation in the UI whether the message succeeded.

Cons:
- Fatal counter bottleneck: a 5 second network timeout makes fee collection painfully sluggish.
- Offline failure: if the internet is down, either the payment must fail (unacceptable) or the message is dropped permanently.
- Distributed transaction failure: if the message succeeds but the database write fails, records diverge.

### Option 3: External Message Broker (RabbitMQ, Redis, or Celery)

Use an external message broker service to manage queues.

Pros:
- Standard enterprise architecture pattern.

Cons:
- Incompatible with a self contained, offline first desktop Windows application running on modest 4 GB RAM PCs.
- Adds unnecessary daemon processes, maintenance overhead, and failure surfaces.
