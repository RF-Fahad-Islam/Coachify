import { useState } from "react";
import { rpc, type ProgramDetail } from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { BookOpen, Loader2 } from "lucide-react";

export interface CreateProgramDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (program: ProgramDetail) => void;
}

export function CreateProgramDialog({
  open,
  onOpenChange,
  onCreated,
}: CreateProgramDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [classLevel, setClassLevel] = useState("");
  const [groupMajor, setGroupMajor] = useState("Science");
  const [description, setDescription] = useState("");

  const resetForm = () => {
    setName("");
    setCode("");
    setClassLevel("");
    setGroupMajor("Science");
    setDescription("");
    setError(null);
  };

  const handleNameChange = (val: string) => {
    setName(val);
    if (!code || code === name.toUpperCase().replace(/\s+/g, "-").slice(0, 10)) {
      // Auto-suggest code from name
      setCode(
        val
          .toUpperCase()
          .replace(/[^A-Z0-9]/g, "-")
          .slice(0, 12)
      );
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setError("Program name is required");
      return;
    }
    if (!code.trim()) {
      setError("Program code is required");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const res = await rpc("programs.create", {
        program: {
          name: name.trim(),
          code: code.trim().toUpperCase(),
          class_level: classLevel.trim() || null,
          group_major: groupMajor.trim() || null,
          description: description.trim() || null,
        },
      });

      if (res.ok && res.data) {
        resetForm();
        onOpenChange(false);
        onCreated(res.data);
      } else {
        setError(res.error?.message || "Failed to create program");
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
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg font-bold text-foreground">
            <BookOpen className="h-5 w-5 text-primary" />
            Add Academic Program
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {error && (
            <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Program Name <span className="text-destructive">*</span>
            </label>
            <Input
              placeholder="e.g. HSC 2026 Science Regular"
              value={name}
              onChange={(e) => handleNameChange(e.target.value)}
              required
              disabled={loading}
              autoFocus
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Program Code <span className="text-destructive">*</span>
              </label>
              <Input
                placeholder="HSC-SCI-26"
                value={code}
                onChange={(e) => setCode(e.target.value.toUpperCase())}
                required
                disabled={loading}
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Class / Grade Level
              </label>
              <Input
                placeholder="Class 11 / HSC"
                value={classLevel}
                onChange={(e) => setClassLevel(e.target.value)}
                disabled={loading}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Group / Stream
              </label>
              <select
                value={groupMajor}
                onChange={(e) => setGroupMajor(e.target.value)}
                disabled={loading}
                aria-label="Group / Stream"
                className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
              >
                <option value="Science">Science (বিজ্ঞান)</option>
                <option value="Commerce">Commerce / Business (ব্যবসায় শিক্ষা)</option>
                <option value="Humanities">Humanities / Arts (মানবিক)</option>
                <option value="General">General / All (সাধারণ)</option>
              </select>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">Description</label>
              <Input
                placeholder="Optional notes"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                disabled={loading}
              />
            </div>
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
                  Creating...
                </>
              ) : (
                "Create Program"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
