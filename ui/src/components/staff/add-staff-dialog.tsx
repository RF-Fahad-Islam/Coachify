import { useState } from "react";
import { rpc, type StaffDetail } from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { UserPlus, Loader2 } from "lucide-react";

export interface AddStaffDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (staff: StaffDetail) => void;
}

const DESIGNATIONS = [
  "Senior Lecturer (সিনিয়র প্রভাষক)",
  "Lecturer (প্রভাষক)",
  "Senior Teacher (সিনিয়র শিক্ষক)",
  "Assistant Teacher (সহকারী শিক্ষক)",
  "Subject Specialist (বিষয় বিশেষজ্ঞ)",
  "Academic Coordinator (একাডেমিক সমন্বয়কারী)",
  "Office Executive (অফিস এক্সিকিউটিভ)",
  "Accounts Officer (হিসাবরক্ষণ কর্মকর্তা)",
  "Cashier (ক্যাশিয়ার)",
];

const DEPARTMENTS = [
  "Science (বিজ্ঞান)",
  "Commerce / Business (ব্যবসায় শিক্ষা)",
  "Humanities (মানবিক)",
  "Mathematics (গণিত)",
  "English (ইংরেজি)",
  "Bangla (বাংলা)",
  "Administration & Accounts (প্রশাসন ও হিসাব)",
];

export function AddStaffDialog({ open, onOpenChange, onCreated }: AddStaffDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [nameBn, setNameBn] = useState("");
  const [phone, setPhone] = useState("");
  const [email, setEmail] = useState("");
  const [designation, setDesignation] = useState(DESIGNATIONS[0]);
  const [department, setDepartment] = useState(DEPARTMENTS[0]);
  const [joiningDate, setJoiningDate] = useState(
    () => new Date().toISOString().split("T")[0]
  );
  const [salaryType, setSalaryType] = useState("fixed_monthly");
  const [notes, setNotes] = useState("");

  const resetForm = () => {
    setName("");
    setNameBn("");
    setPhone("");
    setEmail("");
    setDesignation(DESIGNATIONS[0]);
    setDepartment(DEPARTMENTS[0]);
    setJoiningDate(new Date().toISOString().split("T")[0]);
    setSalaryType("fixed_monthly");
    setNotes("");
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setError("Staff full name is required");
      return;
    }
    if (!phone.trim()) {
      setError("Mobile phone number is required");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const res = await rpc("staff.create", {
        staff: {
          name: name.trim(),
          name_bn: nameBn.trim() || null,
          phone: phone.trim(),
          email: email.trim() || null,
          designation,
          department: department || null,
          joining_date: joiningDate,
          salary_type: salaryType,
          notes: notes.trim() || null,
        },
      });

      if (res.ok && res.data) {
        resetForm();
        onOpenChange(false);
        onCreated(res.data);
      } else {
        setError(res.error?.message || "Failed to create staff member");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Network error");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v) resetForm();
        onOpenChange(v);
      }}
    >
      <DialogContent className="max-w-xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg font-bold text-foreground">
            <UserPlus className="h-5 w-5 text-primary" />
            Add Staff Member / Teacher
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {error && (
            <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}

          {/* Name & Bangla Name */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Full Name (English) <span className="text-destructive">*</span>
              </label>
              <Input
                placeholder="e.g. Mohammad Rafiqul Islam"
                value={name}
                onChange={(e) => setName(e.target.value)}
                required
                disabled={loading}
                autoFocus
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Name (বাংলায়)
              </label>
              <Input
                placeholder="মো: রফিকুল ইসলাম"
                value={nameBn}
                onChange={(e) => setNameBn(e.target.value)}
                disabled={loading}
                className="font-bengali"
              />
            </div>
          </div>

          {/* Phone & Email */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Mobile Phone <span className="text-destructive">*</span>
              </label>
              <Input
                placeholder="01712345678"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                required
                disabled={loading}
              />
              <span className="text-[10px] text-muted-foreground block">
                Standard 11-digit Bangladeshi mobile number
              </span>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Email Address
              </label>
              <Input
                type="email"
                placeholder="rafiqul@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                disabled={loading}
              />
            </div>
          </div>

          {/* Designation & Department */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Designation <span className="text-destructive">*</span>
              </label>
              <select
                value={designation}
                onChange={(e) => setDesignation(e.target.value)}
                disabled={loading}
                aria-label="Designation"
                className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
              >
                {DESIGNATIONS.map((d) => (
                  <option key={d} value={d}>
                    {d}
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Department / Faculty
              </label>
              <select
                value={department}
                onChange={(e) => setDepartment(e.target.value)}
                disabled={loading}
                aria-label="Department"
                className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
              >
                {DEPARTMENTS.map((dept) => (
                  <option key={dept} value={dept}>
                    {dept}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Joining Date & Salary Structure */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Joining Date <span className="text-destructive">*</span>
              </label>
              <Input
                type="date"
                value={joiningDate}
                onChange={(e) => setJoiningDate(e.target.value)}
                required
                disabled={loading}
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Remuneration Type
              </label>
              <select
                value={salaryType}
                onChange={(e) => setSalaryType(e.target.value)}
                disabled={loading}
                aria-label="Remuneration Type"
                className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
              >
                <option value="fixed_monthly">Fixed Monthly (মাসিক বেতন)</option>
                <option value="per_class">Per Class / Lecture (ক্লাস ভিত্তিক)</option>
                <option value="hourly">Hourly (ঘন্টা ভিত্তিক)</option>
                <option value="revenue_share">Revenue Share (লভ্যাংশ)</option>
              </select>
            </div>
          </div>

          {/* Notes */}
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Remarks & Qualifications
            </label>
            <Input
              placeholder="e.g. M.Sc in Physics (DU), 8 years coaching experience"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              disabled={loading}
            />
          </div>

          <DialogFooter className="pt-3">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading} className="gap-1.5">
              {loading ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Saving...
                </>
              ) : (
                "Save Staff Record"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
