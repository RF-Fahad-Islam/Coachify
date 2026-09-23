import { useState } from "react";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { TodayCockpitView } from "./today-cockpit-view";
import { CoachingDayView } from "./coaching-day-view";
import { BatchHealthView } from "./batch-health-view";
import { BatchProfitabilityView } from "./batch-profitability-view";
import { SmartDueQueueView } from "./smart-due-queue-view";
import {
  Compass,
  Clock,
  Activity,
  TrendingUp,
  AlertCircle,
  Sparkles,
} from "lucide-react";

export interface IntelligenceHubProps {
  initialSubTab?: string;
  onNavigateTab?: (tabId: string) => void;
  onSelectStudent?: (studentId: string) => void;
  onCollectPayment?: (studentId?: string) => void;
  onOpenAttendance?: (batchId?: string) => void;
  onNavigateBatch?: (batchId: string) => void;
}

export function IntelligenceHub({
  initialSubTab = "cockpit",
  onNavigateTab,
  onSelectStudent,
  onCollectPayment,
  onOpenAttendance,
  onNavigateBatch,
}: IntelligenceHubProps) {
  const [subTab, setSubTab] = useState(initialSubTab);

  return (
    <div className="space-y-6">
      {/* Top Banner and Navigation Bar */}
      <Tabs value={subTab} onValueChange={setSubTab} className="space-y-6">
        <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-4 border-b border-border pb-4">
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                Owner Intelligence &amp; Operations
              </h1>
              <span className="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-primary-subtle text-primary border border-primary/20">
                <Sparkles className="h-3 w-3" />
                Slice 7
              </span>
            </div>
            <p className="text-xs text-muted-foreground mt-0.5">
              Operations cockpit, whole-day operational stream, batch health vigilance,
              cohort P&amp;L, and prioritized due collection.
            </p>
          </div>

          <TabsList className="flex flex-wrap h-auto gap-1">
            <TabsTrigger value="cockpit" className="gap-1.5 text-xs">
              <Compass className="h-3.5 w-3.5" />
              Today Cockpit
            </TabsTrigger>
            <TabsTrigger value="day_timeline" className="gap-1.5 text-xs">
              <Clock className="h-3.5 w-3.5" />
              Coaching Day
            </TabsTrigger>
            <TabsTrigger value="batch_health" className="gap-1.5 text-xs">
              <Activity className="h-3.5 w-3.5" />
              Batch Health
            </TabsTrigger>
            <TabsTrigger value="profitability" className="gap-1.5 text-xs">
              <TrendingUp className="h-3.5 w-3.5" />
              Batch Profitability
            </TabsTrigger>
            <TabsTrigger value="due_queue" className="gap-1.5 text-xs">
              <AlertCircle className="h-3.5 w-3.5" />
              Smart Due Queue
            </TabsTrigger>
          </TabsList>
        </div>

        {/* 1. Today Cockpit Tab */}
        <TabsContent value="cockpit">
          <TodayCockpitView
            onNavigateTab={onNavigateTab}
            onOpenAttendance={onOpenAttendance}
            onCollectPayment={onCollectPayment}
          />
        </TabsContent>

        {/* 2. Coaching Day Timeline Tab */}
        <TabsContent value="day_timeline">
          <CoachingDayView />
        </TabsContent>

        {/* 3. Batch Health Tab */}
        <TabsContent value="batch_health">
          <BatchHealthView
            onNavigateBatch={onNavigateBatch}
            onOpenAttendance={onOpenAttendance}
            onNavigateTab={onNavigateTab}
          />
        </TabsContent>

        {/* 4. Batch Profitability Tab */}
        <TabsContent value="profitability">
          <BatchProfitabilityView />
        </TabsContent>

        {/* 5. Smart Due Recovery Queue Tab */}
        <TabsContent value="due_queue">
          <SmartDueQueueView
            onCollectDue={(id) => onCollectPayment?.(id)}
            onViewStudentProfile={onSelectStudent}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
