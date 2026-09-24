# 0035. AI copilot shell

> Feature 54 (Slice 10: AI foundation)

## Summary

This specification defines the AI Copilot Shell, providing a natural language interface for coaching center owners, administrators, and staff. The copilot opens as a slide-over panel from any screen in the application shell (triggered by `Ctrl+J` / `Cmd+J` or a dedicated top bar button).

The copilot answers questions about students, batches, attendance, fee collections, dues, and exams using the center's local data. Crucially, the copilot operates strictly through **controlled, read-only tools**; it never possesses direct SQL query execution capabilities, cannot mutate database records, and logs every query and tool execution for complete institutional accountability.

## Requirements

**User stories**:
* As a coaching center owner, I want to ask questions in natural language like *"How many students in Physics Batch A have dues over 5000 taka?"* so that I can make operational decisions without manually filtering reports.
* As an administrator, I want to know who had the lowest attendance in Class 10 Math this month so our counselors can follow up with guardians.
* As an auditor, I want absolute assurance that the AI copilot cannot modify student records, alter financial numbers, or run raw SQL statements against our SQLite database.
* As an operator, I want previous conversation turns to be preserved during my session with one-click clear options to reset context, but kept strictly private from other staff who share this PC.
* As a Bengali-speaking user, I want the copilot to understand and respond in natural Bangla when I ask in Bangla, with proper Bangladeshi financial conventions (Taka, poisha).

**Acceptance criteria**:
* **AC-1**: App shell provides a responsive slide-over Copilot Drawer toggled via `Ctrl+J` / `Cmd+J` or top bar Sparkles button, retaining conversation state across navigation.
* **AC-2**: Command `ai.ask_copilot` accepts user prompts, coordinates with the Cloud AI Gateway and local controlled tools, and returns streaming or buffered natural language responses with source attribution and protection metadata badges.
* **AC-3**: Bounded multi-hop execution: The copilot engine executes up to 3 sequential tool invocations per turn to resolve cross-domain inquiries (e.g. dues + attendance), with hard timeout and cycle guards.
* **AC-4**: The copilot queries local data solely through 6 read-only Rust tool implementations:
  1. `query_students_with_dues`: filters enrolled students with unpaid balances above a threshold.
  2. `get_batch_summary`: returns enrollment counts, schedules, capacity, and assigned teachers.
  3. `get_attendance_stats`: computes attendance percentages, absence counts, and session streaks.
  4. `get_today_operational_summary`: summarizes today's classes, collection totals, and attendance completion.
  5. `search_students`: finds students by name, code, or phone (subject to privacy rules).
  6. `get_exam_results_summary`: calculates batch GPA averages, pass rates, and mark distributions.
* **AC-5**: Zero direct SQL access: The LLM prompt and gateway payload explicitly constrain interaction to the declared tool schema. Any attempts to generate SQL or invoke non-existent mutating commands are rejected with typed validation errors.
* **AC-6**: Defense-in-depth RBAC: Financial tools are omitted from the tool schema sent to the gateway for non-financial roles (Teachers/Receptionists), and Rust tool handlers enforce permission checks defensively.
* **AC-7**: Auto-mirroring bilingual behavior: The copilot automatically detects and mirrors the language of the prompt (Bangla or English), using standard Bangladeshi numerical terms (Taka, poisha, Lakhs).
* **AC-8**: Conversation turns are strictly isolated per logged-in user in `ai_copilot_conversations` and `ai_copilot_messages`, supporting multi-turn context (limited to the last 10 messages) while preventing cross-shift history leakage.
* **AC-9**: Command `ai.clear_conversation` purges active conversation messages for the current user while preserving permanent audit records in `ai_audit_logs`.

## Inline rationale

Restricting the AI assistant to explicit, typed Rust tools completely eliminates SQL injection, unauthorized schema discovery, and accidental data mutation risks. Bounded multi-hop execution allows compound analytical questions across domains without risk of infinite loops, while user-isolated conversation threads guarantee privacy on shared workstation environments.

## Controlled tool contracts

```rust
// 1. Query students with dues
pub struct QueryStudentsWithDuesArgs {
    pub batch_id: Option<String>,
    pub min_due_poisha: Option<i64>,
    pub limit: Option<u32>,
}

// 2. Get batch summary
pub struct GetBatchSummaryArgs {
    pub batch_name_or_id: String,
}

// 3. Get attendance stats
pub struct GetAttendanceStatsArgs {
    pub batch_id: Option<String>,
    pub days: Option<u32>,
    pub min_attendance_pct: Option<f64>,
}

// 4. Get today operational summary
pub struct GetTodayOperationalSummaryArgs {
    pub target_date: Option<String>,
}

// 5. Search students
pub struct SearchStudentsArgs {
    pub query: String,
    pub limit: Option<u32>,
}

// 6. Get exam results summary
pub struct GetExamResultsSummaryArgs {
    pub exam_id_or_title: String,
    pub batch_id: Option<String>,
}
```

### React & Vercel AI SDK UI Integration (`CopilotDrawer`)

The Copilot Drawer utilizes `@ai-sdk/react` (`useChat`) connected to Coachify's typed Rust RPC bridge:

```tsx
import { useChat } from '@ai-sdk/react';
import { rpc } from '@/gen/rpc';

export function CopilotDrawer({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const { messages, input, handleInputChange, handleSubmit, isLoading, reload, stop } = useChat({
    // Custom RPC transport bridging Vercel AI SDK with Tauri Rust backend
    async fetch(_url, options) {
      const payload = JSON.parse(options?.body as string);
      const userPrompt = payload.messages[payload.messages.length - 1].content;
      
      const res = await rpc('ai.ask_copilot', {
        prompt: userPrompt,
        conversation_id: activeConversationId,
      });

      if (!res.ok) {
        throw new Error(res.error?.message || 'AI request failed');
      }

      // Convert RPC response into Vercel AI SDK data stream
      return new Response(res.data.content);
    },
  });

  return (
    <aside className={cn("fixed right-0 top-0 h-full w-[420px] bg-card border-l z-50 transition-transform", isOpen ? "translate-x-0" : "translate-x-full")}>
      {/* Header with Balance Badge & Language Indicator */}
      <CopilotHeader onClear={handleClear} onClose={onClose} />
      {/* Message Stream with Tool Badges & Protection Shield Badge */}
      <CopilotMessages messages={messages} isLoading={isLoading} />
      {/* Input Form with Auto-Resize & Stop Button */}
      <CopilotInput input={input} onChange={handleInputChange} onSubmit={handleSubmit} onStop={stop} isLoading={isLoading} />
    </aside>
  );
}
```

## API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `ai.ask_copilot` | command | `conversation_id: Option<String>`, `prompt: String` | `CopilotResponse` (conversation_id, message_id, content, tools_used, credits_deducted, balance_remaining, protection_metadata) | Any authenticated | `credits_exhausted`, `gateway_unreachable`, `gateway_auth_failed`, `rate_limited` |
| `ai.get_conversation` | query | `conversation_id: String` | `CopilotConversationDetails` (id, title, messages) | Any authenticated | `not_found` |
| `ai.clear_conversation` | command | `conversation_id: String` | `SuccessResponse` | Any authenticated | `not_found` |

## Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| `ask_copilot` | Natural language answer | Synthesized by Cloud AI Gateway from prompt and tool outputs |
| `ask_copilot` | Student dues metrics | Computed via `query_students_with_dues` from `ledger_charges` and `payments` |
| `ask_copilot` | Batch schedule & capacity | Queried via `get_batch_summary` from `batches` and `schedules` |
| `ask_copilot` | Attendance percentages | Aggregated via `get_attendance_stats` from `attendance_records` |
| `ask_copilot` | Today's operational stats | Derived via `get_today_operational_summary` from `batches`, `payments`, `attendance_sessions` |
| `ask_copilot` | Tool execution indicators | Invocations captured during tool loop execution |
| `ask_copilot` | Shield protection metadata | Count of names and numbers masked by `PrivacyRedactionEngine` |

## Build tasks

1. Define 6 read-only tool schemas and implementations in `crates/core/src/ai/tools.rs` with defensive limits (max 50 rows per tool invocation), satisfies **AC-3**, **AC-4**
2. Implement conversation turn tracking in `crates/core/src/ai/copilot.rs` storing messages in `ai_copilot_conversations` and `ai_copilot_messages` per local user, satisfies **AC-8**, **AC-9**
3. Implement core tool loop: Prompt $\to$ Regex Scrubber $\to$ Redaction $\to$ Gateway $\to$ Local Tool Execution $\to$ Gateway Synthesis $\to$ Local De-anonymization, satisfies **AC-2**, **AC-3**
4. Install `@ai-sdk/react` and `ai` in `ui/package.json` and build `CopilotDrawer` component with `useChat` custom transport, shortcut listener (`Ctrl+J`), tool progress indicators, and protection shield badges, satisfies **AC-1**, **AC-2**
