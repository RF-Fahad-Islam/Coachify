import { useState, useEffect } from "react";
import { rpc, type ExpenseCategoryItem } from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Receipt, Loader2, Plus } from "lucide-react";

export interface ExpenseEntryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function ExpenseEntryDialog({
  open,
  onOpenChange,
  onSuccess,
}: ExpenseEntryDialogProps) {
  const [categories, setCategories] = useState<ExpenseCategoryItem[]>([]);
  const [loadingCats, setLoadingCats] = useState(false);

  const [categoryId, setCategoryId] = useState("");
  const [newCatName, setNewCatName] = useState("");
  const [creatingCat, setCreatingCat] = useState(false);
  const [showNewCatInput, setShowNewCatInput] = useState(false);

  const [amountTaka, setAmountTaka] = useState("");
  const [method, setMethod] = useState("cash");
  const [payee, setPayee] = useState("");
  const [reference, setReference] = useState("");
  const [businessDate, setBusinessDate] = useState(() =>
    new Date().toISOString().slice(0, 10)
  );

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      void (async () => {
        setLoadingCats(true);
        try {
          const res = await rpc("expense_categories.list", {});
          if (res.ok && res.data) {
            setCategories(res.data);
            if (res.data.length > 0 && !categoryId) {
              setCategoryId(res.data[0].id);
            }
          }
        } catch (err: unknown) {
          console.error("Failed to load expense categories", err);
        } finally {
          setLoadingCats(false);
        }
      })();
    }
  }, [open, categoryId]);

  const handleCreateCategory = async () => {
    if (!newCatName.trim()) return;
    setCreatingCat(true);
    try {
      const res = await rpc("expense_categories.create", {
        category: { name: newCatName.trim() },
      });
      if (res.ok && res.data) {
        const newCat = res.data;
        setCategories((prev) => [...prev, newCat]);
        setCategoryId(newCat.id);
        setNewCatName("");
        setShowNewCatInput(false);
      } else {
        setError(res.error?.message || "Failed to create category");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setCreatingCat(false);
    }
  };

  const handleSubmit = async () => {
    const numTaka = parseFloat(amountTaka);
    if (isNaN(numTaka) || numTaka <= 0) {
      setError("Please enter a valid expense amount");
      return;
    }

    if (!categoryId) {
      setError("Please select an expense category");
      return;
    }

    if (!payee.trim()) {
      setError("Please enter the recipient or payee name");
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      const res = await rpc("expenses.create", {
        expense: {
          category_id: categoryId,
          amount_poisha: Math.round(numTaka * 100),
          method,
          payee: payee.trim(),
          reference: reference.trim() || null,
          business_date: businessDate,
          attachment_vault_id: null,
        },
      });

      if (res.ok) {
        setAmountTaka("");
        setPayee("");
        setReference("");
        onOpenChange(false);
        onSuccess();
      } else {
        setError(res.error?.message || "Failed to record expense");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-foreground">
            <Receipt className="h-5 w-5 text-primary" />
            Record Operating Expense
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2 text-xs">
          {/* Category selection */}
          <div className="space-y-1.5">
            <div className="flex justify-between items-center">
              <label htmlFor="expense-cat" className="font-medium text-foreground">
                Category <span className="text-destructive">*</span>
              </label>
              <button
                type="button"
                onClick={() => setShowNewCatInput(!showNewCatInput)}
                className="text-[11px] text-primary hover:underline font-medium cursor-pointer flex items-center gap-0.5"
              >
                <Plus className="h-3 w-3" />
                {showNewCatInput ? "Select Existing" : "Add New Category"}
              </button>
            </div>

            {showNewCatInput ? (
              <div className="flex gap-2">
                <Input
                  placeholder="e.g. Lab Equipment, Software License"
                  value={newCatName}
                  onChange={(e) => setNewCatName(e.target.value)}
                  className="text-xs h-9"
                />
                <Button
                  size="sm"
                  onClick={() => void handleCreateCategory()}
                  disabled={creatingCat || !newCatName.trim()}
                  className="h-9 px-3 text-xs"
                >
                  {creatingCat ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    "Save"
                  )}
                </Button>
              </div>
            ) : (
              <select
                id="expense-cat"
                value={categoryId}
                onChange={(e) => setCategoryId(e.target.value)}
                disabled={loadingCats}
                className="w-full h-9 px-3 rounded-[var(--radius-md)] border border-input bg-background text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
              >
                {categories.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                  </option>
                ))}
              </select>
            )}
          </div>

          {/* Amount and Method */}
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label htmlFor="expense-amount" className="font-medium text-foreground">
                Amount (৳) <span className="text-destructive">*</span>
              </label>
              <Input
                id="expense-amount"
                type="number"
                step="any"
                min="1"
                placeholder="e.g. 350"
                value={amountTaka}
                onChange={(e) => setAmountTaka(e.target.value)}
                className="text-xs font-mono font-medium"
                autoFocus
              />
            </div>

            <div className="space-y-1.5">
              <label htmlFor="expense-method" className="font-medium text-foreground">
                Disbursement Method
              </label>
              <select
                id="expense-method"
                value={method}
                onChange={(e) => setMethod(e.target.value)}
                className="w-full h-9 px-3 rounded-[var(--radius-md)] border border-input bg-background text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
              >
                <option value="cash">Cash (Drawer)</option>
                <option value="bkash">bKash</option>
                <option value="nagad">Nagad</option>
                <option value="bank">Bank Transfer</option>
                <option value="other">Other</option>
              </select>
            </div>
          </div>

          {/* Payee and Reference */}
          <div className="space-y-1.5">
            <label htmlFor="expense-payee" className="font-medium text-foreground">
              Payee / Vendor <span className="text-destructive">*</span>
            </label>
            <Input
              id="expense-payee"
              placeholder="e.g. Bismillah Stationery, Electrician Rahim"
              value={payee}
              onChange={(e) => setPayee(e.target.value)}
              className="text-xs"
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label htmlFor="expense-ref" className="font-medium text-foreground">
                Voucher / Bill No (Optional)
              </label>
              <Input
                id="expense-ref"
                placeholder="e.g. MEMO-4029"
                value={reference}
                onChange={(e) => setReference(e.target.value)}
                className="text-xs"
              />
            </div>

            <div className="space-y-1.5">
              <label htmlFor="expense-date" className="font-medium text-foreground">
                Business Date
              </label>
              <Input
                id="expense-date"
                type="date"
                value={businessDate}
                onChange={(e) => setBusinessDate(e.target.value)}
                className="text-xs"
              />
            </div>
          </div>

          {error && (
            <div className="p-2.5 rounded-[var(--radius-sm)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => onOpenChange(false)}
            disabled={submitting}
          >
            Cancel
          </Button>
          <Button
            type="button"
            size="sm"
            onClick={() => void handleSubmit()}
            disabled={submitting || !amountTaka || !payee.trim()}
            className="gap-1.5"
          >
            {submitting ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
            Post Expense
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
