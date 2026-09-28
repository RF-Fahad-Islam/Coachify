import { useState } from "react";
import {
  Building2,
  Save,
  CheckCircle2,
  RotateCcw,
  Palette,
  Sun,
  Moon,
  Laptop,
} from "lucide-react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  useTheme,
  type Palette as ThemePalette,
  type Mode,
  type Density,
  type Radius,
  type FontFamily,
} from "@/theme/theme-provider";
import { cn } from "@/lib/utils";

export interface InstituteProfile {
  name: string;
  code: string;
  tagline: string;
  phone: string;
  email: string;
  address: string;
  currency: string;
  academicSession: string;
  receiptHeaderNote: string;
  receiptFooterNote: string;
}

const DEFAULT_PROFILE: InstituteProfile = {
  name: "Apex Academy Coaching Center",
  code: "AAC-DHK",
  tagline: "Empowering Academic Excellence & Future Leaders",
  phone: "+880 1711-432109",
  email: "contact@apexacademy.edu.bd",
  address: "House 42, Road 11, Dhanmondi, Dhaka-1209",
  currency: "৳",
  academicSession: "2026",
  receiptHeaderNote: "Thank you for partnering in your child's academic journey.",
  receiptFooterNote:
    "Official computer generated voucher. Subject to institute terms & conditions.",
};

const STORAGE_KEY = "coachify_institute_profile";

const PALETTE_OPTIONS: {
  id: ThemePalette;
  name: string;
  color: string;
  subtle: string;
}[] = [
  { id: "orange", name: "Sunset Orange", color: "#ea580c", subtle: "#fff7ed" },
  { id: "blue", name: "Ocean Blue", color: "#2563eb", subtle: "#eff6ff" },
  { id: "emerald", name: "Emerald Growth", color: "#059669", subtle: "#ecfdf5" },
  { id: "violet", name: "Royal Violet", color: "#7c3aed", subtle: "#f5f3ff" },
  { id: "rose", name: "Crimson Rose", color: "#e11d48", subtle: "#fff1f2" },
  { id: "slate", name: "Monochrome Slate", color: "#334155", subtle: "#f8fafc" },
];

export function GeneralSettingsCard() {
  const [profile, setProfile] = useState<InstituteProfile>(() => {
    if (typeof window === "undefined") return DEFAULT_PROFILE;
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) return { ...DEFAULT_PROFILE, ...JSON.parse(stored) };
    } catch {
      // Fallback
    }
    return DEFAULT_PROFILE;
  });

  const [saving, setSaving] = useState(false);
  const [savedSuccess, setSavedSuccess] = useState(false);

  const {
    palette,
    mode,
    fontFamily,
    density,
    radius,
    setPalette,
    setMode,
    setFontFamily,
    setDensity,
    setRadius,
    resetDefaults: resetThemeDefaults,
  } = useTheme();

  const handleProfileChange = (key: keyof InstituteProfile, value: string) => {
    setProfile((prev) => ({ ...prev, [key]: value }));
  };

  const handleSaveAll = () => {
    setSaving(true);
    setSavedSuccess(false);

    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(profile));
      if (typeof window !== "undefined") {
        window.dispatchEvent(
          new CustomEvent("coachify:institute-profile-changed", { detail: profile })
        );
      }
      setSavedSuccess(true);
      setTimeout(() => setSavedSuccess(false), 4000);
    } catch (e) {
      console.error("Failed to save institute profile to storage", e);
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setProfile(DEFAULT_PROFILE);
    localStorage.removeItem(STORAGE_KEY);
    resetThemeDefaults();
  };

  return (
    <div className="space-y-6">
      {/* 1. Institute Profile & Branding */}
      <Card className="border-border/80 shadow-sm">
        <CardHeader className="flex flex-row items-center justify-between pb-3 border-b border-border/60">
          <div>
            <CardTitle className="flex items-center gap-2 text-base sm:text-lg">
              <Building2 className="h-5 w-5 text-primary" />
              <span>Institute Profile & Operational Details</span>
            </CardTitle>
            <p className="text-xs sm:text-sm text-muted-foreground mt-0.5 leading-relaxed">
              Official institute identity printed on receipts, admit cards, due
              statements, and SMS signatures.
            </p>
          </div>
          <Badge variant="outline" size="sm" className="hidden sm:inline-flex">
            Module 13 Configuration
          </Badge>
        </CardHeader>

        <CardContent className="pt-6 space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Institution Legal / Trade Name
              </label>
              <Input
                value={profile.name}
                onChange={(e) => handleProfileChange("name", e.target.value)}
                placeholder="Apex Academy Coaching Center"
                className="text-xs sm:text-sm font-medium"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Branch / Campus Code
              </label>
              <Input
                value={profile.code}
                onChange={(e) => handleProfileChange("code", e.target.value)}
                placeholder="AAC-DHK"
                className="text-xs sm:text-sm font-mono font-medium"
              />
            </div>

            <div className="space-y-1.5 md:col-span-2">
              <label className="text-xs font-semibold text-foreground">
                Brand Tagline / Motto
              </label>
              <Input
                value={profile.tagline}
                onChange={(e) => handleProfileChange("tagline", e.target.value)}
                placeholder="Empowering Academic Excellence & Future Leaders"
                className="text-xs sm:text-sm"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Official Helpline / Cashier Phone
              </label>
              <Input
                value={profile.phone}
                onChange={(e) => handleProfileChange("phone", e.target.value)}
                placeholder="+880 1711-432109"
                className="text-xs sm:text-sm font-mono font-medium"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Official Email Address
              </label>
              <Input
                value={profile.email}
                onChange={(e) => handleProfileChange("email", e.target.value)}
                placeholder="contact@apexacademy.edu.bd"
                className="text-xs sm:text-sm"
              />
            </div>

            <div className="space-y-1.5 md:col-span-2">
              <label className="text-xs font-semibold text-foreground">
                Physical Campus Address
              </label>
              <Input
                value={profile.address}
                onChange={(e) => handleProfileChange("address", e.target.value)}
                placeholder="House 42, Road 11, Dhanmondi, Dhaka-1209"
                className="text-xs sm:text-sm"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Primary Currency Symbol
              </label>
              <Input
                value={profile.currency}
                onChange={(e) => handleProfileChange("currency", e.target.value)}
                placeholder="৳"
                className="text-xs sm:text-sm font-bold w-32"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Current Academic Session / Year
              </label>
              <Input
                value={profile.academicSession}
                onChange={(e) => handleProfileChange("academicSession", e.target.value)}
                placeholder="2026"
                className="text-xs sm:text-sm font-semibold w-40"
              />
            </div>

            <div className="space-y-1.5 md:col-span-2">
              <label className="text-xs font-semibold text-foreground">
                Printed Receipt Footer Note
              </label>
              <Input
                value={profile.receiptFooterNote}
                onChange={(e) => handleProfileChange("receiptFooterNote", e.target.value)}
                placeholder="Official computer generated voucher. Subject to institute terms."
                className="text-xs sm:text-sm"
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 2. Visual Theme & Ergonomics Customizer */}
      <Card className="border-border/80 shadow-sm">
        <CardHeader className="flex flex-row items-center justify-between pb-3 border-b border-border/60">
          <div>
            <CardTitle className="flex items-center gap-2 text-base sm:text-lg">
              <Palette className="h-5 w-5 text-primary" />
              <span>Appearance, Density & Typography Engine</span>
            </CardTitle>
            <p className="text-xs sm:text-sm text-muted-foreground mt-0.5 leading-relaxed">
              Personalize palette themes, plush fatty corner radii, display density, and
              offline typography.
            </p>
          </div>
          <Badge variant="active" size="sm">
            Live Preview
          </Badge>
        </CardHeader>

        <CardContent className="pt-6 space-y-6">
          {/* Color Palette */}
          <div>
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block mb-2.5">
              Brand Accent Palette
            </label>
            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
              {PALETTE_OPTIONS.map((p) => {
                const isSelected = palette === p.id;
                return (
                  <button
                    key={p.id}
                    type="button"
                    onClick={() => setPalette(p.id)}
                    className={cn(
                      "p-3 rounded-2xl border text-left flex flex-col justify-between transition-all cursor-pointer group shadow-2xs hover:shadow-xs",
                      isSelected
                        ? "border-primary ring-2 ring-primary/20 bg-card"
                        : "border-border/80 hover:border-primary/40 bg-card/60"
                    )}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <span
                        className="h-6 w-6 rounded-full shadow-xs border border-white/20 transition-transform group-hover:scale-110"
                        style={{ backgroundColor: p.color }}
                      />
                      {isSelected && (
                        <Badge variant="default" size="sm">
                          Active
                        </Badge>
                      )}
                    </div>
                    <span className="font-semibold text-xs text-foreground">
                      {p.name}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
            {/* Appearance Mode */}
            <div className="space-y-2.5">
              <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block">
                Appearance Mode
              </label>
              <div className="grid grid-cols-3 gap-2">
                {[
                  { id: "light", label: "Light", icon: Sun },
                  { id: "dark", label: "Dark", icon: Moon },
                  { id: "system", label: "System", icon: Laptop },
                ].map((item) => {
                  const Icon = item.icon;
                  const isSelected = mode === item.id;
                  return (
                    <Button
                      key={item.id}
                      type="button"
                      variant={isSelected ? "primary" : "outline"}
                      size="sm"
                      onClick={() => setMode(item.id as Mode)}
                      className={cn(
                        "h-10 text-xs font-semibold gap-2 rounded-xl cursor-pointer",
                        !isSelected && "border-border hover:bg-muted"
                      )}
                    >
                      <Icon className="h-4 w-4" />
                      <span>{item.label}</span>
                    </Button>
                  );
                })}
              </div>
            </div>

            {/* Information Density */}
            <div className="space-y-2.5">
              <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block">
                Layout Density
              </label>
              <div className="grid grid-cols-3 gap-2">
                {[
                  { id: "compact", label: "Compact", sub: "36px" },
                  { id: "comfortable", label: "Comfortable", sub: "44px" },
                  { id: "spacious", label: "Spacious", sub: "52px" },
                ].map((item) => {
                  const isSelected = density === item.id;
                  return (
                    <Button
                      key={item.id}
                      type="button"
                      variant={isSelected ? "primary" : "outline"}
                      size="sm"
                      onClick={() => setDensity(item.id as Density)}
                      className={cn(
                        "h-10 text-xs font-semibold flex-col gap-0.5 rounded-xl cursor-pointer",
                        !isSelected && "border-border hover:bg-muted"
                      )}
                    >
                      <span>{item.label}</span>
                      <span className="text-[11px] opacity-75 font-normal">
                        ({item.sub})
                      </span>
                    </Button>
                  );
                })}
              </div>
            </div>

            {/* Corner Radius Style */}
            <div className="space-y-2.5">
              <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block">
                Corner Radius Style (Fatty / Plush)
              </label>
              <div className="grid grid-cols-3 gap-2">
                {[
                  { id: "sharp", label: "Sharp", val: "4px" },
                  { id: "rounded", label: "Rounded (Default)", val: "10px" },
                  { id: "soft", label: "Plush Fatty", val: "16px" },
                ].map((item) => {
                  const isSelected = radius === item.id;
                  return (
                    <Button
                      key={item.id}
                      type="button"
                      variant={isSelected ? "primary" : "outline"}
                      size="sm"
                      onClick={() => setRadius(item.id as Radius)}
                      className={cn(
                        "h-10 text-xs font-semibold flex-col gap-0.5 rounded-xl cursor-pointer",
                        !isSelected && "border-border hover:bg-muted"
                      )}
                    >
                      <span>{item.label}</span>
                      <span className="text-[11px] opacity-75 font-normal">
                        ({item.val})
                      </span>
                    </Button>
                  );
                })}
              </div>
            </div>

            {/* Typography Font Family */}
            <div className="space-y-2.5">
              <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground block">
                Bilingual Font Family
              </label>
              <div className="grid grid-cols-2 gap-2">
                {[
                  { id: "system", label: "OS Native System", sub: "Default" },
                  { id: "plus-jakarta", label: "Plus Jakarta Sans", sub: "EduManage" },
                  { id: "inter", label: "Inter", sub: "Clean & Dense" },
                  { id: "outfit", label: "Outfit", sub: "Modern & Warm" },
                ].map((item) => {
                  const isSelected = fontFamily === item.id;
                  return (
                    <Button
                      key={item.id}
                      type="button"
                      variant={isSelected ? "primary" : "outline"}
                      size="sm"
                      onClick={() => setFontFamily(item.id as FontFamily)}
                      className={cn(
                        "h-10 text-xs font-semibold flex items-center justify-between px-3 rounded-xl cursor-pointer",
                        !isSelected && "border-border hover:bg-muted"
                      )}
                    >
                      <span>{item.label}</span>
                      <span className="text-[11px] opacity-75 font-normal">
                        ({item.sub})
                      </span>
                    </Button>
                  );
                })}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Save Action Bar */}
      <div className="p-4 rounded-2xl border border-border/80 bg-card shadow-sm flex flex-col sm:flex-row items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          {savedSuccess ? (
            <div className="flex items-center gap-2 text-xs font-semibold text-emerald-600 dark:text-emerald-400">
              <CheckCircle2 className="h-4 w-4" />
              <span>
                Institute configuration & appearance preferences saved successfully!
              </span>
            </div>
          ) : (
            <span className="text-xs text-muted-foreground">
              Changes take effect immediately across all desktop workstations.
            </span>
          )}
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleReset}
            className="text-xs gap-1.5 h-10 px-4 rounded-xl cursor-pointer"
          >
            <RotateCcw className="h-3.5 w-3.5" />
            Reset Defaults
          </Button>

          <Button
            type="button"
            variant="primary"
            size="sm"
            onClick={handleSaveAll}
            disabled={saving}
            className="text-xs font-semibold gap-2 h-10 px-5 rounded-xl cursor-pointer shadow-sm"
          >
            <Save className="h-4 w-4" />
            {saving ? "Saving..." : "Save Configuration"}
          </Button>
        </div>
      </div>
    </div>
  );
}
