import { useState } from "react";
import { rpc, type MigrationExportResult, type MigrationImportResult } from "@/gen/rpc";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  ArrowRightLeft,
  Upload,
  Download,
  CheckCircle2,
  AlertTriangle,
  Eye,
  EyeOff,
} from "lucide-react";

export function PcMigrationWizard() {
  // Export state
  const [exportPhrase, setExportPhrase] = useState("");
  const [exportHint, setExportHint] = useState("");
  const [exporting, setExporting] = useState(false);
  const [exportResult, setExportResult] = useState<MigrationExportResult | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);
  const [showExportPhrase, setShowExportPhrase] = useState(false);

  // Import state
  const [importPath, setImportPath] = useState("");
  const [importPhrase, setImportPhrase] = useState("");
  const [importing, setImporting] = useState(false);
  const [importResult, setImportResult] = useState<MigrationImportResult | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  const [showImportPhrase, setShowImportPhrase] = useState(false);

  const handleExport = async () => {
    if (!exportPhrase.trim()) {
      setExportError("Recovery passphrase is required.");
      return;
    }
    setExporting(true);
    setExportError(null);
    setExportResult(null);
    const res = await rpc("integrity.export_migration_bundle", {
      target_hint: exportHint.trim() || null,
      recovery_phrase: exportPhrase,
    });
    setExporting(false);
    if (res.ok && res.data) {
      setExportResult(res.data);
    } else {
      setExportError(res.error?.message ?? "Export failed");
    }
  };

  const handleImport = async () => {
    if (!importPath.trim()) {
      setImportError("Bundle file path is required.");
      return;
    }
    if (!importPhrase.trim()) {
      setImportError("Recovery passphrase is required.");
      return;
    }
    setImporting(true);
    setImportError(null);
    setImportResult(null);
    const res = await rpc("integrity.import_migration_bundle", {
      bundle_path: importPath,
      recovery_phrase: importPhrase,
    });
    setImporting(false);
    if (res.ok && res.data) {
      setImportResult(res.data);
    } else {
      setImportError(res.error?.message ?? "Import failed");
    }
  };

  return (
    <div className="space-y-5">
      {/* Header */}
      <div>
        <h2 className="text-lg font-bold tracking-tight text-foreground flex items-center gap-2">
          <ArrowRightLeft className="h-5 w-5 text-primary" />
          PC Migration Wizard
        </h2>
        <p className="text-xs text-muted-foreground mt-0.5">
          Export a full encrypted bundle for a new PC, or import one from the source
          machine.
        </p>
      </div>

      <Tabs defaultValue="export">
        <TabsList>
          <TabsTrigger value="export" className="gap-1.5 text-xs">
            <Download className="h-3.5 w-3.5" /> Export
          </TabsTrigger>
          <TabsTrigger value="import" className="gap-1.5 text-xs">
            <Upload className="h-3.5 w-3.5" /> Import
          </TabsTrigger>
        </TabsList>

        {/* Export Tab */}
        <TabsContent value="export" className="space-y-4 pt-4">
          <Card className="shadow-sm">
            <CardHeader className="pb-3">
              <CardTitle className="text-sm font-semibold flex items-center gap-2">
                <Download className="h-4 w-4 text-primary" />
                Export Migration Bundle
              </CardTitle>
              <CardDescription className="text-xs">
                Creates an encrypted backup bundle that can be imported on a new PC. Keep
                the passphrase secret — you will need it to import.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-1.5">
                <label htmlFor="export-phrase" className="text-xs font-medium">
                  Recovery Passphrase <span className="text-destructive">*</span>
                </label>
                <div className="relative">
                  <Input
                    id="export-phrase"
                    type={showExportPhrase ? "text" : "password"}
                    value={exportPhrase}
                    onChange={(e) => setExportPhrase(e.target.value)}
                    placeholder="Enter your recovery passphrase"
                    className="text-xs pr-9"
                    disabled={exporting}
                  />
                  <button
                    type="button"
                    onClick={() => setShowExportPhrase((v) => !v)}
                    className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                    aria-label={showExportPhrase ? "Hide passphrase" : "Show passphrase"}
                  >
                    {showExportPhrase ? (
                      <EyeOff className="h-3.5 w-3.5" />
                    ) : (
                      <Eye className="h-3.5 w-3.5" />
                    )}
                  </button>
                </div>
              </div>
              <div className="space-y-1.5">
                <label htmlFor="export-hint" className="text-xs font-medium">
                  Target PC Name{" "}
                  <span className="text-muted-foreground font-normal">(optional)</span>
                </label>
                <Input
                  id="export-hint"
                  type="text"
                  value={exportHint}
                  onChange={(e) => setExportHint(e.target.value)}
                  placeholder='e.g. "Principal Office PC"'
                  className="text-xs"
                  disabled={exporting}
                />
              </div>

              {exportError && (
                <div className="rounded bg-destructive/10 border border-destructive/20 px-3 py-2 text-xs text-destructive">
                  {exportError}
                </div>
              )}

              <Button
                onClick={() => void handleExport()}
                disabled={exporting}
                className="gap-1.5 text-xs"
              >
                <Download className="h-3.5 w-3.5" />
                {exporting ? "Exporting..." : "Export Bundle"}
              </Button>

              {exportResult && (
                <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/5 p-4 space-y-2">
                  <div className="flex items-center gap-2 text-emerald-700 dark:text-emerald-300 font-semibold text-sm">
                    <CheckCircle2 className="h-4 w-4" aria-hidden="true" />
                    Export Successful
                  </div>
                  <div className="space-y-1.5 text-xs">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Bundle Path:</span>
                      <span className="font-mono font-medium max-w-[260px] truncate text-right">
                        {exportResult.bundle_path}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Size:</span>
                      <span className="font-medium">
                        {(exportResult.package_size_bytes / 1024 / 1024).toFixed(2)} MB
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">SHA-256:</span>
                      <span className="font-mono text-[11px]">
                        {exportResult.checksum_sha256.slice(0, 20)}…
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Exported At:</span>
                      <span>{new Date(exportResult.exported_at).toLocaleString()}</span>
                    </div>
                  </div>
                  <div className="rounded bg-blue-500/10 border border-blue-500/20 px-3 py-2 text-xs text-blue-700 dark:text-blue-300">
                    Copy the bundle file to the target PC. You will need the same
                    passphrase to import it.
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Import Tab */}
        <TabsContent value="import" className="space-y-4 pt-4">
          <Card className="shadow-sm">
            <CardHeader className="pb-3">
              <CardTitle className="text-sm font-semibold flex items-center gap-2">
                <Upload className="h-4 w-4 text-primary" />
                Import Migration Bundle
              </CardTitle>
              <CardDescription className="text-xs">
                Restore from a bundle exported on the source machine.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="rounded-lg border border-amber-500/30 bg-amber-500/5 px-4 py-3 flex items-start gap-2.5">
                <AlertTriangle
                  className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5"
                  aria-hidden="true"
                />
                <p className="text-xs text-amber-700 dark:text-amber-300">
                  <strong>Warning:</strong> Importing a bundle will overwrite the current
                  database. Ensure you have a backup before proceeding.
                </p>
              </div>

              <div className="space-y-1.5">
                <label htmlFor="import-path" className="text-xs font-medium">
                  Bundle File Path <span className="text-destructive">*</span>
                </label>
                <Input
                  id="import-path"
                  type="text"
                  value={importPath}
                  onChange={(e) => setImportPath(e.target.value)}
                  placeholder="e.g. C:\Users\...\coachify_migration_bundle.enc"
                  className="text-xs font-mono"
                  disabled={importing}
                />
              </div>
              <div className="space-y-1.5">
                <label htmlFor="import-phrase" className="text-xs font-medium">
                  Recovery Passphrase <span className="text-destructive">*</span>
                </label>
                <div className="relative">
                  <Input
                    id="import-phrase"
                    type={showImportPhrase ? "text" : "password"}
                    value={importPhrase}
                    onChange={(e) => setImportPhrase(e.target.value)}
                    placeholder="Enter the passphrase used during export"
                    className="text-xs pr-9"
                    disabled={importing}
                  />
                  <button
                    type="button"
                    onClick={() => setShowImportPhrase((v) => !v)}
                    className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                    aria-label={showImportPhrase ? "Hide passphrase" : "Show passphrase"}
                  >
                    {showImportPhrase ? (
                      <EyeOff className="h-3.5 w-3.5" />
                    ) : (
                      <Eye className="h-3.5 w-3.5" />
                    )}
                  </button>
                </div>
              </div>

              {importError && (
                <div className="rounded bg-destructive/10 border border-destructive/20 px-3 py-2 text-xs text-destructive">
                  {importError}
                </div>
              )}

              <Button
                onClick={() => void handleImport()}
                disabled={importing}
                variant="destructive"
                className="gap-1.5 text-xs"
              >
                <Upload className="h-3.5 w-3.5" />
                {importing ? "Importing..." : "Import Bundle"}
              </Button>

              {importResult && (
                <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/5 p-4 space-y-2">
                  <div className="flex items-center gap-2 text-emerald-700 dark:text-emerald-300 font-semibold text-sm">
                    <CheckCircle2 className="h-4 w-4" aria-hidden="true" />
                    Import Successful
                  </div>
                  <div className="space-y-1.5 text-xs">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Migration ID:</span>
                      <span className="font-mono">{importResult.migration_id}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Tables Restored:</span>
                      <span className="font-medium">
                        {importResult.restored_tables_count}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Checksum Verified:</span>
                      <span
                        className={
                          importResult.checksum_verified
                            ? "text-emerald-600 dark:text-emerald-400 font-medium"
                            : "text-destructive font-medium"
                        }
                      >
                        {importResult.checksum_verified ? "✓ Yes" : "✗ No"}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Imported At:</span>
                      <span>{new Date(importResult.imported_at).toLocaleString()}</span>
                    </div>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
