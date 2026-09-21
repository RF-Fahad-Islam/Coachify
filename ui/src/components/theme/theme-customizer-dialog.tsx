import { Sun, Moon, Laptop, RotateCcw, Check, Sparkles } from "lucide-react";
import {
  useTheme,
  type Palette,
  type Mode,
  type FontFamily,
  type Density,
  type UiScale,
  type Radius,
} from "@/theme/theme-provider";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ThemeCustomizerDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface PaletteOption {
  id: Palette;
  name: string;
  color: string;
  subtle: string;
}

const PALETTES: PaletteOption[] = [
  { id: "orange", name: "Sunset Orange", color: "#ea580c", subtle: "#fff7ed" },
  { id: "blue", name: "Ocean Blue", color: "#2563eb", subtle: "#eff6ff" },
  { id: "emerald", name: "Emerald Growth", color: "#059669", subtle: "#ecfdf5" },
  { id: "violet", name: "Royal Violet", color: "#7c3aed", subtle: "#f5f3ff" },
  { id: "rose", name: "Crimson Rose", color: "#e11d48", subtle: "#fff1f2" },
  { id: "slate", name: "Monochrome Slate", color: "#334155", subtle: "#f8fafc" },
];

const FONTS: { id: FontFamily; name: string; sample: string }[] = [
  { id: "plus-jakarta", name: "Plus Jakarta Sans", sample: "EduManage Default" },
  { id: "inter", name: "Inter", sample: "Neutral & Clean" },
  { id: "outfit", name: "Outfit", sample: "Modern & Friendly" },
  { id: "system", name: "System Native", sample: "OS Default" },
];

const DENSITIES: { id: Density; name: string; desc: string }[] = [
  { id: "compact", name: "Compact", desc: "36px rows (Cashier & Mark Entry)" },
  { id: "comfortable", name: "Comfortable", desc: "44px rows (Standard Default)" },
  { id: "spacious", name: "Spacious", desc: "52px rows (Large Displays)" },
];

const SCALES: UiScale[] = [90, 100, 110, 125];

const RADII: { id: Radius; name: string; val: string }[] = [
  { id: "sharp", name: "Sharp", val: "4px" },
  { id: "rounded", name: "Rounded", val: "8px" },
  { id: "soft", name: "Soft", val: "14px" },
];

export function ThemeCustomizerDialog({
  open,
  onOpenChange,
}: ThemeCustomizerDialogProps) {
  const {
    palette,
    mode,
    fontFamily,
    density,
    uiScale,
    radius,
    setPalette,
    setMode,
    setFontFamily,
    setDensity,
    setUiScale,
    setRadius,
    resetDefaults,
  } = useTheme();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-center justify-between pr-6">
            <div className="flex items-center gap-2">
              <Sparkles className="h-5 w-5 text-primary" />
              <DialogTitle>Theme & Appearance Customizer</DialogTitle>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={resetDefaults}
              className="text-xs gap-1.5"
            >
              <RotateCcw className="h-3.5 w-3.5" />
              Reset
            </Button>
          </div>
          <DialogDescription>
            Customize your coaching management system appearance, colors, fonts, density,
            and scaling.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-2">
          {/* Section 1: Color Palette */}
          <div>
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block mb-2.5">
              Brand Accent Palette
            </label>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
              {PALETTES.map((p) => {
                const isSelected = palette === p.id;
                return (
                  <button
                    key={p.id}
                    type="button"
                    onClick={() => setPalette(p.id)}
                    className={cn(
                      "flex items-center gap-2.5 p-2.5 rounded-[var(--radius)] border text-left transition-all cursor-pointer",
                      isSelected
                        ? "border-primary bg-primary-subtle/50 ring-1 ring-primary"
                        : "border-border bg-card hover:bg-muted/60"
                    )}
                  >
                    <span
                      className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full shadow-xs text-white"
                      style={{ backgroundColor: p.color }}
                    >
                      {isSelected && <Check className="h-3 w-3 stroke-[3]" />}
                    </span>
                    <span className="text-xs font-medium text-foreground truncate">
                      {p.name}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Section 2: Mode */}
          <div>
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block mb-2.5">
              Appearance Mode
            </label>
            <div className="grid grid-cols-3 gap-2.5">
              {(
                [
                  { id: "light" as Mode, name: "Light", icon: Sun },
                  { id: "dark" as Mode, name: "Dark", icon: Moon },
                  { id: "system" as Mode, name: "System", icon: Laptop },
                ] as const
              ).map((m) => {
                const isSelected = mode === m.id;
                const Icon = m.icon;
                return (
                  <button
                    key={m.id}
                    type="button"
                    onClick={() => setMode(m.id)}
                    className={cn(
                      "flex items-center justify-center gap-2 p-2.5 rounded-[var(--radius)] border text-xs font-medium transition-all cursor-pointer",
                      isSelected
                        ? "border-primary bg-primary text-primary-foreground shadow-xs"
                        : "border-border bg-card text-foreground hover:bg-muted"
                    )}
                  >
                    <Icon className="h-4 w-4" />
                    {m.name}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Section 3: Typography */}
          <div>
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block mb-2.5">
              Typography Font Family
            </label>
            <div className="grid grid-cols-2 gap-2.5">
              {FONTS.map((f) => {
                const isSelected = fontFamily === f.id;
                return (
                  <button
                    key={f.id}
                    type="button"
                    onClick={() => setFontFamily(f.id)}
                    className={cn(
                      "flex flex-col p-2.5 rounded-[var(--radius)] border text-left transition-all cursor-pointer",
                      isSelected
                        ? "border-primary bg-primary-subtle/50 ring-1 ring-primary"
                        : "border-border bg-card hover:bg-muted/60"
                    )}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-semibold text-foreground">
                        {f.name}
                      </span>
                      {isSelected && <Check className="h-3.5 w-3.5 text-primary" />}
                    </div>
                    <span className="text-[11px] text-muted-foreground mt-0.5">
                      {f.sample}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Section 4: Information Density */}
          <div>
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block mb-2.5">
              Information Density & Spacing
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2.5">
              {DENSITIES.map((d) => {
                const isSelected = density === d.id;
                return (
                  <button
                    key={d.id}
                    type="button"
                    onClick={() => setDensity(d.id)}
                    className={cn(
                      "flex flex-col p-2.5 rounded-[var(--radius)] border text-left transition-all cursor-pointer",
                      isSelected
                        ? "border-primary bg-primary-subtle/50 ring-1 ring-primary"
                        : "border-border bg-card hover:bg-muted/60"
                    )}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-semibold text-foreground">
                        {d.name}
                      </span>
                      {isSelected && <Check className="h-3.5 w-3.5 text-primary" />}
                    </div>
                    <span className="text-[11px] text-muted-foreground mt-0.5">
                      {d.desc}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Section 5: UI Zoom Scale & Border Radius */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block mb-2">
                UI Scale / Zoom
              </label>
              <div className="flex items-center gap-1.5">
                {SCALES.map((s) => (
                  <button
                    key={s}
                    type="button"
                    onClick={() => setUiScale(s)}
                    className={cn(
                      "flex-1 py-1.5 text-xs font-medium rounded-[var(--radius-sm)] border transition-all cursor-pointer",
                      uiScale === s
                        ? "border-primary bg-primary text-primary-foreground"
                        : "border-border bg-card text-foreground hover:bg-muted"
                    )}
                  >
                    {s}%
                  </button>
                ))}
              </div>
            </div>

            <div>
              <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block mb-2">
                Border Radius
              </label>
              <div className="flex items-center gap-1.5">
                {RADII.map((r) => (
                  <button
                    key={r.id}
                    type="button"
                    onClick={() => setRadius(r.id)}
                    className={cn(
                      "flex-1 py-1.5 text-xs font-medium rounded-[var(--radius-sm)] border transition-all cursor-pointer",
                      radius === r.id
                        ? "border-primary bg-primary text-primary-foreground"
                        : "border-border bg-card text-foreground hover:bg-muted"
                    )}
                  >
                    {r.name}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
