import { rpc } from "../gen/rpc";

export type ExportableReportType =
  | "students_list"
  | "dues_list"
  | "exam_results_list"
  | "attendance"
  | "collection_summary"
  | "collector_summary"
  | "dues_aging"
  | "expenses"
  | "exam_performance";

/**
 * Triggers a browser download for a UTF-8 CSV string, ensuring the UTF-8 BOM (`\uFEFF`)
 * is present so Microsoft Excel opens Bangla characters and `৳` symbols without mojibake.
 */
export function triggerCsvDownload(filename: string, csvContent: string): void {
  const normalizedContent = csvContent.startsWith("\uFEFF")
    ? csvContent
    : `\uFEFF${csvContent}`;

  const blob = new Blob([normalizedContent], {
    type: "text/csv;charset=utf-8;",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.setAttribute("download", filename);
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * Calls the Rust core `reports.export_csv` RPC endpoint with the active filter state
 * and triggers an Excel-compatible `.csv` download.
 */
export async function exportCsvFromCore(
  reportType: ExportableReportType,
  params: Record<string, unknown> = {},
  sessionToken?: string | null
): Promise<{ ok: boolean; filename?: string; error?: string }> {
  const res = await rpc(
    "reports.export_csv",
    {
      report_type: reportType,
      params,
    },
    sessionToken
  );

  if (res.ok && res.data) {
    triggerCsvDownload(res.data.filename, res.data.csv_content);
    return { ok: true, filename: res.data.filename };
  }

  return {
    ok: false,
    error: res.error?.message ?? "Failed to export CSV",
  };
}
