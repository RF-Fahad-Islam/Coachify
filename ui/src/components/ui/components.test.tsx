import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { Button, buttonVariants } from "./button";
import { Badge, badgeVariants } from "./badge";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "./card";
import { MetricCard } from "./metric-card";
import { Progress } from "./progress";
import { Input } from "./input";
import { Switch } from "./switch";
import {
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableHead,
  TableRow,
  TableCell,
} from "./table";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./tabs";
import { Avatar, AvatarFallback } from "./avatar";
import { Skeleton } from "./skeleton";
import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "./dialog";
import {
  Command,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from "./command";
import { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from "./tooltip";
import { Users } from "lucide-react";

describe("Design system component primitives", () => {
  describe("Button", () => {
    it("renders primary button variant with default size", () => {
      const html = renderToString(<Button>Save Record</Button>);
      expect(html).toContain("Save Record");
      expect(html).toContain("bg-primary");
      expect(html).toContain("text-primary-foreground");
    });

    it("generates correct class names for outline and destructive variants", () => {
      const outlineClasses = buttonVariants({ variant: "outline", size: "sm" });
      expect(outlineClasses).toContain("border-border");
      expect(outlineClasses).toContain("h-8");

      const destructiveClasses = buttonVariants({ variant: "destructive" });
      expect(destructiveClasses).toContain("bg-danger");
    });
  });

  describe("Badge", () => {
    it("renders payment and status badge variants", () => {
      const paidClasses = badgeVariants({ variant: "paid" });
      expect(paidClasses).toContain("bg-success-subtle");

      const overdueClasses = badgeVariants({ variant: "overdue" });
      expect(overdueClasses).toContain("bg-danger-subtle");

      const pendingClasses = badgeVariants({ variant: "pending" });
      expect(pendingClasses).toContain("bg-warning-subtle");

      const html = renderToString(<Badge variant="paid">Paid</Badge>);
      expect(html).toContain("Paid");
    });
  });

  describe("Card and MetricCard", () => {
    it("renders Card container with header and content", () => {
      const html = renderToString(
        <Card>
          <CardHeader>
            <CardTitle>Batch Performance</CardTitle>
            <CardDescription>Monthly test results</CardDescription>
          </CardHeader>
          <CardContent>Average score: 84%</CardContent>
          <CardFooter>Updated today</CardFooter>
        </Card>
      );
      expect(html).toContain("Batch Performance");
      expect(html).toContain("Monthly test results");
      expect(html).toContain("Average score: 84%");
      expect(html).toContain("Updated today");
    });

    it("renders MetricCard with delta and icon", () => {
      const html = renderToString(
        <MetricCard
          title="Total Students"
          value="1,420"
          trend={{ value: "+12%", isPositive: true }}
          subtitle="vs last month"
          icon={<Users className="h-5 w-5" />}
        />
      );
      expect(html).toContain("Total Students");
      expect(html).toContain("1,420");
      expect(html).toContain("+12%");
      expect(html).toContain("vs last month");
    });
  });

  describe("Progress", () => {
    it("renders progress track and indicator percentage", () => {
      const html = renderToString(<Progress value={75} />);
      expect(html).toContain('role="progressbar"');
      expect(html).toContain('style="width:75%"');
    });
  });

  describe("Input and Switch", () => {
    it("renders Input with accessible attributes", () => {
      const html = renderToString(
        <Input placeholder="Search records..." aria-label="Search" />
      );
      expect(html).toContain('placeholder="Search records..."');
      expect(html).toContain('aria-label="Search"');
      expect(html).toContain("h-[var(--input-height)]");
    });

    it("renders Switch primitive", () => {
      const html = renderToString(<Switch aria-label="Toggle notifications" />);
      expect(html).toContain('role="switch"');
      expect(html).toContain('aria-label="Toggle notifications"');
    });
  });

  describe("Table", () => {
    it("renders semantic table markup with headers and rows", () => {
      const html = renderToString(
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Student</TableHead>
              <TableHead>Roll</TableHead>
              <TableHead>Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow>
              <TableCell>Tanvir Ahmed</TableCell>
              <TableCell>101</TableCell>
              <TableCell>Active</TableCell>
            </TableRow>
          </TableBody>
          <TableFooter>
            <TableRow>
              <TableCell colSpan={3}>1 student total</TableCell>
            </TableRow>
          </TableFooter>
        </Table>
      );
      expect(html).toContain("<table");
      expect(html).toContain("Tanvir Ahmed");
      expect(html).toContain("101");
      expect(html).toContain("1 student total");
    });
  });

  describe("Tabs", () => {
    it("renders segmented tabs structure", () => {
      const html = renderToString(
        <Tabs defaultValue="all">
          <TabsList>
            <TabsTrigger value="all">All Batches</TabsTrigger>
            <TabsTrigger value="active">Active Only</TabsTrigger>
          </TabsList>
          <TabsContent value="all">All content</TabsContent>
          <TabsContent value="active">Active content</TabsContent>
        </Tabs>
      );
      expect(html).toContain("All Batches");
      expect(html).toContain("Active Only");
      expect(html).toContain('role="tablist"');
    });
  });

  describe("Avatar and Skeleton", () => {
    it("renders avatar with fallback initials", () => {
      const html = renderToString(
        <Avatar>
          <AvatarFallback>AK</AvatarFallback>
        </Avatar>
      );
      expect(html).toContain("AK");
    });

    it("renders skeleton loading placeholder", () => {
      const html = renderToString(<Skeleton className="h-4 w-32" />);
      expect(html).toContain("animate-pulse");
    });
  });

  describe("Dialog, Tooltip, and Command", () => {
    it("renders Dialog structure components without crashing", () => {
      const html = renderToString(
        <Dialog open={false}>
          <DialogTrigger>Open</DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Fee Voucher</DialogTitle>
              <DialogDescription>Review transaction</DialogDescription>
            </DialogHeader>
            <DialogFooter>Confirm</DialogFooter>
          </DialogContent>
        </Dialog>
      );
      expect(html).toContain("Open");
    });

    it("renders Tooltip components inside TooltipProvider", () => {
      const html = renderToString(
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger>Hover me</TooltipTrigger>
            <TooltipContent>Help text</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      );
      expect(html).toContain("Hover me");
    });

    it("renders Command palette components", () => {
      const html = renderToString(
        <Command>
          <CommandInput placeholder="Search actions..." />
          <CommandList>
            <CommandEmpty>No results</CommandEmpty>
            <CommandGroup heading="Actions">
              <CommandItem>
                <span>Add Record</span>
                <CommandShortcut>Ctrl+A</CommandShortcut>
              </CommandItem>
            </CommandGroup>
          </CommandList>
        </Command>
      );
      expect(html).toContain("Search actions...");
      expect(html).toContain("Add Record");
      expect(html).toContain("Ctrl+A");
    });
  });
});
