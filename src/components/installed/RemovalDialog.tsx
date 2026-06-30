import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { OperationStatus } from "../Feedback";
import { getClientLabel } from "@/lib/clients";
import type { ExistingServer } from "../../types";

export type PendingRemoval =
  | {
      kind: "integration";
      id: string;
      title: string;
      description: string;
    }
  | {
      kind: "server";
      server: ExistingServer;
      title: string;
      description: string;
    }
  | {
      kind: "plugin";
      id: string;
      title: string;
      description: string;
    };

type RemovalClient = {
  client_id: string;
  enabled: boolean;
};

interface RemovalDialogProps {
  pendingRemoval: PendingRemoval | null;
  removalClients: RemovalClient[];
  selectedRemovalClients: string[];
  removalTargetsLoading: boolean;
  onClose: () => void;
  onConfirm: () => void;
  onToggleClient: (clientId: string, checked: boolean) => void;
}

export function RemovalDialog({
  pendingRemoval,
  removalClients,
  selectedRemovalClients,
  removalTargetsLoading,
  onClose,
  onConfirm,
  onToggleClient,
}: RemovalDialogProps) {
  return (
    <Dialog open={pendingRemoval !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{pendingRemoval?.title}</DialogTitle>
          <DialogDescription>{pendingRemoval?.description}</DialogDescription>
        </DialogHeader>
        <div className="space-y-2">
          {removalTargetsLoading ? (
            <OperationStatus
              message="Loading clients…"
              detail="Finding where this is installed"
            />
          ) : removalClients.length > 0 ? (
            removalClients.map((client) => {
              const checkboxId = `remove-${pendingRemoval?.kind}-${client.client_id}`;
              return (
                <label
                  key={client.client_id}
                  htmlFor={checkboxId}
                  className="flex items-center gap-3 rounded-md border border-border px-3 py-2 text-sm"
                >
                  <Checkbox
                    id={checkboxId}
                    checked={selectedRemovalClients.includes(client.client_id)}
                    onCheckedChange={(checked) =>
                      onToggleClient(client.client_id, checked === true)
                    }
                  />
                  <span className="min-w-0 flex-1">
                    {getClientLabel(client.client_id)}
                  </span>
                </label>
              );
            })
          ) : (
            <p className="rounded-md border border-border px-3 py-2 text-sm text-muted-foreground">
              There are no active clients to remove from.
            </p>
          )}
        </div>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="button"
            variant="destructive"
            disabled={
              removalTargetsLoading || selectedRemovalClients.length === 0
            }
            onClick={onConfirm}
          >
            Remove
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
