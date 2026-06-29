import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { ClientName } from "@/components/ClientLogo";
import type { HarnessInstanceRecord, HarnessSnapshot } from "../../types";
import { HarnessProbeBadge } from "./HarnessProbeBadge";

interface HarnessInstanceCardProps {
  instance: HarnessInstanceRecord;
  snapshot?: HarnessSnapshot;
  busy?: boolean;
  onToggleEnabled: (enabled: boolean) => void;
  onSetDefault: () => void;
  onDelete: () => void;
}

export function HarnessInstanceCard({
  instance,
  snapshot,
  busy,
  onToggleEnabled,
  onSetDefault,
  onDelete,
}: HarnessInstanceCardProps) {
  return (
    <Card>
      <CardContent className="flex flex-col gap-3 py-4 sm:flex-row sm:items-center">
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            <ClientName
              clientId=""
              driverKind={instance.driver_kind}
              name={instance.display_name}
              className="font-medium text-foreground"
            />
            <Badge variant="outline">{instance.driver_kind}</Badge>
            {instance.is_default_install_agent && (
              <Badge variant="secondary">Default install agent</Badge>
            )}
          </div>
          <HarnessProbeBadge snapshot={snapshot} />
          {snapshot?.version && (
            <p className="text-xs text-muted-foreground">v{snapshot.version}</p>
          )}
          {snapshot?.probe_detail && (
            <p className="text-xs text-muted-foreground">{snapshot.probe_detail}</p>
          )}
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() => onToggleEnabled(!instance.enabled)}
          >
            {instance.enabled ? "Disable" : "Enable"}
          </Button>
          {!instance.is_default_install_agent && instance.enabled && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={busy || !snapshot?.agent_capable}
              onClick={onSetDefault}
            >
              Set as default
            </Button>
          )}
          <Button
            type="button"
            variant="destructive"
            size="sm"
            disabled={busy}
            onClick={onDelete}
          >
            Remove
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
