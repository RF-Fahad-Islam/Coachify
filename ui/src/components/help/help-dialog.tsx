import { useMemo, useRef, useState } from "react";
import {
  BookOpen,
  Search,
  Printer,
  ArrowUp,
  CheckCircle2,
  ChevronRight,
  WifiOff,
  FileText,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  getCustomerDocs,
  filterCustomerDocs,
  printCustomerGuideAsPdf,
  type CustomerDocTopic,
} from "@/lib/docs";

export interface HelpDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  initialTopicId?: string;
  initialSearchQuery?: string;
}

function renderInlineSegments(text: string): React.ReactNode[] {
  const parts = text.split(/(\*\*[^*]+\*\*|`[^`]+`)/g);
  return parts.map((part, idx) => {
    if (part.startsWith("**") && part.endsWith("**")) {
      return (
        <strong key={idx} className="font-semibold text-foreground">
          {part.slice(2, -2)}
        </strong>
      );
    }
    if (part.startsWith("`") && part.endsWith("`")) {
      return (
        <code
          key={idx}
          className="px-1.5 py-0.5 rounded bg-muted text-foreground font-mono text-xs border border-border/60"
        >
          {part.slice(1, -1)}
        </code>
      );
    }
    return part;
  });
}

function MarkdownArticleRenderer({ content }: { content: string }) {
  const lines = content.split(/\r?\n/);
  const elements: React.ReactNode[] = [];
  let i = 0;

  while (i < lines.length) {
    const rawLine = lines[i];
    const line = rawLine.trim();

    if (!line) {
      i++;
      continue;
    }

    if (line === "---") {
      elements.push(<hr key={`hr-${i}`} className="my-5 border-border" />);
      i++;
      continue;
    }

    if (line.startsWith("# ")) {
      elements.push(
        <h2
          key={`h1-${i}`}
          className="text-xl font-bold tracking-tight text-foreground mb-3"
        >
          {renderInlineSegments(line.slice(2))}
        </h2>
      );
      i++;
      continue;
    }

    if (line.startsWith("## ")) {
      elements.push(
        <h3
          key={`h2-${i}`}
          className="text-base font-semibold text-foreground mt-6 mb-2 pb-1 border-b border-border/60"
        >
          {renderInlineSegments(line.slice(3))}
        </h3>
      );
      i++;
      continue;
    }

    if (line.startsWith("### ")) {
      elements.push(
        <h4 key={`h3-${i}`} className="text-sm font-semibold text-foreground mt-4 mb-1.5">
          {renderInlineSegments(line.slice(4))}
        </h4>
      );
      i++;
      continue;
    }

    if (line.startsWith("> ")) {
      elements.push(
        <blockquote
          key={`quote-${i}`}
          className="my-3 pl-3.5 py-2 pr-3 border-l-4 border-primary bg-primary/5 rounded-r-md text-xs sm:text-sm text-foreground/90"
        >
          {renderInlineSegments(line.slice(2))}
        </blockquote>
      );
      i++;
      continue;
    }

    // Markdown table block
    if (line.startsWith("|") && line.endsWith("|")) {
      const tableLines: string[] = [];
      while (i < lines.length && lines[i].trim().startsWith("|")) {
        tableLines.push(lines[i].trim());
        i++;
      }
      if (tableLines.length >= 2) {
        const parseRow = (row: string) =>
          row
            .slice(1, -1)
            .split("|")
            .map((cell) => cell.trim());
        const headers = parseRow(tableLines[0]);
        const dataRows = tableLines.slice(2).map(parseRow);
        elements.push(
          <div
            key={`table-${i}`}
            className="my-3 overflow-x-auto rounded-lg border border-border"
          >
            <table className="w-full text-xs text-left border-collapse">
              <thead className="bg-muted/60 text-foreground font-semibold border-b border-border">
                <tr>
                  {headers.map((header, hIdx) => (
                    <th key={hIdx} className="px-3 py-2">
                      {renderInlineSegments(header)}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {dataRows.map((row, rIdx) => (
                  <tr key={rIdx} className="hover:bg-muted/30">
                    {row.map((cell, cIdx) => (
                      <td key={cIdx} className="px-3 py-2 text-muted-foreground">
                        {renderInlineSegments(cell)}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        );
        continue;
      }
    }

    // List block (unordered or numbered)
    if (line.startsWith("- ") || line.startsWith("* ") || /^\d+\.\s+/.test(line)) {
      const listItems: string[] = [];
      while (
        i < lines.length &&
        (lines[i].trim().startsWith("- ") ||
          lines[i].trim().startsWith("* ") ||
          /^\d+\.\s+/.test(lines[i].trim()))
      ) {
        listItems.push(lines[i].trim().replace(/^(-|\*|\d+\.)\s+/, ""));
        i++;
      }
      elements.push(
        <ul
          key={`list-${i}`}
          className="my-2.5 space-y-1.5 pl-4 list-disc text-xs sm:text-sm text-muted-foreground"
        >
          {listItems.map((item, idx) => {
            const cleanItem = item.replace(/^\[[ xX]\]\s*/, "");
            const isChecklist = /^\[[ xX]\]/.test(item);
            return (
              <li key={idx} className="leading-relaxed">
                {isChecklist && (
                  <span className="inline-block mr-1.5 text-primary font-semibold">
                    ✓
                  </span>
                )}
                {renderInlineSegments(cleanItem)}
              </li>
            );
          })}
        </ul>
      );
      continue;
    }

    elements.push(
      <p
        key={`p-${i}`}
        className="text-xs sm:text-sm text-muted-foreground leading-relaxed my-2"
      >
        {renderInlineSegments(line)}
      </p>
    );
    i++;
  }

  return <div className="space-y-1">{elements}</div>;
}

export function HelpDialog({
  open,
  onOpenChange,
  initialTopicId,
  initialSearchQuery = "",
}: HelpDialogProps) {
  const allTopics = useMemo(() => getCustomerDocs(), []);
  const [searchQuery, setSearchQuery] = useState(initialSearchQuery);
  const [selectedTopicId, setSelectedTopicId] = useState<string>(
    initialTopicId ?? allTopics[0]?.id ?? "01-quick-start"
  );
  const [isPrinting, setIsPrinting] = useState(false);
  const [printSuccess, setPrintSuccess] = useState(false);
  const articleScrollRef = useRef<HTMLDivElement | null>(null);

  const filteredTopics = useMemo(
    () => filterCustomerDocs(allTopics, searchQuery),
    [allTopics, searchQuery]
  );

  const activeTopic: CustomerDocTopic | undefined = useMemo(() => {
    const inFiltered = filteredTopics.find((t) => t.id === selectedTopicId);
    if (inFiltered) return inFiltered;
    return filteredTopics[0] ?? allTopics[0];
  }, [filteredTopics, selectedTopicId, allTopics]);

  const handleSelectTopic = (topicId: string) => {
    setSelectedTopicId(topicId);
    setPrintSuccess(false);
    if (articleScrollRef.current) {
      articleScrollRef.current.scrollTo({ top: 0, behavior: "smooth" });
    }
  };

  const handleBackToTop = () => {
    if (articleScrollRef.current) {
      articleScrollRef.current.scrollTo({ top: 0, behavior: "smooth" });
    }
  };

  const handlePrintGuide = async () => {
    if (!activeTopic || isPrinting) return;
    setIsPrinting(true);
    setPrintSuccess(false);
    try {
      const result = await printCustomerGuideAsPdf(activeTopic);
      if (result.success) {
        setPrintSuccess(true);
      }
    } finally {
      setIsPrinting(false);
    }
  };

  if (!open) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Coachify Help & Documentation"
      data-testid="help-dialog"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-3 sm:p-6"
    >
      <div className="max-w-5xl w-[95vw] h-[85vh] rounded-xl border border-border bg-card text-card-foreground shadow-lg flex flex-col overflow-hidden relative">
        {/* Top Header & Search Bar */}
        <div className="px-6 py-4 border-b border-border bg-muted/30 shrink-0">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pr-8">
            <div className="flex items-center gap-2.5">
              <div className="h-9 w-9 rounded-lg bg-primary/10 text-primary flex items-center justify-center shrink-0">
                <BookOpen className="h-5 w-5" />
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <h2 className="text-base sm:text-lg font-bold text-foreground">
                    Coachify Help & Documentation
                  </h2>
                  <Badge
                    variant="outline"
                    className="text-[10px] gap-1 font-medium border-emerald-500/30 text-emerald-700 dark:text-emerald-300 bg-emerald-500/10"
                  >
                    <WifiOff className="h-3 w-3" />
                    100% Offline Ready
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground">
                  Step-by-step setup, printer, backup, and operational guides (বাংলা ও
                  English)
                </p>
              </div>
            </div>

            <div className="relative w-full sm:w-72">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                type="text"
                placeholder="Search help guides (e.g. printer, backup)..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                data-testid="help-search-input"
                className="pl-9 h-9 text-xs bg-card"
              />
            </div>
          </div>

          <Button
            variant="ghost"
            size="sm"
            onClick={() => onOpenChange(false)}
            aria-label="Close help dialog"
            className="absolute right-3 top-3 h-8 w-8 p-0 text-muted-foreground hover:text-foreground cursor-pointer"
          >
            ✕
          </Button>
        </div>

        {/* Main Split Body: Sidebar + Markdown Content */}
        <div className="flex-1 flex flex-col md:flex-row min-h-0 overflow-hidden">
          {/* Left Topics Sidebar */}
          <aside className="w-full md:w-64 border-b md:border-b-0 md:border-r border-border bg-muted/20 flex flex-col justify-between shrink-0">
            <div className="p-3 overflow-y-auto flex-1">
              <div className="px-2 pb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
                {`Topics (${filteredTopics.length})`}
              </div>

              {filteredTopics.length === 0 ? (
                <div className="px-2 py-6 text-center text-xs text-muted-foreground">
                  No guides match &ldquo;{searchQuery}&rdquo;.
                </div>
              ) : (
                <nav aria-label="Help documentation topics" className="space-y-1">
                  {filteredTopics.map((topic) => {
                    const isActive = activeTopic?.id === topic.id;
                    return (
                      <button
                        key={topic.id}
                        type="button"
                        data-testid={`help-topic-${topic.id}`}
                        onClick={() => handleSelectTopic(topic.id)}
                        className={`w-full text-left px-3 py-2 rounded-md text-xs transition-colors flex items-center justify-between gap-2 cursor-pointer ${
                          isActive
                            ? "bg-primary text-primary-foreground font-semibold shadow-2xs"
                            : "text-foreground/80 hover:bg-muted hover:text-foreground"
                        }`}
                      >
                        <div className="min-w-0">
                          <div className="truncate">{topic.shortTitle}</div>
                          <div
                            className={`text-[10px] truncate mt-0.5 ${
                              isActive
                                ? "text-primary-foreground/80"
                                : "text-muted-foreground"
                            }`}
                          >
                            {topic.titleBn}
                          </div>
                        </div>
                        <ChevronRight
                          className={`h-3.5 w-3.5 shrink-0 ${
                            isActive ? "opacity-100" : "opacity-40"
                          }`}
                        />
                      </button>
                    );
                  })}
                </nav>
              )}
            </div>

            {/* Sidebar Footer Action: Print Guide as PDF */}
            <div className="p-3 border-t border-border bg-card/50 space-y-2">
              <Button
                variant="outline"
                size="sm"
                disabled={!activeTopic || isPrinting}
                onClick={() => void handlePrintGuide()}
                data-testid="print-guide-pdf-btn"
                className="w-full text-xs gap-1.5 cursor-pointer justify-center"
              >
                <Printer className="h-3.5 w-3.5 text-primary" />
                <span>{isPrinting ? "Preparing Print..." : "Print Guide as PDF"}</span>
              </Button>
              {printSuccess && (
                <div
                  data-testid="print-guide-feedback"
                  className="flex items-center justify-center gap-1 text-[11px] text-emerald-600 dark:text-emerald-400 font-medium"
                >
                  <CheckCircle2 className="h-3 w-3" />
                  Sent to print preview
                </div>
              )}
            </div>
          </aside>

          {/* Right Markdown Article Viewer */}
          <div className="flex-1 flex flex-col min-w-0 min-h-0 bg-card">
            <div
              ref={articleScrollRef}
              data-testid="help-article-content"
              className="flex-1 overflow-y-auto px-6 py-5"
            >
              {activeTopic ? (
                <article className="max-w-3xl">
                  <div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
                    <FileText className="h-3.5 w-3.5 text-primary" />
                    <span>
                      Guide #{activeTopic.order} of {allTopics.length}
                    </span>
                    <span>·</span>
                    <span>{activeTopic.filename}</span>
                  </div>
                  <MarkdownArticleRenderer content={activeTopic.content} />
                </article>
              ) : (
                <div className="py-16 text-center text-sm text-muted-foreground">
                  Select a help topic from the sidebar or clear your search filter.
                </div>
              )}
            </div>

            {/* Bottom Article Bar with Back to Top */}
            <div className="px-6 py-2.5 border-t border-border bg-muted/20 flex items-center justify-between text-xs text-muted-foreground shrink-0">
              <span>
                Need emergency recovery assistance? Export a Diagnostic Bundle in Settings
                → Diagnostics.
              </span>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleBackToTop}
                data-testid="help-back-to-top-btn"
                className="h-7 text-xs gap-1 cursor-pointer"
              >
                <ArrowUp className="h-3.5 w-3.5" />
                Back to Top
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
