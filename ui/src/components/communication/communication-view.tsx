import { useState } from "react";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { OutboxMonitorView } from "./outbox-monitor-view";
import { BatchBroadcastView } from "./batch-broadcast-view";
import { DueRemindersView } from "./due-reminders-view";
import { Send, Radio, Bell } from "lucide-react";

export function CommunicationView() {
  const [activeSubTab, setActiveSubTab] = useState<string>("outbox");

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Communication & Messaging
          </h1>
          <p className="text-xs text-muted-foreground mt-0.5">
            Durable transactional outbox, multi-channel batch announcements, and fee due
            reminders.
          </p>
        </div>

        <Tabs value={activeSubTab} onValueChange={setActiveSubTab} className="w-auto">
          <TabsList className="grid grid-cols-3 h-9 p-1 bg-muted border border-border">
            <TabsTrigger value="outbox" className="gap-1.5 text-xs font-semibold">
              <Send className="h-3.5 w-3.5" />
              Outbox Monitor
            </TabsTrigger>
            <TabsTrigger value="broadcast" className="gap-1.5 text-xs font-semibold">
              <Radio className="h-3.5 w-3.5" />
              Batch Broadcast
            </TabsTrigger>
            <TabsTrigger value="due" className="gap-1.5 text-xs font-semibold">
              <Bell className="h-3.5 w-3.5" />
              Due Reminders
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      {activeSubTab === "outbox" && <OutboxMonitorView />}
      {activeSubTab === "broadcast" && <BatchBroadcastView />}
      {activeSubTab === "due" && <DueRemindersView />}
    </div>
  );
}
