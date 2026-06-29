import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const STATUS_LABELS: Record<
  string,
  { label: string; variant: "default" | "secondary" | "destructive" | "outline"; className?: string }
> = {
  connected: {
    label: "Connected",
    variant: "default",
    className:
      "bg-emerald-500/10 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400",
  },
  needs_secret: {
    label: "API Key required",
    variant: "secondary",
    className:
      "bg-amber-500/10 text-amber-700 dark:bg-amber-500/20 dark:text-amber-400",
  },
  error: {
    label: "Error",
    variant: "destructive",
  },
  disabled: {
    label: "Disabled",
    variant: "outline",
  },
  installed: {
    label: "Installed",
    variant: "default",
    className:
      "bg-emerald-500/10 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400",
  },
  update_available: {
    label: "Update available",
    variant: "secondary",
    className:
      "bg-blue-500/10 text-blue-700 dark:bg-blue-500/20 dark:text-blue-400",
  },
};

interface StatusBadgeProps {
  status: string;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const config = STATUS_LABELS[status] ?? {
    label: status,
    variant: "outline" as const,
  };

  return (
    <Badge variant={config.variant} className={cn(config.className)}>
      {config.label}
    </Badge>
  );
}
