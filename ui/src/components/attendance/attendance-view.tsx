import { useState } from "react";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AttendanceSheet } from "./attendance-sheet";
import { AttendanceMatrixView } from "./attendance-matrix-view";
import { CalendarCheck, CalendarDays } from "lucide-react";

export function AttendanceView() {
  const [subTab, setSubTab] = useState<string>("sheet");
  const [sharedBatchId, setSharedBatchId] = useState<string>("");

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Attendance & Class Register
          </h1>
          <p className="text-xs text-muted-foreground mt-0.5">
            Rapid batch attendance marking with keyboard shortcuts and monthly attendance
            registers.
          </p>
        </div>

        <Tabs value={subTab} onValueChange={setSubTab} className="w-auto">
          <TabsList className="grid grid-cols-2 h-9 p-1 bg-muted border border-border">
            <TabsTrigger value="sheet" className="gap-1.5 text-xs font-semibold">
              <CalendarCheck className="h-3.5 w-3.5" />
              Daily Sheet
            </TabsTrigger>
            <TabsTrigger value="matrix" className="gap-1.5 text-xs font-semibold">
              <CalendarDays className="h-3.5 w-3.5" />
              Monthly Matrix
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      {subTab === "sheet" ? (
        <AttendanceSheet
          initialBatchId={sharedBatchId}
          onBatchChange={setSharedBatchId}
        />
      ) : (
        <AttendanceMatrixView
          initialBatchId={sharedBatchId}
          onBatchChange={setSharedBatchId}
        />
      )}
    </div>
  );
}
