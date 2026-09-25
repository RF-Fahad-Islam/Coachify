import { executeHtmlPrintWithAssets, type PrintResult } from "@/lib/printer";

export interface CustomerDocTopic {
  id: string;
  filename: string;
  order: number;
  title: string;
  shortTitle: string;
  titleBn: string;
  headings: string[];
  content: string;
}

const RAW_DOCS_GLOB = import.meta.glob<string>("../../../docs/customer/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
});

const TOPIC_METADATA: Record<
  string,
  { id: string; order: number; shortTitle: string; titleBn: string }
> = {
  "01-quick-start.md": {
    id: "01-quick-start",
    order: 1,
    shortTitle: "Quick Start Guide",
    titleBn: "দ্রুত শুরু করার নির্দেশিকা",
  },
  "02-backup-and-restore.md": {
    id: "02-backup-and-restore",
    order: 2,
    shortTitle: "Backup and Restore",
    titleBn: "ব্যাকআপ এবং রিস্টোর",
  },
  "03-printer-setup.md": {
    id: "03-printer-setup",
    order: 3,
    shortTitle: "Printer Setup",
    titleBn: "প্রিন্টার সেটআপ",
  },
  "04-sms-setup.md": {
    id: "04-sms-setup",
    order: 4,
    shortTitle: "SMS Setup",
    titleBn: "এসএমএস সেটআপ",
  },
  "05-excel-student-import.md": {
    id: "05-excel-student-import",
    order: 5,
    shortTitle: "Excel Import",
    titleBn: "এক্সেল ইমপোর্ট",
  },
  "06-troubleshooting.md": {
    id: "06-troubleshooting",
    order: 6,
    shortTitle: "Troubleshooting",
    titleBn: "সমস্যা ও সমাধান",
  },
  "07-privacy-and-ownership.md": {
    id: "07-privacy-and-ownership",
    order: 7,
    shortTitle: "Privacy & Ownership",
    titleBn: "গোপনীয়তা ও মালিকানা",
  },
  "08-license-guide.md": {
    id: "08-license-guide",
    order: 8,
    shortTitle: "License & Activation",
    titleBn: "লাইসেন্স ও অ্যাক্টিভেশন",
  },
};

function extractTitle(markdown: string, fallback: string): string {
  const match = /^#\s+(.+)$/m.exec(markdown);
  return match ? match[1].trim() : fallback;
}

function extractHeadings(markdown: string): string[] {
  const headings: string[] = [];
  const regex = /^#{1,3}\s+(.+)$/gm;
  let match = regex.exec(markdown);
  while (match !== null) {
    headings.push(match[1].trim());
    match = regex.exec(markdown);
  }
  return headings;
}

/**
 * Loads and parses the 8 bundled customer documentation guides in order.
 * Excludes internal operational runbooks (`support-runbook.md`) from customer view.
 */
export function getCustomerDocs(): CustomerDocTopic[] {
  const topics: CustomerDocTopic[] = [];

  for (const [globPath, rawContent] of Object.entries(RAW_DOCS_GLOB)) {
    const filename = globPath.split("/").pop() ?? "";
    const meta = TOPIC_METADATA[filename];
    if (!meta) {
      continue;
    }

    topics.push({
      id: meta.id,
      filename,
      order: meta.order,
      title: extractTitle(rawContent, meta.shortTitle),
      shortTitle: meta.shortTitle,
      titleBn: meta.titleBn,
      headings: extractHeadings(rawContent),
      content: rawContent,
    });
  }

  return topics.sort((a, b) => a.order - b.order);
}

/**
 * Filters customer documentation topics by search query across title, Bengali title,
 * section headings, and full markdown body.
 */
export function filterCustomerDocs(
  topics: CustomerDocTopic[],
  query: string
): CustomerDocTopic[] {
  const normalized = query.trim().toLowerCase();
  if (!normalized) {
    return topics;
  }

  return topics.filter((topic) => {
    if (topic.shortTitle.toLowerCase().includes(normalized)) return true;
    if (topic.title.toLowerCase().includes(normalized)) return true;
    if (topic.titleBn.toLowerCase().includes(normalized)) return true;
    if (topic.headings.some((h) => h.toLowerCase().includes(normalized))) {
      return true;
    }
    return topic.content.toLowerCase().includes(normalized);
  });
}

function escapeHtml(input: string): string {
  return input
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function formatInlineHtml(text: string): string {
  const escaped = escapeHtml(text);
  return escaped
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/`([^`]+)`/g, "<code>$1</code>");
}

/**
 * Renders a CustomerDocTopic into a clean, print-ready A4 HTML document
 * compatible with the WebView2 print engine.
 */
export function buildPrintableGuideHtml(topic: CustomerDocTopic): string {
  const lines = topic.content.split(/\r?\n/);
  const bodyParts: string[] = [];
  let inList = false;

  const closeList = () => {
    if (inList) {
      bodyParts.push("</ul>");
      inList = false;
    }
  };

  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (!line) {
      closeList();
      continue;
    }

    if (line === "---") {
      closeList();
      bodyParts.push("<hr />");
      continue;
    }

    if (line.startsWith("# ")) {
      closeList();
      bodyParts.push(`<h1>${formatInlineHtml(line.slice(2))}</h1>`);
      continue;
    }

    if (line.startsWith("## ")) {
      closeList();
      bodyParts.push(`<h2>${formatInlineHtml(line.slice(3))}</h2>`);
      continue;
    }

    if (line.startsWith("### ")) {
      closeList();
      bodyParts.push(`<h3>${formatInlineHtml(line.slice(4))}</h3>`);
      continue;
    }

    if (line.startsWith("> ")) {
      closeList();
      bodyParts.push(`<blockquote>${formatInlineHtml(line.slice(2))}</blockquote>`);
      continue;
    }

    if (line.startsWith("- ") || line.startsWith("* ") || /^\d+\.\s+/.test(line)) {
      if (!inList) {
        bodyParts.push("<ul>");
        inList = true;
      }
      const itemText = line.replace(/^(-|\*|\d+\.)\s+/, "");
      bodyParts.push(`<li>${formatInlineHtml(itemText)}</li>`);
      continue;
    }

    closeList();
    bodyParts.push(`<p>${formatInlineHtml(line)}</p>`);
  }

  closeList();

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>${escapeHtml(topic.shortTitle)} — Coachify Documentation</title>
  <style>
    @page { size: A4 portrait; margin: 16mm; }
    body {
      font-family: 'Inter', 'Noto Sans Bengali', 'Kalpurush', sans-serif;
      color: #0f172a;
      line-height: 1.55;
      font-size: 11pt;
    }
    .doc-header {
      border-bottom: 2px solid #0f172a;
      padding-bottom: 8px;
      margin-bottom: 16px;
      display: flex;
      justify-content: space-between;
      font-size: 9pt;
      color: #475569;
    }
    h1 { font-size: 18pt; margin: 0 0 12px 0; color: #0f172a; }
    h2 { font-size: 13pt; margin: 18px 0 8px 0; color: #1e293b; border-bottom: 1px solid #cbd5e1; padding-bottom: 4px; }
    h3 { font-size: 11.5pt; margin: 12px 0 6px 0; color: #334155; }
    p, li { margin-bottom: 6px; }
    ul { padding-left: 20px; margin: 6px 0 12px 0; }
    code {
      background: #f1f5f9;
      padding: 1px 4px;
      border-radius: 3px;
      font-family: Consolas, monospace;
      font-size: 10pt;
    }
    blockquote {
      margin: 10px 0;
      padding: 8px 12px;
      background: #f8fafc;
      border-left: 4px solid #2563eb;
    }
    hr { border: 0; border-top: 1px solid #e2e8f0; margin: 14px 0; }
  </style>
</head>
<body>
  <div class="doc-header">
    <span><strong>Coachify Desktop</strong> · Offline Reference Manual</span>
    <span>Topic: ${escapeHtml(topic.shortTitle)}</span>
  </div>
  ${bodyParts.join("\n")}
</body>
</html>`;
}

/**
 * Triggers an offline A4 print / PDF export for the selected customer documentation guide.
 */
export async function printCustomerGuideAsPdf(
  topic: CustomerDocTopic,
  printFn: (html: string) => Promise<PrintResult> = executeHtmlPrintWithAssets
): Promise<PrintResult> {
  const html = buildPrintableGuideHtml(topic);
  return printFn(html);
}
