import { useState, useEffect, useCallback } from "react";
import { rpc, type ContactHealthReport, type GuardianContactIssue } from "@/gen/rpc";
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
  Users,
  Phone,
  PhoneOff,
  PhoneMissed,
  RefreshCw,
  Check,
  X,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import { cn } from "@/lib/utils";

function IssueTypeBadge({ type }: { type: string }) {
  const variants: Record<string, { label: string; class: string }> = {
    missing: {
      label: "Missing",
      class: "bg-destructive/10 text-destructive border-destructive/20 border",
    },
    invalid_format: {
      label: "Invalid",
      class:
        "bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/20 border",
    },
    duplicate: {
      label: "Duplicate",
      class: "bg-blue-500/10 text-blue-700 dark:text-blue-300 border-blue-500/20 border",
    },
  };
  const v = variants[type] ?? {
    label: type,
    class: "bg-muted text-muted-foreground border",
  };
  return (
    <span className={cn("px-2 py-0.5 rounded-full text-[11px] font-semibold", v.class)}>
      {v.label}
    </span>
  );
}

interface EditingRow {
  guardian_id: string;
  new_phone: string;
}

export function GuardianContactHealthView() {
  const [report, setReport] = useState<ContactHealthReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<EditingRow | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveFeedback, setSaveFeedback] = useState<string | null>(null);
  const [expandedSiblings, setExpandedSiblings] = useState<Set<string>>(new Set());

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    const res = await rpc("integrity.get_guardian_contact_health", {
      limit: 100,
      offset: 0,
    });
    if (res.ok && res.data) {
      setReport(res.data);
    } else {
      setError(res.error?.message ?? "Failed to load contact health report");
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const handleSave = async (issue: GuardianContactIssue) => {
    if (!editing) return;
    setSaving(true);
    setSaveError(null);
    const res = await rpc("integrity.bulk_update_guardian_contacts", {
      updates: [{ guardian_id: issue.guardian_id, new_phone: editing.new_phone }],
    });
    setSaving(false);
    if (res.ok && res.data) {
      if (res.data.failed_count > 0) {
        setSaveError(res.data.errors.join("; "));
      } else {
        setSaveFeedback(`Updated ${res.data.updated_count} contact(s)`);
        setEditing(null);
        setTimeout(() => setSaveFeedback(null), 3000);
        void load();
      }
    } else {
      setSaveError(res.error?.message ?? "Failed to save");
    }
  };

  const toggleSibling = (phone: string) => {
    setExpandedSiblings((prev) => {
      const next = new Set(prev);
      if (next.has(phone)) next.delete(phone);
      else next.add(phone);
      return next;
    });
  };

  if (loading) {
    return (
      <div className="space-y-4">
        <div className="h-7 w-64 bg-muted animate-pulse rounded" />
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="h-20 bg-muted animate-pulse rounded-lg" />
          ))}
        </div>
        <div className="h-64 bg-muted animate-pulse rounded-lg" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
        <p className="font-semibold">Failed to load guardian contact health</p>
        <p className="text-xs mt-1 opacity-80">{error}</p>
        <Button
          variant="outline"
          size="sm"
          className="mt-3 gap-1.5 text-xs"
          onClick={() => void load()}
        >
          <RefreshCw className="h-3.5 w-3.5" /> Retry
        </Button>
      </div>
    );
  }

  if (!report) return null;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-bold tracking-tight text-foreground flex items-center gap-2">
            <Users className="h-5 w-5 text-primary" />
            Guardian Contact Health
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Review and fix missing, invalid, or duplicate guardian phone numbers.
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void load()}
          className="gap-1.5 text-xs"
        >
          <RefreshCw className="h-3.5 w-3.5" /> Refresh
        </Button>
      </div>

      {/* Feedback */}
      {saveFeedback && (
        <div className="rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-700 dark:text-emerald-300 px-4 py-2 text-xs flex items-center gap-2">
          <Check className="h-3.5 w-3.5" />
          {saveFeedback}
        </div>
      )}

      {/* KPI Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className="shadow-sm">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1">
              <Users className="h-3.5 w-3.5" /> Total Guardians
            </div>
            <div className="text-2xl font-bold text-foreground">
              {report.total_guardians.toLocaleString()}
            </div>
          </CardContent>
        </Card>
        <Card className="shadow-sm border-destructive/30">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-destructive text-xs mb-1">
              <PhoneMissed className="h-3.5 w-3.5" /> Missing Phone
            </div>
            <div className="text-2xl font-bold text-destructive">
              {report.missing_phone_count}
            </div>
          </CardContent>
        </Card>
        <Card className="shadow-sm border-amber-500/30">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-amber-600 dark:text-amber-400 text-xs mb-1">
              <PhoneOff className="h-3.5 w-3.5" /> Invalid Format
            </div>
            <div className="text-2xl font-bold text-amber-600 dark:text-amber-400">
              {report.invalid_phone_count}
            </div>
          </CardContent>
        </Card>
        <Card className="shadow-sm border-blue-500/30">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-blue-600 dark:text-blue-400 text-xs mb-1">
              <Phone className="h-3.5 w-3.5" /> Duplicates
            </div>
            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
              {report.duplicate_phone_count}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Issues Table */}
      <Card className="shadow-sm">
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-semibold">
            Contact Issues ({report.issues.length})
          </CardTitle>
          {report.issues.length === 0 && (
            <CardDescription className="text-xs text-emerald-600 dark:text-emerald-400">
              ✓ All guardian contacts are healthy — no issues detected.
            </CardDescription>
          )}
        </CardHeader>
        {report.issues.length > 0 && (
          <CardContent className="p-0">
            {saveError && (
              <div className="mx-4 mb-3 rounded bg-destructive/10 border border-destructive/20 px-3 py-2 text-xs text-destructive">
                {saveError}
              </div>
            )}
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50">
                    <TableHead className="text-xs">Guardian</TableHead>
                    <TableHead className="text-xs">Student</TableHead>
                    <TableHead className="text-xs">Relationship</TableHead>
                    <TableHead className="text-xs">Current Phone</TableHead>
                    <TableHead className="text-xs">Issue</TableHead>
                    <TableHead className="text-xs">Suggested Fix</TableHead>
                    <TableHead className="text-xs text-right">Action</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {report.issues.map((issue) => {
                    const isEditingThis = editing?.guardian_id === issue.guardian_id;
                    return (
                      <TableRow key={`${issue.guardian_id}-${issue.student_id}`}>
                        <TableCell className="text-xs font-medium">
                          {issue.guardian_name}
                        </TableCell>
                        <TableCell className="text-xs">
                          <div>{issue.student_name}</div>
                          <div className="text-[11px] text-muted-foreground">
                            {issue.student_code}
                          </div>
                        </TableCell>
                        <TableCell className="text-xs text-muted-foreground">
                          {issue.relationship}
                        </TableCell>
                        <TableCell className="text-xs">
                          {issue.current_phone ?? (
                            <span className="text-muted-foreground italic">None</span>
                          )}
                        </TableCell>
                        <TableCell>
                          <IssueTypeBadge type={issue.issue_type} />
                        </TableCell>
                        <TableCell className="text-xs text-muted-foreground">
                          {issue.suggested_fix ?? "—"}
                        </TableCell>
                        <TableCell className="text-right">
                          {isEditingThis ? (
                            <div className="flex items-center gap-1 justify-end">
                              <Input
                                value={editing.new_phone}
                                onChange={(e) =>
                                  setEditing({
                                    ...editing,
                                    new_phone: e.target.value,
                                  })
                                }
                                placeholder="+880..."
                                className="h-7 w-32 text-xs"
                                disabled={saving}
                                onKeyDown={(e) => {
                                  if (e.key === "Enter") void handleSave(issue);
                                  if (e.key === "Escape") setEditing(null);
                                }}
                                aria-label="New phone number"
                                autoFocus
                              />
                              <Button
                                size="icon"
                                variant="ghost"
                                className="h-7 w-7 text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-950"
                                onClick={() => void handleSave(issue)}
                                disabled={saving}
                                aria-label="Save phone number"
                              >
                                <Check className="h-3.5 w-3.5" />
                              </Button>
                              <Button
                                size="icon"
                                variant="ghost"
                                className="h-7 w-7 text-muted-foreground hover:bg-muted"
                                onClick={() => setEditing(null)}
                                disabled={saving}
                                aria-label="Cancel edit"
                              >
                                <X className="h-3.5 w-3.5" />
                              </Button>
                            </div>
                          ) : (
                            <Button
                              size="sm"
                              variant="outline"
                              className="h-7 text-xs gap-1"
                              onClick={() =>
                                setEditing({
                                  guardian_id: issue.guardian_id,
                                  new_phone: issue.current_phone ?? "",
                                })
                              }
                              aria-label={`Edit phone for ${issue.guardian_name}`}
                            >
                              <Phone className="h-3 w-3" /> Edit
                            </Button>
                          )}
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </div>
          </CardContent>
        )}
      </Card>

      {/* Sibling Groups */}
      {report.sibling_groups.length > 0 && (
        <Card className="shadow-sm">
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-semibold">
              Sibling Phone Groups ({report.sibling_groups.length})
            </CardTitle>
            <CardDescription className="text-xs">
              These guardians share a phone number with multiple students.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {report.sibling_groups.map((group) => {
              const expanded = expandedSiblings.has(group.normalized_phone);
              return (
                <div
                  key={group.normalized_phone}
                  className="rounded-lg border border-border overflow-hidden"
                >
                  <button
                    type="button"
                    className="w-full flex items-center justify-between px-4 py-2.5 bg-muted/40 hover:bg-muted/70 transition-colors text-left"
                    onClick={() => toggleSibling(group.normalized_phone)}
                    aria-expanded={expanded}
                  >
                    <div className="flex items-center gap-2 text-xs font-medium">
                      <Phone className="h-3.5 w-3.5 text-blue-500" />
                      <span>{group.guardian_name}</span>
                      <span className="text-muted-foreground">
                        {group.normalized_phone}
                      </span>
                      <Badge variant="secondary" className="text-[11px]">
                        {group.students.length} students
                      </Badge>
                    </div>
                    {expanded ? (
                      <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
                    ) : (
                      <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />
                    )}
                  </button>
                  {expanded && (
                    <div className="px-4 py-2 space-y-1">
                      {group.students.map((s) => (
                        <div
                          key={s.student_id}
                          className="flex items-center gap-3 text-xs text-muted-foreground"
                        >
                          <Users className="h-3 w-3 shrink-0" />
                          <span className="font-medium text-foreground">
                            {s.student_name}
                          </span>
                          <span>{s.student_code}</span>
                          <span className="text-[11px]">{s.relationship}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
