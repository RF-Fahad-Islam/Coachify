# 0035. AI Foundation (Decision Record & Rationale)

## Context

Coachify is an offline-first desktop operating system for coaching centers in Bangladesh. Most centers operate on modest hardware (often dual-core or quad-core Intel Core i3/i5 computers with 4 GB to 8 GB of RAM and spinning hard drives or budget SSDs) with intermittent internet connections. 

Center owners want natural language interaction—for example, asking *"How many students in Physics Batch A haven't paid their fees this month?"* or *"Who had attendance under 60% this week?"*—to avoid digging through multiple menus and data tables.

However, introducing AI into a coaching management application presents four severe architectural challenges:

1. **Data privacy and confidentiality**: Student personal records, guardian contact numbers, and financial collections must not be exposed to third parties, logged on untrusted servers, or ingested into public LLM training corpuses.
2. **Database security & injection risks**: Allowing an LLM to generate and execute arbitrary SQL against SQLite (`Text-to-SQL`) is dangerous. A prompt injection or hallucinated `DROP`, `UPDATE`, or malformed `JOIN` can cause silent data loss or freeze the desktop process.
3. **Local hardware constraints**: Running a quantized 7B or 8B local LLM (e.g. LLaMA 3, Mistral) via `llama.cpp` or ONNX requires at least 6 GB of available RAM and substantial GPU acceleration, which does not exist on the target reference PC.
4. **Commercial and billing realities**: In Bangladesh, credit cards are rare and Stripe is unavailable. Center owners cannot easily pay monthly USD subscriptions directly to OpenAI or Anthropic.

## Options considered

### Option 1: Local-First Tool-Calling Copilot via Stateless Cloud AI Gateway (Chosen)

In this architecture:
- The desktop app connects to a vendor-hosted Cloud AI Gateway (`ai.coachify.cloud`) that proxies requests to upstream frontier models (Claude 3.5 Sonnet, GPT-4o).
- Upstream API keys reside exclusively on Coachify's secure server.
- The LLM never writes or executes SQL; it operates solely through 6 strictly typed, read-only Rust tools (`query_students_with_dues`, `get_batch_summary`, `get_attendance_stats`, etc.).
- A client-side Privacy Redaction Engine strips personal identifiers and financial numbers *before* sending payloads over the network.
- The local desktop re-hydrates pseudonyms on the user's screen using an ephemeral in-memory dictionary.
- Usage is metered via an auditable local SQLite credit wallet ledger.

**Pros**:
* Maximum security: zero SQL injection possible; the database is read-only during AI queries.
* Maximum privacy: sensitive PII (names, phones, finances) is redacted locally before touching the network.
* Low local footprint: works smoothly on 4 GB RAM PCs since inference occurs in the cloud.
* Commercial alignment: allows institutions to buy Taka-denominated AI credits via bKash/Nagad on the Coachify website.
* Contractual zero retention: gateway operates with upstream Zero Data Retention agreements.

**Cons**:
* Requires an active internet connection to query the copilot (local operations remain 100% offline).
* Multi-hop roundtrip latency (Prompt $\to$ Gateway $\to$ Local Tool Execution $\to$ Gateway $\to$ Final Response).

---

### Option 2: Direct Client-to-Provider API Integration (BYO API Key)

In this approach, the desktop app calls OpenAI or Anthropic APIs directly, requiring each coaching center to create a developer account and paste their API key into Coachify.

**Pros**:
* Avoids hosting a vendor proxy gateway.

**Cons**:
* Disastrous UX for Bangladeshi owners: almost none have international credit cards or know how to manage developer API consoles.
* Major security risk: API keys stored in plaintext or extracted by local computer technicians.
* Prevents unified credit bundling, usage metering, and centralized privacy compliance.

---

### Option 3: Direct Text-to-SQL Generation

In this approach, the LLM is given the full SQLite schema and instructed to generate raw SQL queries for execution against the database.

**Pros**:
* Highly flexible; can answer arbitrary ad-hoc questions.

**Cons**:
* Extreme security and stability risk: prompt injection can craft destructive or leaking SQL queries.
* High latency and hallucination rate: models frequently miscalculate poisha vs taka, join wrong tables, or miss soft-delete filters.
* Completely violates Coachify's core rule of strict domain invariants and defense-in-depth access control.

---

### Option 4: Local Offline LLM Pack (Desktop Bundled Model)

In this approach, a 1B–3B parameter quantized model (e.g., SmolLM, Qwen 2.5 1.5B, or LLaMA 3.2 1B) is bundled or downloaded locally and run on CPU.

**Pros**:
* Operates 100% offline without sending any bytes over the network.

**Cons**:
* Severe performance degradation: on a 4 GB RAM Celeron/Core i3 PC, even a 1.5B model consumes 1.5 GB+ RAM and takes 15–40 seconds per response, causing severe UI lag.
* Poor reasoning and tool-calling fidelity: small sub-3B models struggle with complex tool selection, JSON formatting, and multi-step reasoning.
* Deferred to version 1.1 as an optional advanced add-on for high-spec machines.

---

## Decision

We chose **Option 1: Local-First Tool-Augmented Copilot powered by Vercel AI SDK and OpenRouter via Stateless Cloud AI Gateway with Client-Side Privacy Redaction and Local Credit Ledger**.

This architecture cleanly respects Coachify's foundational boundaries:
1. **Institution records remain sovereign and local**: Only minimal, redacted query parameters and tool results are transmitted.
2. **Zero SQL execution**: The LLM is restricted to 6 controlled, deterministic Rust tools with bounded multi-hop execution.
3. **Conservative default privacy**: Out of the box, no student names, phone numbers, guardian contacts, or financial figures leave the machine.
4. **Transparent integer credit accounting**: Every credit deduction is permanently recorded in SQLite, aligning with the financial ledger principles from Slice 2.

### Why OpenRouter as the Upstream Provider?

1. **Multi-Model Flexibility**: OpenRouter provides a single, unified OpenAI-compatible endpoint (`https://openrouter.ai/api/v1`) that routes to Claude 3.5 Sonnet, GPT-4o, and LLaMA 3.3. This avoids vendor lock-in to OpenAI or Anthropic directly.
2. **Cost-Efficiency & Model Tiering**: The gateway can dynamically route complex analytical questions (e.g. multi-batch due cross-referencing) to `anthropic/claude-3.5-sonnet` while routing simple lookups to `openai/gpt-4o-mini`, cutting token consumption costs by over 70%.
3. **Transparent Usage Headers**: OpenRouter returns explicit token usage and cost metrics in every response, ensuring local credit ledger deductions match upstream billing down to the single token.
4. **Zero Data Retention Compliance**: OpenRouter's enterprise data privacy policy guarantees zero data persistence for requests flagged with zero-retention headers.

### Why Vercel AI SDK?

1. **Standardized Tool-Calling Protocol**: Vercel AI SDK's Core API provides first-class primitives for tool calling, structured outputs, and multi-step agentic loops (`streamText`, `tool(...)`) without writing boilerplate JSON schemas manually.
2. **Seamless UI Streaming**: In the React frontend, `@ai-sdk/react` (`useChat`) handles streaming text chunks, tool invocation lifecycle states (loading, executing, completed), abort controls, and optimistic UI updates natively.
3. **Typed RPC Bridge**: By implementing a lightweight custom transport for `useChat`, the UI bridges directly to Coachify's typed Tauri Rust backend, maintaining the single-RPC architecture rule while reaping all the UI benefits of the AI SDK.
