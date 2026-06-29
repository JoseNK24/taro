import { Badge } from "@/components/ui/badge";
import type { HarnessSnapshot } from "../../types";

interface HarnessProbeBadgeProps {
  snapshot?: HarnessSnapshot;
}

export function HarnessProbeBadge({ snapshot }: HarnessProbeBadgeProps) {
  if (!snapshot) {
    return <Badge variant="outline">Unknown</Badge>;
  }

  if (!snapshot.detected) {
    return (
      <Badge variant="outline" className="text-muted-foreground">
        Not found
      </Badge>
    );
  }

  return (
    <div className="flex flex-wrap gap-1">
      <Badge className="bg-emerald-500/10 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400">
        Detected
      </Badge>
      {snapshot.agent_capable && (
        <Badge variant="secondary">Agent capable</Badge>
      )}
      {snapshot.auth_status === "unauthenticated" && (
        <Badge variant="outline" className="text-amber-700 dark:text-amber-400">
          Not authenticated
        </Badge>
      )}
    </div>
  );
}
