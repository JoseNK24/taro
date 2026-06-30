import { Badge } from "@/components/ui/badge";
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
  CommunityInstallMeta,
  ExistingServer,
  InstallationRecord,
  IntegrationCatalogEntry,
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

interface InstalledMcpTabProps {
  installations: InstallationRecord[];
  catalogMap: Record<string, IntegrationCatalogEntry>;
  communityMeta: Record<string, CommunityInstallMeta>;
  communityNames: Record<string, string>;
  allServers: ExistingServer[];
  operations: Record<string, RowOperation>;
  results: Record<string, RowResult>;
  onToggle: (id: string, enabled: boolean) => void;
  onHealth: (id: string) => void;
  onRemove: (id: string) => void;
  onForceRemove: (server: ExistingServer) => void;
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

export function InstalledMcpTab({
  installations,
  catalogMap,
  communityMeta,
  communityNames,
  allServers,
  operations,
  results,
  onToggle,
  onHealth,
  onRemove,
  onForceRemove,
}: InstalledMcpTabProps) {
  return (
    <>
      {installations.length === 0 ? (
        <EmptyState
          title="No integrations installed"
          description="Browse the catalog in Discover to install your first integration."
        />
      ) : (
        <div className="space-y-3">
          {installations.map((inst) => {
            const entry = catalogMap[inst.integration_id];
            const isCommunity = inst.source === "community";
            const meta = communityMeta[inst.id];
            const operation = operations[inst.id];
            const result = results[inst.id];
            const busy = Boolean(operation);
            const displayName = isCommunity
              ? communityNames[inst.id] ??
                inst.integration_id.replace("community-", "")
              : entry?.name ?? inst.integration_id;
            return (
              <Card key={inst.id}>
                <CardContent className="flex items-center gap-4 py-4">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <h3 className="font-medium text-foreground">
                        {displayName}
                      </h3>
                      <StatusBadge status={inst.status} />
                      <Badge variant="outline">
                        {isCommunity ? "Community" : "Curated"}
                      </Badge>
                    </div>
                    {isCommunity && meta && (
                      <p className="mt-1 text-xs text-muted-foreground">
                        {meta.resolved_server.command}{" "}
                        {meta.resolved_server.args.join(" ")}
                      </p>
                    )}
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
                      variant="outline"
                      size="sm"
                      loading={operation?.message === "Checking…"}
                      loadingLabel="Checking…"
                      disabled={busy}
                      onClick={() => onHealth(inst.id)}
                    >
                      Check
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
      )}

      {allServers.length > 0 && (
        <div className="mt-8">
          <h3 className="mb-1 text-sm font-medium text-foreground">
            MCP servers on this system
          </h3>
          <p className="mb-3 text-xs text-muted-foreground">
            Every MCP server found in your clients' configs. You can remove any
            of them, including servers Taro did not install.
          </p>
          <div className="space-y-2">
            {allServers.map((server) => {
              const rowId = `${server.client_id}:${server.server_id}`;
              const operation = operations[rowId];
              return (
                <Card key={rowId}>
                  <CardContent className="flex items-center gap-4 py-3">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <h4 className="truncate font-medium text-foreground">
                          {server.server_id}
                        </h4>
                        <Badge
                          variant={server.managed ? "default" : "outline"}
                        >
                          {server.managed ? "Installed by Taro" : "External"}
                        </Badge>
                      </div>
                      <p className="mt-0.5 truncate text-xs text-muted-foreground">
                        {getClientLabel(server.client_id)} · {server.command}
                      </p>
                      {operation && (
                        <OperationStatus
                          className="mt-2"
                          message={operation.message}
                          detail={operation.detail}
                        />
                      )}
                    </div>
                    <LoadingButton
                      type="button"
                      variant="destructive"
                      size="sm"
                      loading={Boolean(operation)}
                      loadingLabel="Removing…"
                      disabled={Boolean(operation)}
                      onClick={() => onForceRemove(server)}
                    >
                      Remove
                    </LoadingButton>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </div>
      )}
    </>
  );
}
