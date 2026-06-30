import type { ComponentProps, ReactNode } from "react";
import { Loader2, XIcon } from "lucide-react";
import { Alert, AlertAction, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

interface ErrorBannerProps {
  message: string;
  onDismiss?: () => void;
}

export function ErrorBanner({ message, onDismiss }: ErrorBannerProps) {
  return (
    <Alert variant="destructive" className="mb-4">
      <AlertDescription>{message}</AlertDescription>
      {onDismiss && (
        <AlertAction>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            onClick={onDismiss}
            aria-label="Close"
          >
            <XIcon />
          </Button>
        </AlertAction>
      )}
    </Alert>
  );
}

interface LoadingStateProps {
  message?: string;
}

export function LoadingState({ message = "Loading…" }: LoadingStateProps) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16">
      <Skeleton className="h-4 w-32" />
      <p className="text-sm text-muted-foreground">{message}</p>
    </div>
  );
}

interface ProgressBarProps {
  value?: number;
  className?: string;
}

export function ProgressBar({ value, className }: ProgressBarProps) {
  const normalized =
    typeof value === "number" ? Math.min(100, Math.max(0, value)) : undefined;

  return (
    <div
      className={cn(
        "h-1.5 overflow-hidden rounded-full bg-muted",
        className,
      )}
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={normalized}
    >
      <div
        className={cn(
          "h-full rounded-full bg-primary transition-all",
          normalized === undefined && "w-1/2 animate-pulse",
        )}
        style={normalized === undefined ? undefined : { width: `${normalized}%` }}
      />
    </div>
  );
}

interface OperationStatusProps {
  message: string;
  detail?: ReactNode;
  value?: number;
  className?: string;
}

export function OperationStatus({
  message,
  detail,
  value,
  className,
}: OperationStatusProps) {
  return (
    <div className={cn("space-y-2", className)}>
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Loader2 className="size-3.5 animate-spin" />
        <span>{message}</span>
      </div>
      <ProgressBar value={value} />
      {detail && <div className="text-xs text-muted-foreground">{detail}</div>}
    </div>
  );
}

type LoadingButtonProps = ComponentProps<typeof Button> & {
  loading?: boolean;
  loadingLabel?: ReactNode;
};

export function LoadingButton({
  loading,
  loadingLabel,
  children,
  disabled,
  ...props
}: LoadingButtonProps) {
  return (
    <Button disabled={disabled || loading} {...props}>
      {loading && <Loader2 className="animate-spin" />}
      {loading ? (loadingLabel ?? children) : children}
    </Button>
  );
}

interface EmptyStateProps {
  title: string;
  description?: string;
  action?: ReactNode;
}

export function EmptyState({ title, description, action }: EmptyStateProps) {
  return (
    <Card className="border-dashed py-16 text-center shadow-none">
      <CardHeader className="items-center">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      {action && <CardContent>{action}</CardContent>}
    </Card>
  );
}
