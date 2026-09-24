# 0035. Cloud AI gateway

> Feature 55 (Slice 10: AI foundation)

## Summary

This specification defines the Cloud AI Gateway and its desktop client integration. Coachify's hosted gateway acts as an authenticated, stateless proxy built with the **Vercel AI SDK Core (`ai`, `@ai-sdk/openai`)** that connects to **OpenRouter** (`https://openrouter.ai/api/v1`) as its universal upstream model provider.

Upstream provider credentials (OpenRouter API keys) reside exclusively on Coachify's cloud infrastructure and are never distributed within the client desktop application. In turn, the desktop app authenticates to the gateway using an institution credit token derived from its license or encrypted locally with Windows DPAPI. The gateway enforces rate limits, authenticates tenant licenses, verifies credit balances, tracks token consumption, and adheres to strict **zero data retention** policies.

For self-hosted and developer environments, the desktop client also supports a **Direct OpenRouter Mode**, allowing administrators to provide their own OpenRouter API key (`sk-or-v1-...`) stored via Windows DPAPI.

## Requirements

**User stories**:
* As an institution owner, I want the desktop app to connect seamlessly to cloud AI models without having to create third-party developer accounts, purchase foreign currency API credits, or configure complex API keys.
* As an owner wary of data privacy, I want cryptographic and contractual guarantees that the Cloud AI Gateway never persists our prompts, student records, or query results on its servers.
* As a technical administrator or self-hoster, I want the option to configure my own OpenRouter API key to query any model directly.
* As a system operator, I want the gateway to enforce clear rate limits and return immediate, friendly error messages if our internet connection is down or our credits are exhausted.
* As a software engineer running automated tests, I want a zero-network mock gateway adapter so that the test suite runs fast and reliably in CI without consuming real credits or requiring live network access.

**Acceptance criteria**:
* **AC-1**: Desktop client connects to the gateway via HTTPS POST requests to `/v1/chat/completions` using a standard bearer token header (`Authorization: Bearer <institution_token>`).
* **AC-2**: Zero raw AI provider keys (Anthropic/OpenAI/OpenRouter) are required on the desktop for standard users; provider credentials remain strictly on Coachify's cloud servers.
* **AC-3**: The gateway is implemented with Vercel AI SDK Core (`ai`, `@ai-sdk/openai`) configured with OpenRouter (`https://openrouter.ai/api/v1`), streaming responses and operating in strict zero-retention mode.
* **AC-4**: Supported upstream models include `anthropic/claude-3.5-sonnet` (primary reasoning engine) and `openai/gpt-4o-mini` (lightweight query router).
* **AC-5**: Direct OpenRouter mode: Settings UI permits entering an OpenRouter API key (`sk-or-v1-...`), encrypted with DPAPI, bypassing the hosted gateway for self-hosted instances.
* **AC-6**: Gateway validates the institution's credit token on every request, verifying license validity, enforcing rate limits (e.g. max 30 requests/minute per institution), and returning HTTP 402 with code `credits_exhausted` when credits reach zero.
* **AC-7**: Gateway responses return precise token metrics (`prompt_tokens`, `completion_tokens`, `credits_deducted`, `balance_remaining`) allowing the local desktop ledger to record identical usage figures.
* **AC-8**: Core engine includes a configurable mock gateway interface (`set_mock_gateway_response` / `MOCK_AI_GATEWAY`) for unit and integration testing without active internet connections.

## Inline rationale

Using **OpenRouter** as the upstream infrastructure provides unified access to all leading models (Claude 3.5 Sonnet, GPT-4o, LLaMA 3.3) under a single provider API, eliminating fragmented billing and multi-SDK bloat. Implementing the gateway with the **Vercel AI SDK** ensures standardized streaming protocols, native tool-calling abstractions, and seamless client-server compatibility.

## Protocol specification

### 1. Request envelope (Desktop $\to$ Gateway)

```http
POST /v1/chat/completions HTTP/1.1
Host: ai.coachify.cloud
Authorization: Bearer <institution_token>
Content-Type: application/json
X-Coachify-App-Version: 0.1.0
X-Coachify-Platform: windows-x86_64
X-Coachify-Request-Id: 019213ab-7c82-7000-8000-000000000001

{
  "model": "coachify-copilot-v1",
  "messages": [
    {
      "role": "user",
      "content": "How many students in Class 10 Physics have dues over 5000 taka?"
    }
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "query_students_with_dues",
        "description": "Query students with outstanding fee balances above a threshold",
        "parameters": {
          "type": "object",
          "properties": {
            "batch_id": { "type": "string" },
            "min_due_poisha": { "type": "integer" }
          }
        }
      }
    }
  ],
  "tool_choice": "auto",
  "temperature": 0.2
}
```

### 2. Response envelope (Gateway $\to$ Desktop)

```http
HTTP/1.1 200 OK
Content-Type: application/json
X-Coachify-Credits-Deducted: 15
X-Coachify-Balance-Remaining: 485

{
  "id": "chatcmpl-9x7a8b9c",
  "object": "chat.completion",
  "created": 1727100000,
  "model": "coachify-copilot-v1",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": null,
        "tool_calls": [
          {
            "id": "call_stu_dues_001",
            "type": "function",
            "function": {
              "name": "query_students_with_dues",
              "arguments": "{\"min_due_poisha\": 500000}"
            }
          }
        ]
      },
      "finish_reason": "tool_calls"
    }
  ],
  "usage": {
    "prompt_tokens": 128,
    "completion_tokens": 42,
    "total_tokens": 170,
    "credits_deducted": 15,
    "balance_remaining": 485
  }
}
```

### 3. Error response (Credits Exhausted)

```http
HTTP/1.1 402 Payment Required
Content-Type: application/json

{
  "error": {
    "code": "credits_exhausted",
    "message": "AI credit balance is exhausted. Please recharge your wallet to continue using Copilot.",
    "balance": 0,
    "top_up_url": "https://coachify.cloud/account/credits"
  }
}
```

### 4. Vercel AI SDK Gateway Route Reference (ai.coachify.cloud)

```typescript
import { createOpenAI } from '@ai-sdk/openai';
import { streamText } from 'ai';

// OpenRouter unified client configuration
const openrouter = createOpenAI({
  baseURL: 'https://openrouter.ai/api/v1',
  apiKey: process.env.OPENROUTER_API_KEY,
  headers: {
    'HTTP-Referer': 'https://coachify.app',
    'X-Title': 'Coachify Cloud AI Gateway',
  },
});

export async function POST(req: Request) {
  // 1. Authenticate institution token & verify credit balance
  const authHeader = req.headers.get('authorization');
  const institution = await verifyInstitutionCredits(authHeader);
  if (!institution || institution.balance <= 0) {
    return Response.json({
      error: { code: 'credits_exhausted', message: 'No credits remaining' }
    }, { status: 402 });
  }

  const { messages, tools, model = 'anthropic/claude-3.5-sonnet' } = await req.json();

  // 2. Stream completions via OpenRouter with zero data retention
  const result = streamText({
    model: openrouter(model),
    messages,
    tools,
    temperature: 0.2,
    onFinish: async ({ usage }) => {
      // 3. Deduct verified credits from institution wallet
      await deductCredits(institution.id, usage.totalTokens);
    },
  });

  return result.toDataStreamResponse();
}
```

## API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `ai.get_gateway_config` | query | _None_ | `AiGatewayConfig` (gateway_url, is_configured, is_mock, upstream_provider) | Owner, Admin | `unauthorized` |
| `ai.update_gateway_config` | command | `gateway_url: String`, `token: Option<String>`, `openrouter_key: Option<String>` | `AiGatewayConfig` | Owner, Admin | `unauthorized`, `invalid_url` |
| `ai.test_gateway_connection` | command | _None_ | `GatewayTestResult` (ok, latency_ms, provider_status) | Owner, Admin | `gateway_unreachable`, `gateway_auth_failed` |

## Build tasks

1. Implement `crates/core/src/ai/gateway.rs` supporting synchronous HTTPS dispatch targeting the Vercel AI SDK Gateway and OpenRouter, satisfies **AC-1**, **AC-3**, **AC-4**
2. Support Direct OpenRouter mode with DPAPI-encrypted API key (`sk-or-v1-...`) for self-hosted instances, satisfies **AC-5**
3. Integrate Windows DPAPI encryption in `ai::gateway` for storing and retrieving the institution and OpenRouter tokens, satisfies **AC-6**
4. Create test mock harness `MockAiGateway` simulating OpenRouter streaming envelopes and usage tokens, satisfies **AC-8**
5. Wire error mapping in `dispatcher.rs` for `gateway_unreachable`, `credits_exhausted`, `gateway_auth_failed`, and `rate_limited`, satisfies **AC-6**
