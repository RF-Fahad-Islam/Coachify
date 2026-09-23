import { useState, useEffect, useCallback } from "react";
import { rpc, type BatchProfitabilityItem, type BatchSummary } from "@/gen/rpc";
import { formatTaka } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  TrendingUp,
  TrendingDown,
  DollarSign,
  PieChart,
  Settings,
  RefreshCw,
  Loader2,
  Building,
  CheckCircle2,
} from "lucide-react";

export function BatchProfitabilityView() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [selectedBatchId, setSelectedBatchId] = useState<string>("");
  const [profitabilityData, setProfitabilityData] = useState<BatchProfitabilityItem[]>(
    []
  );

  // Edit Cost Config Modal State
  const [editingItem, setEditingItem] = useState<BatchProfitabilityItem | null>(null);
  const [modalOpen, setModalOpen] = useState(false);
  const [teacherCostTaka, setTeacherCostTaka] = useState("");
  const [teacherCostType, setTeacherCostType] = useState("flat");
  const [roomCostTaka, setRoomCostTaka] = useState("");
  const [overheadCostTaka, setOverheadCostTaka] = useState("");
  const [savingCost, setSavingCost] = useState(false);
  const [modalError, setModalError] = useState<string | null>(null);

  // Load batches list for filter
  useEffect(() => {
    void (async () => {
      try {
        const res = await rpc("batches.list", {});
        if (res.ok && res.data) {
          setBatches(res.data);
        }
      } catch (err: unknown) {
        console.error("Failed to load batches list", err);
      }
    })();
  }, []);

  const loadProfitability = useCallback(async (batchId: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await rpc("intelligence.get_batch_profitability", {
        batch_id: batchId || null,
        session: null,
      });

      if (res.ok && res.data) {
        setProfitabilityData(res.data);
      } else {
        setError(res.error?.message || "Failed to load batch profitability");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadProfitability(selectedBatchId);
  }, [selectedBatchId, loadProfitability]);

  const handleOpenEditModal = (item: BatchProfitabilityItem) => {
    setEditingItem(item);
    setTeacherCostTaka(Math.floor(item.teacher_monthly_cost_poisha / 100).toString());
    setTeacherCostType(item.teacher_cost_type || "flat");
    setRoomCostTaka(Math.floor(item.room_monthly_cost_poisha / 100).toString());
    setOverheadCostTaka(
      Math.floor(item.operational_overhead_cost_poisha / 100).toString()
    );
    setModalError(null);
    setModalOpen(true);
  };

  const handleSaveCostConfig = async () => {
    if (!editingItem) return;
    setSavingCost(true);
    setModalError(null);

    const teacherVal = parseFloat(teacherCostTaka) || 0;
    const roomVal = parseFloat(roomCostTaka) || 0;
    const overheadVal = parseFloat(overheadCostTaka) || 0;

    if (teacherVal < 0 || roomVal < 0 || overheadVal < 0) {
      setModalError("Costs cannot be negative");
      setSavingCost(false);
      return;
    }

    try {
      const res = await rpc("intelligence.update_batch_cost_config", {
        batch_id: editingItem.batch_id,
        teacher_monthly_cost: Math.round(teacherVal * 100),
        teacher_cost_type: teacherCostType,
        room_monthly_cost: Math.round(roomVal * 100),
        operational_overhead_cost: Math.round(overheadVal * 100),
      });

      if (res.ok) {
        setModalOpen(false);
        void loadProfitability(selectedBatchId);
      } else {
        setModalError(res.error?.message || "Failed to update cost configuration");
      }
    } catch (err: unknown) {
      setModalError(err instanceof Error ? err.message : String(err));
    } finally {
      setSavingCost(false);
    }
  };

  // Aggregate stats across displayed cohorts
  const totalProjectedRevenue = profitabilityData.reduce(
    (sum, b) => sum + b.projected_monthly_revenue_poisha,
    0
  );
  const totalCollectedRevenue = profitabilityData.reduce(
    (sum, b) => sum + b.net_collected_revenue_poisha,
    0
  );
  const totalCosts = profitabilityData.reduce((sum, b) => sum + b.total_cost_poisha, 0);
  const netMargin = totalCollectedRevenue - totalCosts;
  const aggregateMarginPct =
    totalCollectedRevenue > 0 ? Math.round((netMargin / totalCollectedRevenue) * 100) : 0;

  return (
    <div className="space-y-6">
      {/* Header & Filter Bar */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground">
            Batch Profitability & Unit Economics
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            P&amp;L by cohort: projected vs collected tuition, teacher compensations, room
            leases, overhead allocation, and margins.
          </p>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <select
            aria-label="Filter by Batch Cohort"
            value={selectedBatchId}
            onChange={(e) => setSelectedBatchId(e.target.value)}
            className="h-8 px-3 py-1 text-xs rounded-[var(--radius-md)] bg-card border border-border text-foreground font-medium focus:outline-hidden cursor-pointer"
          >
            <option value="">All Batches ({batches.length})</option>
            {batches.map((b) => (
              <option key={b.id} value={b.id}>
                {b.name}
              </option>
            ))}
          </select>

          <Button
            variant="ghost"
            size="sm"
            onClick={() => void loadProfitability(selectedBatchId)}
            disabled={loading}
            className="h-8 w-8 p-0"
            title="Refresh"
            aria-label="Refresh profitability data"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>

      {/* Aggregate Portfolio KPI Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className="border-border/80 bg-card shadow-2xs">
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-muted-foreground">
                Projected Tuition
              </span>
              <div className="text-2xl font-bold tracking-tight text-foreground">
                {formatTaka(totalProjectedRevenue)}
              </div>
              <span className="text-[11px] text-muted-foreground">
                Active enrolled billing
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-primary-subtle text-primary flex items-center justify-center">
              <DollarSign className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card className="border-border/80 bg-card shadow-2xs">
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-muted-foreground">
                Net Collections
              </span>
              <div className="text-2xl font-bold tracking-tight text-foreground">
                {formatTaka(totalCollectedRevenue)}
              </div>
              <span className="text-[11px] text-muted-foreground">
                Actually received in counter
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-emerald-500/10 text-emerald-600 flex items-center justify-center">
              <CheckCircle2 className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card className="border-border/80 bg-card shadow-2xs">
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-muted-foreground">
                Cohort Operating Costs
              </span>
              <div className="text-2xl font-bold tracking-tight text-foreground">
                {formatTaka(totalCosts)}
              </div>
              <span className="text-[11px] text-muted-foreground">
                Teacher + Room + Overhead
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-blue-500/10 text-blue-600 flex items-center justify-center">
              <Building className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card className="border-border/80 bg-card shadow-2xs">
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-muted-foreground">
                Portfolio Net Margin
              </span>
              <div
                className={`text-2xl font-bold tracking-tight ${
                  netMargin >= 0
                    ? "text-emerald-600 dark:text-emerald-400"
                    : "text-destructive"
                }`}
              >
                {formatTaka(netMargin)}
              </div>
              <span className="text-[11px] font-semibold font-mono text-muted-foreground">
                {aggregateMarginPct}% net margin
              </span>
            </div>
            <div
              className={`h-10 w-10 rounded-[var(--radius-md)] flex items-center justify-center ${
                netMargin >= 0
                  ? "bg-emerald-500/10 text-emerald-600"
                  : "bg-destructive/10 text-destructive"
              }`}
            >
              {netMargin >= 0 ? (
                <TrendingUp className="h-5 w-5" />
              ) : (
                <TrendingDown className="h-5 w-5" />
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {error && (
        <div className="p-4 rounded-[var(--radius-lg)] bg-destructive/10 border border-destructive/20 text-destructive text-xs">
          {error}
        </div>
      )}

      {/* Loading state */}
      {loading ? (
        <div className="py-20 flex flex-col items-center justify-center gap-3">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
          <span className="text-xs text-muted-foreground font-medium">
            Calculating cohort revenues, teacher costs, and profit margins...
          </span>
        </div>
      ) : profitabilityData.length === 0 ? (
        <Card className="border-dashed border-2 border-border">
          <CardContent className="py-16 text-center space-y-3">
            <div className="h-12 w-12 rounded-full bg-muted flex items-center justify-center mx-auto text-muted-foreground">
              <PieChart className="h-6 w-6" />
            </div>
            <h3 className="text-sm font-bold text-foreground">
              No cohort profitability data available
            </h3>
            <p className="text-xs text-muted-foreground max-w-sm mx-auto">
              Ensure academic batches and fee plans have been configured.
            </p>
          </CardContent>
        </Card>
      ) : (
        /* Batches Profitability Table */
        <Card className="border-border">
          <CardHeader className="pb-3 border-b border-border/70 flex flex-row items-center justify-between">
            <div>
              <CardTitle className="text-base flex items-center gap-2">
                <PieChart className="h-4 w-4 text-primary" />
                Batch Unit Economics &amp; Contribution Margins
              </CardTitle>
              <p className="text-xs text-muted-foreground mt-0.5">
                Financial performance breakdown for {profitabilityData.length} batches
              </p>
            </div>
          </CardHeader>
          <CardContent className="p-0">
            <div className="overflow-x-auto">
              <table className="w-full text-xs text-left border-collapse">
                <thead>
                  <tr className="border-b border-border bg-muted/40 font-semibold text-muted-foreground">
                    <th className="py-3 px-4">Batch &amp; Program</th>
                    <th className="py-3 px-3 text-right">Students</th>
                    <th className="py-3 px-3 text-right">Projected Revenue</th>
                    <th className="py-3 px-3 text-right">Net Collected</th>
                    <th className="py-3 px-3 text-right">Teacher Cost</th>
                    <th className="py-3 px-3 text-right">Room Cost</th>
                    <th className="py-3 px-3 text-right">Overhead</th>
                    <th className="py-3 px-3 text-right">Total Cost</th>
                    <th className="py-3 px-3 text-right">Net Margin</th>
                    <th className="py-3 px-3 text-center">Status</th>
                    <th className="py-3 px-4 text-right">Action</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border/60">
                  {profitabilityData.map((item) => (
                    <tr
                      key={item.batch_id}
                      className="hover:bg-muted/30 transition-colors"
                    >
                      <td className="py-3 px-4">
                        <span className="font-bold text-foreground block">
                          {item.batch_name}
                        </span>
                        <span className="text-[11px] text-muted-foreground">
                          {item.program_name}
                        </span>
                      </td>

                      <td className="py-3 px-3 text-right font-mono font-medium text-foreground">
                        {item.enrolled_students}
                      </td>

                      <td className="py-3 px-3 text-right font-mono text-muted-foreground">
                        {formatTaka(item.projected_monthly_revenue_poisha)}
                      </td>

                      <td className="py-3 px-3 text-right font-mono font-bold text-foreground">
                        {formatTaka(item.net_collected_revenue_poisha)}
                      </td>

                      <td className="py-3 px-3 text-right font-mono text-muted-foreground">
                        {formatTaka(item.teacher_monthly_cost_poisha)}
                        <span className="text-[10px] text-muted-foreground block">
                          ({item.teacher_cost_type})
                        </span>
                      </td>

                      <td className="py-3 px-3 text-right font-mono text-muted-foreground">
                        {formatTaka(item.room_monthly_cost_poisha)}
                      </td>

                      <td className="py-3 px-3 text-right font-mono text-muted-foreground">
                        {formatTaka(item.operational_overhead_cost_poisha)}
                      </td>

                      <td className="py-3 px-3 text-right font-mono font-semibold text-foreground">
                        {formatTaka(item.total_cost_poisha)}
                      </td>

                      <td className="py-3 px-3 text-right font-mono font-bold">
                        <span
                          className={
                            item.net_margin_poisha >= 0
                              ? "text-emerald-600 dark:text-emerald-400"
                              : "text-destructive"
                          }
                        >
                          {formatTaka(item.net_margin_poisha)}
                        </span>
                        <span className="text-[10px] text-muted-foreground block">
                          {item.margin_percentage}%
                        </span>
                      </td>

                      <td className="py-3 px-3 text-center">
                        {item.is_breakeven ? (
                          <Badge variant="paid" size="sm">
                            Breakeven
                          </Badge>
                        ) : (
                          <Badge variant="overdue" size="sm">
                            Deficit
                          </Badge>
                        )}
                      </td>

                      <td className="py-3 px-4 text-right">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleOpenEditModal(item)}
                          className="h-7 text-xs font-semibold text-primary hover:bg-primary-subtle gap-1"
                        >
                          <Settings className="h-3 w-3" />
                          Costs
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Edit Cost Config Modal */}
      <Dialog open={modalOpen} onOpenChange={setModalOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base font-bold flex items-center gap-2">
              <Settings className="h-4 w-4 text-primary" />
              Configure Batch Operating Costs
            </DialogTitle>
            {editingItem && (
              <p className="text-xs text-muted-foreground">
                Set monthly recurring expenses for{" "}
                <strong>{editingItem.batch_name}</strong> ({editingItem.program_name})
              </p>
            )}
          </DialogHeader>

          {modalError && (
            <div className="p-3 rounded bg-destructive/10 text-destructive text-xs">
              {modalError}
            </div>
          )}

          <div className="space-y-4 py-2 text-xs">
            {/* Teacher Cost */}
            <div className="space-y-1.5">
              <label className="font-semibold text-foreground flex items-center justify-between">
                <span>Teacher Monthly Compensation</span>
                <span className="text-[11px] text-muted-foreground">in BDT Taka</span>
              </label>
              <div className="flex gap-2">
                <input
                  type="number"
                  min="0"
                  value={teacherCostTaka}
                  onChange={(e) => setTeacherCostTaka(e.target.value)}
                  placeholder="e.g. 15000"
                  className="flex-1 px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground font-mono focus:outline-hidden"
                />
                <select
                  value={teacherCostType}
                  onChange={(e) => setTeacherCostType(e.target.value)}
                  className="px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground font-medium focus:outline-hidden"
                >
                  <option value="flat">Flat Monthly</option>
                  <option value="percentage">% of Tuition</option>
                </select>
              </div>
            </div>

            {/* Room Cost */}
            <div className="space-y-1.5">
              <label className="font-semibold text-foreground flex items-center justify-between">
                <span>Room / Classroom Lease Allocation</span>
                <span className="text-[11px] text-muted-foreground">in BDT Taka</span>
              </label>
              <input
                type="number"
                min="0"
                value={roomCostTaka}
                onChange={(e) => setRoomCostTaka(e.target.value)}
                placeholder="e.g. 5000"
                className="w-full px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground font-mono focus:outline-hidden"
              />
            </div>

            {/* Overhead Cost */}
            <div className="space-y-1.5">
              <label className="font-semibold text-foreground flex items-center justify-between">
                <span>Operational Overhead &amp; Utilities Allocation</span>
                <span className="text-[11px] text-muted-foreground">in BDT Taka</span>
              </label>
              <input
                type="number"
                min="0"
                value={overheadCostTaka}
                onChange={(e) => setOverheadCostTaka(e.target.value)}
                placeholder="e.g. 3000"
                className="w-full px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground font-mono focus:outline-hidden"
              />
            </div>
          </div>

          <DialogFooter className="gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setModalOpen(false)}
              disabled={savingCost}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSaveCostConfig}
              disabled={savingCost}
              className="gap-1.5"
            >
              {savingCost ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
              Save Cost Config
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
