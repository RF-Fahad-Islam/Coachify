import type { SVGProps } from "react";
import { cn } from "../utils/cn";
import {
  Sparkles,
  WifiOff,
  Monitor,
  Network,
  Banknote,
  ClipboardCheck,
  FileText,
  Bell,
  ShieldCheck,
  Database,
  Users,
  Printer,
  Download,
  ArrowRight,
  Check,
  X,
  ChevronDown,
  Phone,
  Mail,
  MapPin,
  Menu,
  Calendar,
  BarChart3,
  Clock,
  MessageSquare,
  Star,
  Search,
  Lock,
  Copy,
} from "lucide-react";

type P = SVGProps<SVGSVGElement>;

/* ------------------------------------------------------------------ */
/* Brand assets (custom, not from an icon set)                         */
/* ------------------------------------------------------------------ */

export const LogoMark = ({ className = "" }: { className?: string }) => (
  <svg viewBox="0 0 32 32" className={className} aria-hidden="true">
    <defs>
      <linearGradient id="cflg" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stopColor="#FB923C" />
        <stop offset="1" stopColor="#EA580C" />
      </linearGradient>
    </defs>
    <rect width="32" height="32" rx="9" fill="url(#cflg)" />
    <path
      d="M21.2 10.1a7 7 0 1 0 0 11.8"
      fill="none"
      stroke="#fff"
      strokeWidth="3.6"
      strokeLinecap="round"
    />
  </svg>
);

/** Full brand lockup: mark + "Coachify." wordmark */
export const BrandLockup = ({
  className = "",
  markClassName = "h-8 w-8",
  textClassName = "text-[19px]",
  dark = false,
}: {
  className?: string;
  markClassName?: string;
  textClassName?: string;
  dark?: boolean;
}) => (
  <span className={cn("inline-flex items-center gap-2.5", className)}>
    <LogoMark className={markClassName} />
    <span
      className={cn(
        "font-display font-extrabold tracking-tight",
        dark ? "text-white" : "text-ink",
        textClassName,
      )}
    >
      Coachify<span className="text-orange-500">.</span>
    </span>
  </span>
);

/* ------------------------------------------------------------------ */
/* UI icons — all from Lucide                                          */
/* ------------------------------------------------------------------ */

export const SparkIcon = (p: P) => <Sparkles strokeWidth={1.7} {...p} />;
export const WifiOffIcon = (p: P) => <WifiOff strokeWidth={1.7} {...p} />;
export const MonitorIcon = (p: P) => <Monitor strokeWidth={1.7} {...p} />;
export const LanIcon = (p: P) => <Network strokeWidth={1.7} {...p} />;
export const BanknoteIcon = (p: P) => <Banknote strokeWidth={1.7} {...p} />;
export const AttendanceIcon = (p: P) => (
  <ClipboardCheck strokeWidth={1.7} {...p} />
);
export const ExamIcon = (p: P) => <FileText strokeWidth={1.7} {...p} />;
export const BellIcon = (p: P) => <Bell strokeWidth={1.7} {...p} />;
export const ShieldIcon = (p: P) => <ShieldCheck strokeWidth={1.7} {...p} />;
export const DatabaseIcon = (p: P) => <Database strokeWidth={1.7} {...p} />;
export const UsersIcon = (p: P) => <Users strokeWidth={1.7} {...p} />;
export const PrinterIcon = (p: P) => <Printer strokeWidth={1.7} {...p} />;
export const DownloadIcon = (p: P) => <Download strokeWidth={1.7} {...p} />;
export const ArrowRightIcon = (p: P) => <ArrowRight strokeWidth={1.9} {...p} />;
export const CheckIcon = (p: P) => <Check strokeWidth={2.6} {...p} />;
export const XIcon = (p: P) => <X strokeWidth={2.1} {...p} />;
export const ChevronDownIcon = (p: P) => (
  <ChevronDown strokeWidth={1.9} {...p} />
);
export const PhoneIcon = (p: P) => <Phone strokeWidth={1.7} {...p} />;
export const MailIcon = (p: P) => <Mail strokeWidth={1.7} {...p} />;
export const MapPinIcon = (p: P) => <MapPin strokeWidth={1.7} {...p} />;
export const MenuIcon = (p: P) => <Menu strokeWidth={2} {...p} />;
export const CalendarIcon = (p: P) => <Calendar strokeWidth={1.7} {...p} />;
export const ChartIcon = (p: P) => <BarChart3 strokeWidth={1.7} {...p} />;
export const ClockIcon = (p: P) => <Clock strokeWidth={1.7} {...p} />;
export const MessageIcon = (p: P) => <MessageSquare strokeWidth={1.7} {...p} />;
export const SearchIcon = (p: P) => <Search strokeWidth={1.9} {...p} />;
export const StarIcon = (p: P) => (
  <Star fill="currentColor" stroke="none" {...p} />
);
export const LockIcon = (p: P) => <Lock strokeWidth={1.7} {...p} />;
export const CopyIcon = (p: P) => <Copy strokeWidth={1.7} {...p} />;
export const FileTextIcon = (p: P) => <FileText strokeWidth={1.7} {...p} />;
