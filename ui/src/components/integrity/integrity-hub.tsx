import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { DisasterRecoveryView } from "@/components/backup/disaster-recovery-view";
import { DatabaseIntegrityView } from "@/components/restore/database-integrity-view";
import { GuardianContactHealthView } from "@/components/communication/guardian-contact-health-view";
import { OutboxLifecycleMonitor } from "@/components/communication/outbox-lifecycle-monitor";
import { PcMigrationWizard } from "@/components/setup/pc-migration-wizard";
import { ShieldCheck, Database, Users, Send, ArrowRightLeft } from "lucide-react";

export function IntegrityHub() {
  return (
    <div className="space-y-6">
      {/* Page header */}
      <div className="border-b border-border pb-4">
        <h1 className="text-2xl font-bold tracking-tight text-foreground flex items-center gap-2">
          <ShieldCheck className="h-6 w-6 text-primary" />
          Operational Integrity
        </h1>
        <p className="text-xs text-muted-foreground mt-0.5">
          System health, contact quality, communication delivery, and data protection.
        </p>
      </div>

      <Tabs defaultValue="recovery" className="space-y-4">
        <TabsList className="flex flex-wrap h-auto gap-1">
          <TabsTrigger value="recovery" className="gap-1.5 text-xs">
            <ShieldCheck className="h-3.5 w-3.5" /> Recovery
          </TabsTrigger>
          <TabsTrigger value="database" className="gap-1.5 text-xs">
            <Database className="h-3.5 w-3.5" /> Database
          </TabsTrigger>
          <TabsTrigger value="contacts" className="gap-1.5 text-xs">
            <Users className="h-3.5 w-3.5" /> Contacts
          </TabsTrigger>
          <TabsTrigger value="outbox" className="gap-1.5 text-xs">
            <Send className="h-3.5 w-3.5" /> Outbox
          </TabsTrigger>
          <TabsTrigger value="migration" className="gap-1.5 text-xs">
            <ArrowRightLeft className="h-3.5 w-3.5" /> PC Migration
          </TabsTrigger>
        </TabsList>

        <TabsContent value="recovery">
          <DisasterRecoveryView />
        </TabsContent>

        <TabsContent value="database">
          <DatabaseIntegrityView />
        </TabsContent>

        <TabsContent value="contacts">
          <GuardianContactHealthView />
        </TabsContent>

        <TabsContent value="outbox">
          <OutboxLifecycleMonitor />
        </TabsContent>

        <TabsContent value="migration">
          <PcMigrationWizard />
        </TabsContent>
      </Tabs>
    </div>
  );
}
