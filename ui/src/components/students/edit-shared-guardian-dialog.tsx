import { useEffect, useState } from "react";
import { rpc, type StudentSummary } from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AlertTriangle, Loader2, Users } from "lucide-react";

export interface EditSharedGuardianDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  guardianId: string | null;
  currentStudentId: string;
  currentStudentName: string;
  onSuccess: () => void;
}

export function EditSharedGuardianDialog({
  open,
  onOpenChange,
  guardianId,
  currentStudentId,
  currentStudentName,
  onSuccess,
}: EditSharedGuardianDialogProps) {
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [siblings, setSiblings] = useState<StudentSummary[]>([]);
  const [splitFromSiblings, setSplitFromSiblings] = useState(false);

  // Form fields
  const [name, setName] = useState("");
  const [nameBn, setNameBn] = useState("");
  const [relationship, setRelationship] = useState("father");
  const [phone, setPhone] = useState("");
  const [phone2, setPhone2] = useState("");
  const [whatsapp, setWhatsapp] = useState("");
  const [email, setEmail] = useState("");
  const [nid, setNid] = useState("");
  const [occupation, setOccupation] = useState("");
  const [presentAddress, setPresentAddress] = useState("");
  const [permanentAddress, setPermanentAddress] = useState("");
  const [notes, setNotes] = useState("");

  useEffect(() => {
    if (!open || !guardianId) return;

    let mounted = true;
    setLoading(true);
    setError(null);
    setSplitFromSiblings(false);

    rpc("guardians.get", { id: guardianId })
      .then((res) => {
        if (!mounted) return;
        if (res.ok && res.data) {
          const g = res.data.guardian;
          setSiblings(res.data.siblings || []);

          setName(g.name || "");
          setNameBn(g.name_bn || "");
          setRelationship(g.relationship || "father");
          setPhone(g.phone_norm || "");
          setPhone2(g.phone2_norm || "");
          setWhatsapp(g.whatsapp_number || "");
          setEmail(g.email || "");
          setNid(g.nid || "");
          setOccupation(g.occupation || "");
          setPresentAddress(g.present_address || "");
          setPermanentAddress(g.permanent_address || "");
          setNotes(g.notes || "");
        } else {
          setError(res.error?.message || "Failed to load guardian details");
        }
      })
      .catch((err: unknown) => {
        if (mounted) setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        if (mounted) setLoading(false);
      });

    return () => {
      mounted = false;
    };
  }, [open, guardianId]);

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!guardianId) return;

    if (!name.trim()) {
      setError("Guardian name is required");
      return;
    }
    if (!phone.trim()) {
      setError("Guardian phone is required");
      return;
    }

    setSaving(true);
    setError(null);

    try {
      const res = await rpc("guardians.update", {
        id: guardianId,
        guardian: {
          name: name.trim(),
          name_bn: nameBn.trim() || null,
          relationship: relationship.trim().toLowerCase(),
          phone: phone.trim(),
          phone2: phone2.trim() || null,
          whatsapp_number: whatsapp.trim() || null,
          email: email.trim() || null,
          nid: nid.trim() || null,
          occupation: occupation.trim() || null,
          monthly_income_poisha: null,
          present_address: presentAddress.trim() || null,
          permanent_address: permanentAddress.trim() || null,
          notes: notes.trim() || null,
        },
        split_from_siblings: splitFromSiblings,
        student_id: splitFromSiblings ? currentStudentId : null,
      });

      if (res.ok) {
        onSuccess();
        onOpenChange(false);
      } else {
        setError(res.error?.message || "Failed to update guardian");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  const sharedCount = siblings.length;
  const isShared = sharedCount > 1;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="text-xl font-bold flex items-center gap-2">
            <Users className="h-5 w-5 text-primary" />
            Edit Guardian Profile
          </DialogTitle>
        </DialogHeader>

        {loading ? (
          <div className="py-12 flex flex-col items-center justify-center gap-3">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
            <span className="text-sm text-muted-foreground">
              Loading guardian data...
            </span>
          </div>
        ) : (
          <form onSubmit={handleSave} className="space-y-4 pt-2">
            {error && (
              <div className="p-3 text-sm rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
                {error}
              </div>
            )}

            {isShared && (
              <div className="p-4 rounded-[var(--radius-md)] bg-warning-subtle border border-warning-border text-foreground space-y-3">
                <div className="flex items-start gap-2.5">
                  <AlertTriangle className="h-5 w-5 text-warning shrink-0 mt-0.5" />
                  <div>
                    <h4 className="text-sm font-semibold text-warning-text">
                      Shared Guardian Detected
                    </h4>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      This guardian record is linked to {sharedCount} students in the
                      directory:{" "}
                      <span className="font-semibold text-foreground">
                        {siblings.map((s) => `${s.name} (${s.student_code})`).join(", ")}
                      </span>
                      .
                    </p>
                  </div>
                </div>

                <div className="space-y-2 pt-1 border-t border-warning-border/60">
                  <label className="flex items-start gap-2.5 text-xs cursor-pointer select-none">
                    <input
                      type="radio"
                      name="split_option"
                      checked={!splitFromSiblings}
                      onChange={() => setSplitFromSiblings(false)}
                      className="mt-0.5 text-primary focus:ring-primary"
                    />
                    <div>
                      <span className="font-medium text-foreground">
                        Update guardian for all {sharedCount} siblings
                      </span>
                      <p className="text-muted-foreground text-[11px]">
                        Changes to phone, address, and info will apply across all linked
                        students.
                      </p>
                    </div>
                  </label>

                  <label className="flex items-start gap-2.5 text-xs cursor-pointer select-none">
                    <input
                      type="radio"
                      name="split_option"
                      checked={splitFromSiblings}
                      onChange={() => setSplitFromSiblings(true)}
                      className="mt-0.5 text-primary focus:ring-primary"
                    />
                    <div>
                      <span className="font-medium text-primary">
                        Detach and split for {currentStudentName} only
                      </span>
                      <p className="text-muted-foreground text-[11px]">
                        Clones this guardian into a new record for this student. The other
                        siblings will retain the previous information.
                      </p>
                    </div>
                  </label>
                </div>
              </div>
            )}

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <label className="text-xs font-semibold text-foreground">
                  Guardian Name (English) <span className="text-destructive">*</span>
                </label>
                <Input
                  className="mt-1"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. Rahim Uddin"
                  required
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Guardian Name (বাংলা)
                </label>
                <Input
                  className="mt-1 font-bengali"
                  value={nameBn}
                  onChange={(e) => setNameBn(e.target.value)}
                  placeholder="e.g. রহিম উদ্দিন"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Relationship <span className="text-destructive">*</span>
                </label>
                <select
                  className="mt-1 flex h-9 w-full rounded-[var(--radius-md)] border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                  value={relationship}
                  onChange={(e) => setRelationship(e.target.value)}
                  required
                >
                  <option value="father">Father (বাবা)</option>
                  <option value="mother">Mother (মা)</option>
                  <option value="brother">Brother (ভাই)</option>
                  <option value="sister">Sister (বোন)</option>
                  <option value="uncle">Uncle (চাচা / মামা)</option>
                  <option value="legal_guardian">Legal Guardian (আইনি অভিভাবক)</option>
                  <option value="other">Other (অন্যান্য)</option>
                </select>
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Mobile Phone Number <span className="text-destructive">*</span>
                </label>
                <Input
                  className="mt-1"
                  value={phone}
                  onChange={(e) => setPhone(e.target.value)}
                  placeholder="017XXXXXXXX"
                  required
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Alternative Phone
                </label>
                <Input
                  className="mt-1"
                  value={phone2}
                  onChange={(e) => setPhone2(e.target.value)}
                  placeholder="Optional second number"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  WhatsApp Number
                </label>
                <Input
                  className="mt-1"
                  value={whatsapp}
                  onChange={(e) => setWhatsapp(e.target.value)}
                  placeholder="Optional WhatsApp number"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Email Address
                </label>
                <Input
                  className="mt-1"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="name@example.com"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  NID Number
                </label>
                <Input
                  className="mt-1"
                  value={nid}
                  onChange={(e) => setNid(e.target.value)}
                  placeholder="National ID"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Occupation
                </label>
                <Input
                  className="mt-1"
                  value={occupation}
                  onChange={(e) => setOccupation(e.target.value)}
                  placeholder="e.g. Teacher, Business"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Present Address
                </label>
                <Input
                  className="mt-1"
                  value={presentAddress}
                  onChange={(e) => setPresentAddress(e.target.value)}
                  placeholder="Current address"
                />
              </div>

              <div className="md:col-span-2">
                <label className="text-xs font-semibold text-foreground">
                  Permanent Address
                </label>
                <Input
                  className="mt-1"
                  value={permanentAddress}
                  onChange={(e) => setPermanentAddress(e.target.value)}
                  placeholder="Permanent village / home address"
                />
              </div>

              <div className="md:col-span-2">
                <label className="text-xs font-semibold text-foreground">
                  Internal Notes
                </label>
                <Input
                  className="mt-1"
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                  placeholder="Any special remarks or instructions"
                />
              </div>
            </div>

            <DialogFooter className="pt-3 gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={saving}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={saving}>
                {saving ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Saving...
                  </>
                ) : splitFromSiblings ? (
                  "Save and Split Record"
                ) : (
                  "Save Changes"
                )}
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
