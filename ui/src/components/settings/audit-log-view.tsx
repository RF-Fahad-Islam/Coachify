import { useCallback, useEffect, useState, useMemo } from "react";
import {
  rpc,
  type AuditLogItem,
  type ListAuditLogsPayload,
  type VerifyAuditChainResponse,
} from "@/gen/rpc";
import { useAuth } from "@/contexts/auth-context";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  ShieldCheck,
  ShieldAlert,
  RefreshCw,
  Eye,
  Filter,
  Copy,
  Check,
  History,
  Lock,
  ChevronLeft,
  ChevronRight,
  Database,
} from "lucide-react";

const PAGE_SIZE_OPTIONS = [10, 15, 25, 50];

function formatActionBadge(action: string) {
  if (action.includes("create") || action.includes("bootstrap")) {
    return (
      <Badge variant="paid" size="sm" className="font-mono text-[10px]">
        {action}
      </Badge>
    );
  }
  if (action.includes("update")) {
    return (
      <Badge variant="active" size="sm" className="font-mono text-[10px]">
        {action}
      </Badge>
    );
  }
  if (
    action.includes("delete") ||
    action.includes("failed") ||
    action.includes("lockout")
  ) {
    return (
      <Badge variant="overdue" size="sm" className="font-mono text-[10px]">
        {action}
      </Badge>
    );
  }
  if (
    action.includes("password") ||
    action.includes("login") ||
    action.includes("auth")
  ) {
    return (
      <Badge variant="pending" size="sm" className="font-mono text-[10px]">
        {action}
      </Badge>
    );
  }
  return (
    <Badge variant="secondary" size="sm" className="font-mono text-[10px]">
      {action}
    </Badge>
  );
}

function truncateHash(hash: string, lead = 8, trail = 6): string {
  if (!hash) return "—";
  if (hash.length <= lead + trail) return hash;
  return `${hash.slice(0, lead)}...${hash.slice(-trail)}`;
}

interface DiffRowProps {
  keyName: string;
  beforeVal: unknown;
  afterVal: unknown;
}

function DiffRow({ keyName, beforeVal, afterVal }: DiffRowProps) {
  const isAdded = beforeVal === undefined && afterVal !== undefined;
  const isRemoved = beforeVal !== undefined && afterVal === undefined;
  const isChanged =
    beforeVal !== undefined &&
    afterVal !== undefined &&
    JSON.stringify(beforeVal) !== JSON.stringify(afterVal);

  return (
    <tr
      className={`border-b border-border/40 text-xs font-mono transition-colors ${
        isAdded
          ? "bg-success-subtle/20"
          : isRemoved
            ? "bg-danger-subtle/20"
            : isChanged
              ? "bg-warning-subtle/20"
              : ""
      }`}
    >
      <td className="p-2 font-semibold text-foreground/80 w-1/4 align-top">{keyName}</td>
      <td className="p-2 w-3/8 align-top text-muted-foreground break-all">
        {beforeVal !== undefined ? (
          beforeVal === "[REDACTED]" ? (
            <span className="inline-flex items-center gap-1 text-danger font-semibold bg-danger-subtle/40 px-1.5 py-0.5 rounded text-[11px]">
              <Lock className="h-3 w-3" /> [REDACTED]
            </span>
          ) : (
            <span>{JSON.stringify(beforeVal, null, 2)}</span>
          )
        ) : (
          <span className="italic text-muted-foreground/50">—</span>
        )}
      </td>
      <td className="p-2 w-3/8 align-top text-foreground break-all">
        {afterVal !== undefined ? (
          afterVal === "[REDACTED]" ? (
            <span className="inline-flex items-center gap-1 text-danger font-semibold bg-danger-subtle/40 px-1.5 py-0.5 rounded text-[11px]">
              <Lock className="h-3 w-3" /> [REDACTED]
            </span>
          ) : (
            <span>{JSON.stringify(afterVal, null, 2)}</span>
          )
        ) : (
          <span className="italic text-muted-foreground/50">—</span>
        )}
      </td>
    </tr>
  );
}

export function AuditLogView() {
  const { sessionToken, hasPermission, isOwner } = useAuth();
  const canViewAudit = isOwner() || hasPermission("audit.view");

  const [items, setItems] = useState<AuditLogItem[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [headSeq, setHeadSeq] = useState<number | null>(null);
  const [headHash, setHeadHash] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Filters
  const [actionFilter, setActionFilter] = useState("");
  const [entityFilter, setEntityFilter] = useState("");
  const [entityIdFilter, setEntityIdFilter] = useState("");
  const [userIdFilter, setUserIdFilter] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");

  // Pagination
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState(15);

  // Verification state
  const [verifying, setVerifying] = useState(false);
  const [verificationResult, setVerificationResult] =
    useState<VerifyAuditChainResponse | null>(null);

  // Diff dialog state
  const [selectedItem, setSelectedItem] = useState<AuditLogItem | null>(null);
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  const fetchLogs = useCallback(async () => {
    if (!canViewAudit) return;
    setLoading(true);
    setError(null);
    try {
      const payload: ListAuditLogsPayload = {
        action: actionFilter.trim() || null,
        entity: entityFilter.trim() || null,
        entity_id: entityIdFilter.trim() || null,
        user_id: userIdFilter.trim() || null,
        date_from: dateFrom || null,
        date_to: dateTo || null,
        limit: pageSize,
        offset: page * pageSize,
      };

      const res = await rpc("audit.list", payload, { sessionToken });
      if (res.ok && res.data) {
        setItems(res.data.items);
        setTotalCount(res.data.total_count);
        setHeadSeq(res.data.head_seq ?? null);
        setHeadHash(res.data.head_hash ?? null);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [
    canViewAudit,
    actionFilter,
    entityFilter,
    entityIdFilter,
    userIdFilter,
    dateFrom,
    dateTo,
    pageSize,
    page,
    sessionToken,
  ]);

  const verifyChain = useCallback(async () => {
    if (!canViewAudit) return;
    setVerifying(true);
    try {
      const res = await rpc("audit.verify_chain", {}, { sessionToken });
      if (res.ok && res.data) {
        setVerificationResult(res.data);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setVerifying(false);
    }
  }, [canViewAudit, sessionToken]);

  useEffect(() => {
    if (canViewAudit) {
      void fetchLogs();
    }
  }, [canViewAudit, fetchLogs]);

  // Run chain verification on initial load
  useEffect(() => {
    if (canViewAudit) {
      void verifyChain();
    }
  }, [canViewAudit, verifyChain]);

  const handleResetFilters = () => {
    setActionFilter("");
    setEntityFilter("");
    setEntityIdFilter("");
    setUserIdFilter("");
    setDateFrom("");
    setDateTo("");
    setPage(0);
  };

  const copyToClipboard = (text: string, key: string) => {
    void navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey(null), 2000);
  };

  // Compute parsed JSON objects for selected diff item
  const diffData = useMemo(() => {
    if (!selectedItem) return null;
    let beforeObj: Record<string, unknown> = {};
    let afterObj: Record<string, unknown> = {};

    try {
      if (selectedItem.before_json) {
        beforeObj = JSON.parse(selectedItem.before_json) as Record<string, unknown>;
      }
    } catch {
      // Keep as empty
    }

    try {
      if (selectedItem.after_json) {
        afterObj = JSON.parse(selectedItem.after_json) as Record<string, unknown>;
      }
    } catch {
      // Keep as empty
    }

    const allKeys = Array.from(
      new Set([...Object.keys(beforeObj), ...Object.keys(afterObj)])
    ).sort();

    return { beforeObj, afterObj, allKeys };
  }, [selectedItem]);

  if (!canViewAudit) {
    return (
      <Card className="border-warning-border bg-warning-subtle/30">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-warning-text">
            <Lock className="h-5 w-5" />
            Audit Log Access Restricted
          </CardTitle>
          <CardDescription>
            You do not have the required <code>audit.view</code> permission to inspect the
            cryptographic audit trail or chain verification diagnostics. Please contact
            the Institute Owner.
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  const totalPages = Math.ceil(totalCount / pageSize);

  return (
    <div className="space-y-6">
      {/* Header and Diagnostics Banner */}
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground flex items-center gap-2.5">
            <History className="h-6 w-6 text-primary" />
            Audit Log & Cryptographic Chain
          </h1>
          <p className="text-xs text-muted-foreground mt-0.5">
            Append-only cryptographic SHA-256 hash chain recording every administrative
            mutation and financial event.
          </p>
        </div>

        {/* Verification Status Badge & Trigger */}
        <div className="flex items-center gap-3">
          {verificationResult ? (
            <div
              className={`flex items-center gap-2 px-3 py-1.5 rounded-[var(--radius)] border text-xs ${
                verificationResult.is_valid
                  ? "bg-success-subtle/50 text-success-text border-success-border font-medium"
                  : "bg-danger-subtle text-danger-text border-danger-border font-bold animate-pulse"
              }`}
            >
              {verificationResult.is_valid ? (
                <>
                  <ShieldCheck className="h-4 w-4 text-success shrink-0" />
                  <div className="flex flex-col">
                    <span>Chain Valid ({verificationResult.total_records} records)</span>
                    <span className="text-[10px] opacity-80 font-mono">
                      Head #{verificationResult.head_seq ?? 0}:{" "}
                      {truncateHash(verificationResult.head_hash ?? "", 6, 4)}
                    </span>
                  </div>
                </>
              ) : (
                <>
                  <ShieldAlert className="h-4 w-4 text-danger shrink-0" />
                  <div className="flex flex-col">
                    <span>CHAIN COMPROMISED at #{verificationResult.broken_seq}</span>
                    <span className="text-[10px]">
                      {verificationResult.error_message}
                    </span>
                  </div>
                </>
              )}
            </div>
          ) : headSeq !== null && headHash ? (
            <div className="flex items-center gap-2 px-3 py-1.5 rounded-[var(--radius)] border text-xs bg-muted/30 border-border text-muted-foreground font-mono">
              <span>
                Head #{headSeq}: {truncateHash(headHash, 6, 4)}
              </span>
            </div>
          ) : null}

          <Button
            size="sm"
            variant="outline"
            onClick={() => void verifyChain()}
            disabled={verifying}
            className="gap-2 text-xs"
          >
            <ShieldCheck
              className={`h-4 w-4 text-primary ${verifying ? "animate-spin" : ""}`}
            />
            {verifying ? "Verifying Chain..." : "Verify Chain Integrity"}
          </Button>

          <Button
            size="sm"
            variant="ghost"
            onClick={() => void fetchLogs()}
            disabled={loading}
            title="Refresh Audit Records"
            className="h-8 w-8 p-0"
          >
            <RefreshCw
              className={`h-4 w-4 text-muted-foreground ${loading ? "animate-spin" : ""}`}
            />
          </Button>
        </div>
      </div>

      {/* Filter Controls Card */}
      <Card className="p-4 bg-muted/20 border-border/70">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
            <Filter className="h-3.5 w-3.5" />
            Filter Audit Entries
          </div>
          {(actionFilter ||
            entityFilter ||
            entityIdFilter ||
            userIdFilter ||
            dateFrom ||
            dateTo) && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleResetFilters}
              className="text-[11px] h-7 px-2 text-muted-foreground hover:text-foreground"
            >
              Reset Filters
            </Button>
          )}
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
          <div>
            <label className="text-[11px] font-medium text-muted-foreground block mb-1">
              Action
            </label>
            <Input
              placeholder="e.g. users.create"
              value={actionFilter}
              onChange={(e) => {
                setActionFilter(e.target.value);
                setPage(0);
              }}
              className="h-8 text-xs"
            />
          </div>

          <div>
            <label className="text-[11px] font-medium text-muted-foreground block mb-1">
              Entity Type
            </label>
            <Input
              placeholder="user, role, payment..."
              value={entityFilter}
              onChange={(e) => {
                setEntityFilter(e.target.value);
                setPage(0);
              }}
              className="h-8 text-xs"
            />
          </div>

          <div>
            <label className="text-[11px] font-medium text-muted-foreground block mb-1">
              Entity ID
            </label>
            <Input
              placeholder="UUID or Entity ID"
              value={entityIdFilter}
              onChange={(e) => {
                setEntityIdFilter(e.target.value);
                setPage(0);
              }}
              className="h-8 text-xs"
            />
          </div>

          <div>
            <label className="text-[11px] font-medium text-muted-foreground block mb-1">
              Actor ID
            </label>
            <Input
              placeholder="User ID"
              value={userIdFilter}
              onChange={(e) => {
                setUserIdFilter(e.target.value);
                setPage(0);
              }}
              className="h-8 text-xs"
            />
          </div>

          <div>
            <label className="text-[11px] font-medium text-muted-foreground block mb-1">
              Date From
            </label>
            <Input
              type="date"
              value={dateFrom}
              onChange={(e) => {
                setDateFrom(e.target.value);
                setPage(0);
              }}
              className="h-8 text-xs"
            />
          </div>

          <div>
            <label className="text-[11px] font-medium text-muted-foreground block mb-1">
              Date To
            </label>
            <Input
              type="date"
              value={dateTo}
              onChange={(e) => {
                setDateTo(e.target.value);
                setPage(0);
              }}
              className="h-8 text-xs"
            />
          </div>
        </div>
      </Card>

      {/* Error Message */}
      {error && (
        <div className="p-3 text-xs bg-danger-subtle text-danger-text rounded-[var(--radius)] border border-danger-border">
          {error}
        </div>
      )}

      {/* Audit Log Table */}
      <Card>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-16">Seq</TableHead>
                <TableHead className="w-44">Time (UTC / Civil)</TableHead>
                <TableHead className="w-36">Actor</TableHead>
                <TableHead className="w-44">Action</TableHead>
                <TableHead className="w-36">Entity</TableHead>
                <TableHead className="w-44">SHA-256 Hash</TableHead>
                <TableHead className="text-right w-24">Diff</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {loading && items.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={7}
                    className="text-center py-12 text-xs text-muted-foreground"
                  >
                    <div className="flex flex-col items-center gap-2">
                      <RefreshCw className="h-5 w-5 animate-spin text-primary" />
                      <span>Loading cryptographic audit log...</span>
                    </div>
                  </TableCell>
                </TableRow>
              ) : items.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={7}
                    className="text-center py-12 text-xs text-muted-foreground"
                  >
                    <Database className="h-8 w-8 mx-auto mb-2 opacity-40" />
                    <span>No audit log records found matching the active filters.</span>
                  </TableCell>
                </TableRow>
              ) : (
                items.map((item) => (
                  <TableRow key={item.seq} className="hover:bg-muted/40 text-xs">
                    <TableCell className="font-mono font-bold text-foreground/80">
                      #{item.seq}
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-col">
                        <span className="font-medium text-foreground">
                          {new Date(item.at).toLocaleTimeString([], {
                            hour: "2-digit",
                            minute: "2-digit",
                            second: "2-digit",
                          })}
                        </span>
                        <span className="text-[10px] text-muted-foreground font-mono">
                          {item.business_date}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-col">
                        <span className="font-medium text-foreground truncate max-w-[130px]">
                          {item.user_display_name || "System"}
                        </span>
                        {item.device_name && (
                          <span className="text-[10px] text-muted-foreground truncate max-w-[130px]">
                            {item.device_name}
                          </span>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>{formatActionBadge(item.action)}</TableCell>
                    <TableCell>
                      <div className="flex flex-col">
                        <span className="font-semibold text-foreground">
                          {item.entity}
                        </span>
                        <span className="text-[10px] font-mono text-muted-foreground truncate max-w-[120px]">
                          {item.entity_id}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-1.5 font-mono text-[11px] text-muted-foreground">
                        <span title={item.hash}>{truncateHash(item.hash, 6, 4)}</span>
                        <button
                          type="button"
                          onClick={() => copyToClipboard(item.hash, `hash-${item.seq}`)}
                          className="hover:text-foreground transition-colors cursor-pointer"
                          title="Copy full SHA-256 hash"
                        >
                          {copiedKey === `hash-${item.seq}` ? (
                            <Check className="h-3 w-3 text-success" />
                          ) : (
                            <Copy className="h-3 w-3 opacity-60 hover:opacity-100" />
                          )}
                        </button>
                      </div>
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setSelectedItem(item)}
                        className="h-7 px-2 text-xs gap-1"
                      >
                        <Eye className="h-3 w-3" />
                        Diff
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>

          {/* Pagination Toolbar */}
          <div className="flex flex-col sm:flex-row items-center justify-between p-3 border-t border-border/70 gap-3 text-xs text-muted-foreground">
            <div className="flex items-center gap-2">
              <span>
                Showing{" "}
                <strong className="text-foreground">
                  {totalCount === 0 ? 0 : page * pageSize + 1}
                </strong>{" "}
                to{" "}
                <strong className="text-foreground">
                  {Math.min((page + 1) * pageSize, totalCount)}
                </strong>{" "}
                of <strong className="text-foreground">{totalCount}</strong> entries
              </span>

              <span className="hidden sm:inline-block text-border">|</span>

              <div className="flex items-center gap-1.5">
                <span>Per page:</span>
                <select
                  value={pageSize}
                  onChange={(e) => {
                    setPageSize(Number(e.target.value));
                    setPage(0);
                  }}
                  className="h-7 rounded border border-border bg-card px-1.5 text-xs text-foreground focus:outline-none"
                >
                  {PAGE_SIZE_OPTIONS.map((opt) => (
                    <option key={opt} value={opt}>
                      {opt}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            <div className="flex items-center gap-1.5">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPage((p) => Math.max(0, p - 1))}
                disabled={page === 0 || loading}
                className="h-7 px-2 text-xs gap-1"
              >
                <ChevronLeft className="h-3.5 w-3.5" />
                Previous
              </Button>
              <span className="text-xs px-2 font-medium text-foreground">
                Page {page + 1} of {Math.max(1, totalPages)}
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
                disabled={page >= totalPages - 1 || loading}
                className="h-7 px-2 text-xs gap-1"
              >
                Next
                <ChevronRight className="h-3.5 w-3.5" />
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Before / After Diff Inspector Dialog */}
      <Dialog
        open={Boolean(selectedItem)}
        onOpenChange={(open) => {
          if (!open) setSelectedItem(null);
        }}
      >
        <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <History className="h-5 w-5 text-primary" />
              Audit Entry #{selectedItem?.seq} — {selectedItem?.action}
            </DialogTitle>
            <DialogDescription className="text-xs">
              Cryptographic SHA-256 record verified against predecessor.
            </DialogDescription>
          </DialogHeader>

          {selectedItem && (
            <div className="space-y-4 overflow-y-auto flex-1 pr-1 text-xs">
              {/* Metadata Grid */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5 p-3 rounded-[var(--radius)] bg-muted/40 border border-border/70">
                <div>
                  <span className="text-[10px] text-muted-foreground block uppercase font-medium">
                    Civil Date
                  </span>
                  <span className="font-semibold text-foreground">
                    {selectedItem.business_date}
                  </span>
                </div>
                <div>
                  <span className="text-[10px] text-muted-foreground block uppercase font-medium">
                    Actor
                  </span>
                  <span className="font-semibold text-foreground truncate block">
                    {selectedItem.user_display_name || "System"}
                  </span>
                </div>
                <div>
                  <span className="text-[10px] text-muted-foreground block uppercase font-medium">
                    Entity
                  </span>
                  <span className="font-semibold text-foreground">
                    {selectedItem.entity}
                  </span>
                </div>
                <div>
                  <span className="text-[10px] text-muted-foreground block uppercase font-medium">
                    Entity ID
                  </span>
                  <span className="font-mono text-[11px] text-foreground truncate block">
                    {selectedItem.entity_id}
                  </span>
                </div>
              </div>

              {/* Cryptographic Linkage Box */}
              <div className="p-3 rounded-[var(--radius)] bg-muted/30 border border-border/80 space-y-2">
                <div>
                  <div className="flex items-center justify-between text-[11px] font-medium text-muted-foreground mb-0.5">
                    <span>Previous Record Hash (prev_hash)</span>
                    <button
                      type="button"
                      onClick={() => copyToClipboard(selectedItem.prev_hash, "prev_hash")}
                      className="hover:text-foreground text-[10px] flex items-center gap-1 cursor-pointer"
                    >
                      {copiedKey === "prev_hash" ? (
                        <Check className="h-3 w-3 text-success" />
                      ) : (
                        <Copy className="h-3 w-3" />
                      )}
                      Copy
                    </button>
                  </div>
                  <div className="p-1.5 rounded bg-background font-mono text-[10px] break-all border border-border/50 text-foreground">
                    {selectedItem.prev_hash}
                  </div>
                </div>

                <div>
                  <div className="flex items-center justify-between text-[11px] font-medium text-muted-foreground mb-0.5">
                    <span>This Record Canonical Hash (hash)</span>
                    <button
                      type="button"
                      onClick={() => copyToClipboard(selectedItem.hash, "hash")}
                      className="hover:text-foreground text-[10px] flex items-center gap-1 cursor-pointer"
                    >
                      {copiedKey === "hash" ? (
                        <Check className="h-3 w-3 text-success" />
                      ) : (
                        <Copy className="h-3 w-3" />
                      )}
                      Copy
                    </button>
                  </div>
                  <div className="p-1.5 rounded bg-background font-mono text-[10px] break-all border border-primary/40 text-primary font-semibold">
                    {selectedItem.hash}
                  </div>
                </div>
              </div>

              {/* State Comparison Table */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="font-semibold text-foreground text-xs">
                    Field State Transition
                  </span>
                  <span className="text-[10px] text-muted-foreground flex items-center gap-1">
                    <Lock className="h-3 w-3 text-warning" />
                    Sensitive keys cryptographically scrubbed
                  </span>
                </div>

                {diffData && diffData.allKeys.length > 0 ? (
                  <div className="border border-border/70 rounded-[var(--radius)] overflow-hidden">
                    <table className="w-full text-left border-collapse">
                      <thead className="bg-muted/60 text-[11px] font-semibold text-muted-foreground border-b border-border/70">
                        <tr>
                          <th className="p-2 w-1/4">Field</th>
                          <th className="p-2 w-3/8">Before Mutation</th>
                          <th className="p-2 w-3/8">After Mutation</th>
                        </tr>
                      </thead>
                      <tbody>
                        {diffData.allKeys.map((k) => (
                          <DiffRow
                            key={k}
                            keyName={k}
                            beforeVal={diffData.beforeObj[k]}
                            afterVal={diffData.afterObj[k]}
                          />
                        ))}
                      </tbody>
                    </table>
                  </div>
                ) : (
                  <div className="p-6 text-center text-xs text-muted-foreground border border-dashed border-border rounded-[var(--radius)]">
                    No before/after payload recorded for this event.
                  </div>
                )}
              </div>
            </div>
          )}

          <DialogFooter className="pt-2 border-t border-border">
            <Button variant="outline" size="sm" onClick={() => setSelectedItem(null)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
