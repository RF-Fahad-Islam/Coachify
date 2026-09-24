import { useState, useEffect, useRef, useCallback } from "react";
import {
  Sparkles,
  X,
  Send,
  Trash2,
  ShieldCheck,
  ShieldAlert,
  Coins,
  Bot,
  User,
  Wrench,
  Copy,
  Check,
  Loader2,
  AlertCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { rpc } from "@/gen/rpc";
import type {
  CopilotMessageItem,
  CopilotResponse,
  AiWalletBalance,
  ProtectionMetadata,
} from "@/gen/rpc";
import { cn } from "@/lib/utils";

interface CopilotDrawerProps {
  open: boolean;
  onClose: () => void;
  onOpenSettings?: () => void;
}

export function CopilotDrawer({ open, onClose, onOpenSettings }: CopilotDrawerProps) {
  const [messages, setMessages] = useState<CopilotMessageItem[]>([]);
  const [conversationId, setConversationId] = useState<string | null>(null);
  const [inputPrompt, setInputPrompt] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [wallet, setWallet] = useState<AiWalletBalance | null>(null);
  const [lastProtection, setLastProtection] = useState<ProtectionMetadata | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Fetch initial wallet balance
  const fetchWallet = useCallback(async () => {
    try {
      const res = await rpc("ai.get_wallet_balance", {});
      if (res.ok && res.data) {
        setWallet(res.data);
      }
    } catch {
      // Ignore background error
    }
  }, []);

  useEffect(() => {
    if (open) {
      void fetchWallet();
      setTimeout(() => textareaRef.current?.focus(), 150);
    }
  }, [open, fetchWallet]);

  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isLoading]);

  // Handle ESC key to close
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && open) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  // Send message handler
  const handleSendMessage = async (promptToSend?: string) => {
    const text = (promptToSend ?? inputPrompt).trim();
    if (!text || isLoading) return;

    setErrorMsg(null);
    setInputPrompt("");

    // Optimistic user message
    const tempUserMsg: CopilotMessageItem = {
      id: `temp-${Date.now()}`,
      role: "user",
      content: text,
      credits_deducted: 0,
      created_at: new Date().toISOString(),
    };
    setMessages((prev) => [...prev, tempUserMsg]);
    setIsLoading(true);

    try {
      const res = await rpc("ai.ask_copilot", {
        conversation_id: conversationId,
        prompt: text,
      });

      if (res.ok && res.data) {
        const resp: CopilotResponse = res.data;
        setConversationId(resp.conversation_id);
        setLastProtection(resp.protection_metadata);

        // Assistant response message
        const asstMsg: CopilotMessageItem = {
          id: resp.message_id,
          role: "assistant",
          content: resp.content,
          tool_calls_json:
            resp.tools_used.length > 0 ? JSON.stringify(resp.tools_used) : null,
          credits_deducted: resp.credits_deducted,
          created_at: new Date().toISOString(),
        };

        setMessages((prev) => [...prev, asstMsg]);
        if (wallet) {
          setWallet({
            ...wallet,
            balance_credits: resp.balance_remaining,
          });
        }
      } else {
        const code = res.error?.code ?? "unknown_error";
        const message =
          res.error?.message ?? "An error occurred while calling AI Copilot.";
        if (code === "credits_exhausted") {
          setErrorMsg(
            "AI credits exhausted. Please recharge your credit wallet in Settings to continue."
          );
        } else {
          setErrorMsg(message);
        }
      }
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleClearChat = async () => {
    if (conversationId) {
      try {
        await rpc("ai.clear_conversation", { conversation_id: conversationId });
      } catch {
        // Ignore
      }
    }
    setMessages([]);
    setConversationId(null);
    setErrorMsg(null);
    setLastProtection(null);
  };

  const handleCopy = (id: string, text: string) => {
    void navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const suggestedPrompts = [
    "Which students have unpaid dues above ৳5,000?",
    "Give me today's operational summary.",
    "Show attendance performance across active batches.",
    "Find student Nafis Fuad and check status.",
  ];

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex justify-end">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 backdrop-blur-xs transition-opacity duration-200"
        onClick={onClose}
        aria-hidden="true"
      />

      {/* Slide-over Drawer Panel */}
      <aside className="relative z-50 w-full sm:w-[480px] md:w-[520px] flex flex-col justify-between border-l border-border bg-card text-card-foreground select-none h-full shadow-2xl animate-in slide-in-from-right duration-200">
        {/* Drawer Header */}
        <div className="h-16 px-4 sm:px-6 border-b border-border/80 flex items-center justify-between bg-card/90 backdrop-blur-xs shrink-0">
          <div className="flex items-center gap-2.5">
            <div className="flex h-9 w-9 items-center justify-center rounded-[var(--radius-md)] bg-primary text-primary-foreground shadow-xs">
              <Sparkles className="h-4.5 w-4.5" />
            </div>
            <div className="flex flex-col">
              <div className="flex items-center gap-2">
                <span className="font-bold text-sm text-foreground leading-tight">
                  Coachify Copilot
                </span>
                <span className="text-[10px] uppercase font-bold tracking-wider px-1.5 py-0.5 rounded-full bg-primary/10 text-primary border border-primary/20">
                  AI
                </span>
              </div>
              <span className="text-[11px] text-muted-foreground">
                Local Data · Controlled Tools · Zero Retention
              </span>
            </div>
          </div>

          <div className="flex items-center gap-1.5">
            {/* Wallet Balance Indicator */}
            {wallet && (
              <div
                title="Current credit balance"
                className="flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold bg-muted/60 border border-border text-foreground"
              >
                <Coins className="h-3.5 w-3.5 text-amber-500" />
                <span>{wallet.balance_credits}</span>
              </div>
            )}

            {/* Clear Chat Button */}
            {messages.length > 0 && (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => void handleClearChat()}
                title="Clear Conversation"
                className="h-8 w-8 text-muted-foreground hover:text-destructive hover:bg-destructive/10 cursor-pointer"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            )}

            {/* Close Button */}
            <Button
              variant="ghost"
              size="icon"
              onClick={onClose}
              title="Close Copilot (Esc)"
              className="h-8 w-8 text-muted-foreground hover:text-foreground cursor-pointer"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {/* Protection Banner */}
        <div className="px-4 py-2 bg-muted/40 border-b border-border text-xs flex items-center justify-between text-muted-foreground shrink-0">
          <div className="flex items-center gap-1.5">
            {lastProtection?.shield_active !== false ? (
              <>
                <ShieldCheck className="h-3.5 w-3.5 text-emerald-500" />
                <span className="font-medium text-foreground">Shield Active:</span>
                <span>PII Redacted & Re-hydrated</span>
              </>
            ) : (
              <>
                <ShieldAlert className="h-3.5 w-3.5 text-amber-500" />
                <span>Privacy Settings Relaxed</span>
              </>
            )}
          </div>

          {onOpenSettings && (
            <button
              type="button"
              onClick={() => {
                onClose();
                onOpenSettings();
              }}
              className="text-[11px] text-primary hover:underline cursor-pointer font-medium"
            >
              Privacy Settings
            </button>
          )}
        </div>

        {/* Conversation Stream */}
        <div className="flex-1 p-4 overflow-y-auto space-y-4">
          {messages.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-center p-6 space-y-4 text-muted-foreground">
              <div className="h-12 w-12 rounded-full bg-primary/10 text-primary flex items-center justify-center">
                <Bot className="h-6 w-6" />
              </div>
              <div>
                <h3 className="text-sm font-semibold text-foreground">
                  Ask Coachify Institutional Copilot
                </h3>
                <p className="text-xs mt-1 max-w-xs text-muted-foreground/80 leading-relaxed">
                  Query student dues, batch schedules, attendance rates, and operational
                  summaries in English or Bengali. Data stays local in SQLite.
                </p>
              </div>

              <div className="w-full space-y-2 pt-2 text-left">
                <span className="text-[11px] font-semibold text-foreground/80 uppercase tracking-wider block px-1">
                  Suggested Questions:
                </span>
                {suggestedPrompts.map((prompt) => (
                  <button
                    key={prompt}
                    type="button"
                    onClick={() => void handleSendMessage(prompt)}
                    className="w-full p-2.5 text-left text-xs rounded-lg border border-border bg-card hover:bg-muted/60 transition-colors text-foreground cursor-pointer flex items-center justify-between group"
                  >
                    <span>{prompt}</span>
                    <Sparkles className="h-3.5 w-3.5 text-primary opacity-0 group-hover:opacity-100 transition-opacity" />
                  </button>
                ))}
              </div>
            </div>
          ) : (
            messages.map((msg) => {
              const isUser = msg.role === "user";
              let tools: string[] = [];
              if (msg.tool_calls_json) {
                try {
                  tools = JSON.parse(msg.tool_calls_json);
                } catch {
                  tools = [];
                }
              }

              return (
                <div
                  key={msg.id}
                  className={cn(
                    "flex gap-3 text-xs",
                    isUser ? "justify-end" : "justify-start"
                  )}
                >
                  {!isUser && (
                    <div className="h-7 w-7 rounded-full bg-primary/15 text-primary flex items-center justify-center shrink-0 mt-0.5">
                      <Sparkles className="h-3.5 w-3.5" />
                    </div>
                  )}

                  <div
                    className={cn(
                      "flex flex-col space-y-1.5 max-w-[85%]",
                      isUser && "items-end"
                    )}
                  >
                    {/* Tool Badges */}
                    {!isUser && tools.length > 0 && (
                      <div className="flex flex-wrap gap-1 mb-1">
                        {tools.map((t) => (
                          <span
                            key={t}
                            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-mono bg-muted border border-border text-muted-foreground"
                          >
                            <Wrench className="h-2.5 w-2.5 text-primary" />
                            {t}
                          </span>
                        ))}
                      </div>
                    )}

                    {/* Message Bubble */}
                    <div
                      className={cn(
                        "p-3 rounded-2xl leading-relaxed whitespace-pre-wrap select-text",
                        isUser
                          ? "bg-primary text-primary-foreground rounded-br-xs"
                          : "bg-muted/70 border border-border text-foreground rounded-tl-xs shadow-2xs"
                      )}
                    >
                      {msg.content}
                    </div>

                    {/* Metadata: Credits & Copy */}
                    <div className="flex items-center gap-2 text-[10px] text-muted-foreground/70 px-1">
                      {!isUser && msg.credits_deducted > 0 && (
                        <span>-{msg.credits_deducted} credit</span>
                      )}
                      {!isUser && (
                        <button
                          type="button"
                          onClick={() => handleCopy(msg.id, msg.content)}
                          className="hover:text-foreground cursor-pointer flex items-center gap-1"
                        >
                          {copiedId === msg.id ? (
                            <>
                              <Check className="h-2.5 w-2.5 text-emerald-500" />
                              <span className="text-emerald-500">Copied</span>
                            </>
                          ) : (
                            <>
                              <Copy className="h-2.5 w-2.5" />
                              <span>Copy</span>
                            </>
                          )}
                        </button>
                      )}
                    </div>
                  </div>

                  {isUser && (
                    <div className="h-7 w-7 rounded-full bg-muted border border-border text-muted-foreground flex items-center justify-center shrink-0 mt-0.5">
                      <User className="h-3.5 w-3.5" />
                    </div>
                  )}
                </div>
              );
            })
          )}

          {/* Thinking / Tool Execution Loading State */}
          {isLoading && (
            <div className="flex items-center gap-3 text-xs text-muted-foreground">
              <div className="h-7 w-7 rounded-full bg-primary/10 text-primary flex items-center justify-center shrink-0 animate-pulse">
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              </div>
              <div className="p-3 rounded-2xl rounded-tl-xs bg-muted/60 border border-border flex items-center gap-2">
                <span>Executing local queries & thinking...</span>
              </div>
            </div>
          )}

          {/* Error Banner */}
          {errorMsg && (
            <div className="p-3 rounded-xl bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-start gap-2.5">
              <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
              <div className="flex-1">
                <div className="font-semibold">Query Failed</div>
                <div className="mt-0.5 text-destructive/90">{errorMsg}</div>
                {errorMsg.includes("credits") && onOpenSettings && (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => {
                      onClose();
                      onOpenSettings();
                    }}
                    className="mt-2 h-7 text-xs border-destructive/40 hover:bg-destructive/15 text-destructive"
                  >
                    Open Wallet Settings
                  </Button>
                )}
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* Input Bar */}
        <div className="p-3 sm:p-4 border-t border-border/80 bg-card shrink-0">
          <form
            onSubmit={(e) => {
              e.preventDefault();
              void handleSendMessage();
            }}
            className="relative"
          >
            <textarea
              ref={textareaRef}
              rows={2}
              value={inputPrompt}
              onChange={(e) => setInputPrompt(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  void handleSendMessage();
                }
              }}
              placeholder="Ask about students, dues, batches... (Enter to send)"
              className="w-full text-xs p-3 pr-12 rounded-[var(--radius)] border border-border bg-muted/40 placeholder:text-muted-foreground/70 focus:bg-card focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary transition-all resize-none"
            />

            <Button
              type="submit"
              size="icon"
              disabled={!inputPrompt.trim() || isLoading}
              className="absolute right-2.5 bottom-3.5 h-8 w-8 cursor-pointer disabled:opacity-40"
              aria-label="Send query to Copilot"
            >
              {isLoading ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Send className="h-4 w-4" />
              )}
            </Button>
          </form>

          <div className="flex items-center justify-between text-[11px] text-muted-foreground/70 mt-2 px-1">
            <span>Press Enter to send · Shift+Enter for new line</span>
            <kbd className="hidden sm:inline-block text-[10px] font-semibold bg-muted px-1.5 py-0.5 rounded border border-border">
              Ctrl+J
            </kbd>
          </div>
        </div>
      </aside>
    </div>
  );
}
