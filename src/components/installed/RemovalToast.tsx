import type { ReactNode } from "react";
import { CheckCircle2, Loader2, XCircle, XIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getClientLabel } from "@/lib/clients";
import type { ClientOperationResult } from "../../types";

export type RemovalNotification = {
  status: "pending" | "success" | "error";
  title: string;
  message: string;
  clientResults?: ClientOperationResult[];
};

interface RemovalToastProps {
  notification: RemovalNotification;
  onDismiss: () => void;
}

export function RemovalToast({ notification, onDismiss }: RemovalToastProps) {
  const icon: ReactNode =
    notification.status === "pending" ? (
      <Loader2 className="mt-0.5 size-4 animate-spin text-primary" />
    ) : notification.status === "success" ? (
      <CheckCircle2 className="mt-0.5 size-4 text-emerald-600" />
    ) : (
      <XCircle className="mt-0.5 size-4 text-destructive" />
    );

  return (
    <div className="animate-removal-toast-in fixed right-5 bottom-5 z-50 w-[min(360px,calc(100vw-2.5rem))] rounded-lg border border-border bg-background p-4 shadow-lg">
      <div className="flex items-start gap-3">
        {icon}
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium text-foreground">
            {notification.title}
          </p>
          <p className="mt-1 text-sm text-muted-foreground">
            {notification.message}
          </p>
          {notification.clientResults &&
            notification.clientResults.length > 0 && (
              <ul className="mt-2 space-y-1 text-xs text-muted-foreground">
                {notification.clientResults.map((client) => (
                  <li key={client.client_id}>
                    {getClientLabel(client.client_id)}: {client.message}
                  </li>
                ))}
              </ul>
            )}
        </div>
        {notification.status !== "pending" && (
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            onClick={onDismiss}
            aria-label="Close notification"
          >
            <XIcon />
          </Button>
        )}
      </div>
    </div>
  );
}
