import { Card, CardContent } from "@/components/ui/card";
import { StatusBadge } from "../StatusBadge";
import {
  EmptyState,
  LoadingButton,
  OperationStatus,
} from "../Feedback";
import { getClientLabel } from "@/lib/clients";
import type {
  ClientOperationResult,
  PluginCatalogEntry,
  PluginInstallationRecord,
} from "../../types";

type RowOperation = {
  message: string;
  detail?: string;
};

type RowResult = {
  success: boolean;
  message: string;
  clientResults?: ClientOperationResult[];
};

interface InstalledPluginsTabProps {
  pluginInstallations: PluginInstallationRecord[];
  pluginCatalogMap: Record<string, PluginCatalogEntry>;
  operations: Record<string, RowOperation>;
  results: Record<string, RowResult>;
  onToggle: (id: string, enabled: boolean) => void;
  onRemove: (id: string) => void;
}

function renderResult(result: RowResult) {
  return (
    <div
      className={
        result.success
          ? "mt-2 rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-700 dark:text-emerald-400"
          : "mt-2 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive"
      }
    >
      <p>{result.message}</p>
      {result.clientResults && result.clientResults.length > 0 && (
        <ul className="mt-1 space-y-0.5">
          {result.clientResults.map((client) => (
            <li key={client.client_id}>
              {getClientLabel(client.client_id)}: {client.message}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function InstalledPluginsTab({
  pluginInstallations,
  pluginCatalogMap,
  operations,
  results,
  onToggle,
  onRemove,
}: InstalledPluginsTabProps) {
  if (pluginInstallations.length === 0) {
    return (
      <EmptyState
        title="No plugins installed"
        description="Browse the Plugins section to install skills and agent rules."
      />
    );
  }

  return (
    <div className="space-y-3">
      {pluginInstallations.map((inst) => {
        const entry = pluginCatalogMap[inst.plugin_id];
        const operation = operations[inst.id];
        const result = results[inst.id];
        const busy = Boolean(operation);
        return (
          <Card key={inst.id}>
            <CardContent className="flex items-center gap-4 py-4">
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <h3 className="font-medium text-foreground">
                    {entry?.name ?? inst.plugin_id}
                  </h3>
                  <StatusBadge status={inst.status} />
                </div>
                {inst.error_message && (
                  <p className="mt-1 text-xs text-destructive">
                    {inst.error_message}
                  </p>
                )}
                <p className="mt-0.5 text-xs text-muted-foreground">
                  Installed on{" "}
                  {new Date(inst.installed_at).toLocaleDateString("en-US")}
                </p>
                {operation && (
                  <OperationStatus
                    className="mt-3"
                    message={operation.message}
                    detail={operation.detail}
                  />
                )}
                {result && renderResult(result)}
              </div>
              <div className="flex shrink-0 items-center gap-2">
                <LoadingButton
                  type="button"
                  variant="outline"
                  size="sm"
                  loading={
                    operation?.message === "Enabling…" ||
                    operation?.message === "Disabling…"
                  }
                  loadingLabel={inst.enabled ? "Disabling…" : "Enabling…"}
                  disabled={busy}
                  onClick={() => onToggle(inst.id, !inst.enabled)}
                >
                  {inst.enabled ? "Disable" : "Enable"}
                </LoadingButton>
                <LoadingButton
                  type="button"
                  variant="destructive"
                  size="sm"
                  loading={operation?.message === "Removing…"}
                  loadingLabel="Removing…"
                  disabled={busy}
                  onClick={() => onRemove(inst.id)}
                >
                  Remove
                </LoadingButton>
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
