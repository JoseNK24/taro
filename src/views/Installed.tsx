import { useCallback, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { StatusBadge } from "../components/StatusBadge";
import {
  ErrorBanner,
  EmptyState,
  LoadingButton,
  LoadingState,
  OperationStatus,
} from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import {
  getCatalog,
  getCommunityInstallMeta,
  getDiscoveredMcp,
  getInstallations,
  getPluginCatalog,
  getPluginInstallations,
  forceRemoveMcpServer,
  runHealthCheck,
  scanMcpServers,
  toggleInstallation,
  togglePluginInstallation,
  uninstallIntegration,
  uninstallPlugin,
} from "../hooks/useTauri";
import { getClientLabel } from "../lib/clients";
import type {
  ClientOperationResult,
  CommunityInstallMeta,
  ExistingServer,
  InstallationRecord,
  IntegrationCatalogEntry,
  PluginInstallationRecord,
  PluginCatalogEntry,
} from "../types";

type InstalledTab = "mcp" | "plugins";
type RowOperation = {
  message: string;
  detail?: string;
};
type RowResult = {
  success: boolean;
  message: string;
  clientResults?: ClientOperationResult[];
};

export function Installed() {
  const [tab, setTab] = useState<InstalledTab>("mcp");
  const [installations, setInstallations] = useState<InstallationRecord[]>([]);
  const [pluginInstallations, setPluginInstallations] = useState<
    PluginInstallationRecord[]
  >([]);
  const [catalog, setCatalog] = useState<IntegrationCatalogEntry[]>([]);
  const [pluginCatalog, setPluginCatalog] = useState<PluginCatalogEntry[]>([]);
  const [communityMeta, setCommunityMeta] = useState<
    Record<string, CommunityInstallMeta>
  >({});
  const [communityNames, setCommunityNames] = useState<Record<string, string>>({});
  const [allServers, setAllServers] = useState<ExistingServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [operations, setOperations] = useState<Record<string, RowOperation>>({});
  const [results, setResults] = useState<Record<string, RowResult>>({});
  const [notice, setNotice] = useState<RowResult | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [inst, cat, pluginInst, pluginCat, servers] = await Promise.all([
        getInstallations(),
        getCatalog(),
        getPluginInstallations(),
        getPluginCatalog(),
        scanMcpServers(),
      ]);
      setInstallations(inst);
      setCatalog(cat);
      setPluginInstallations(pluginInst);
      setPluginCatalog(pluginCat);
      setAllServers(servers);
      const metaEntries = await Promise.all(
        inst
          .filter((i) => i.source === "community")
          .map(async (i) => {
            const meta = await getCommunityInstallMeta(i.id);
            return meta ? ([i.id, meta] as const) : null;
          }),
      );
      const metaMap = Object.fromEntries(
        metaEntries.filter((e): e is [string, CommunityInstallMeta] => e !== null),
      );
      setCommunityMeta(metaMap);
      const names: Record<string, string> = {};
      await Promise.all(
        Object.values(metaMap).map(async (meta) => {
          const discovered = await getDiscoveredMcp(meta.discovered_mcp_id);
          if (discovered) {
            names[meta.installation_id] = discovered.name;
          }
        }),
      );
      setCommunityNames(names);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const catalogMap = Object.fromEntries(catalog.map((c) => [c.id, c]));
  const pluginCatalogMap = Object.fromEntries(
    pluginCatalog.map((c) => [c.id, c]),
  );

  const setOperation = (id: string, operation: RowOperation | null) => {
    setOperations((prev) => {
      const next = { ...prev };
      if (operation) next[id] = operation;
      else delete next[id];
      return next;
    });
  };

  const setRowResult = (id: string, result: RowResult | null) => {
    setResults((prev) => {
      const next = { ...prev };
      if (result) next[id] = result;
      else delete next[id];
      return next;
    });
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    setOperation(id, {
      message: enabled ? "Enabling…" : "Disabling…",
      detail: "Syncing client configuration",
    });
    setRowResult(id, null);
    setNotice(null);
    try {
      await toggleInstallation(id, enabled);
      await load();
      setRowResult(id, {
        success: true,
        message: enabled ? "Integration enabled" : "Integration disabled",
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setOperation(id, null);
    }
  };

  const handleRemove = async (id: string) => {
    if (
      !confirm(
        "Remove this integration? It will be removed from configured clients.",
      )
    ) {
      return;
    }
    setOperation(id, {
      message: "Removing…",
      detail: "Removing MCP configuration from enabled clients",
    });
    setRowResult(id, null);
    setNotice(null);
    try {
      const result = await uninstallIntegration(id);
      await load();
      setRowResult(id, {
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
      setNotice({
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setOperation(id, null);
    }
  };

  const handleForceRemove = async (server: ExistingServer) => {
    const rowId = `${server.client_id}:${server.server_id}`;
    const warning = server.managed
      ? `Remove '${server.server_id}' from ${server.client_name}?`
      : `'${server.server_id}' was not installed by Taro. Remove it from ${server.client_name} anyway?`;
    if (!confirm(warning)) {
      return;
    }
    setOperation(rowId, {
      message: "Removing…",
      detail: `Removing ${server.server_id} from ${server.client_name}`,
    });
    setNotice(null);
    try {
      const result = await forceRemoveMcpServer(
        server.client_id,
        server.server_id,
      );
      await load();
      setNotice({
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setOperation(rowId, null);
    }
  };

  const handleHealth = async (id: string) => {
    setOperation(id, {
      message: "Checking…",
      detail: "Running a health check against this MCP server",
    });
    setRowResult(id, null);
    setNotice(null);
    try {
      const result = await runHealthCheck(id);
      await load();
      setRowResult(id, {
        success: result.ok,
        message: result.ok
          ? "Health check passed"
          : result.detail ?? "Health check failed",
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setOperation(id, null);
    }
  };

  const handlePluginToggle = async (id: string, enabled: boolean) => {
    setOperation(id, {
      message: enabled ? "Enabling…" : "Disabling…",
      detail: "Syncing plugin state with enabled clients",
    });
    setRowResult(id, null);
    setNotice(null);
    try {
      await togglePluginInstallation(id, enabled);
      await load();
      setRowResult(id, {
        success: true,
        message: enabled ? "Plugin enabled" : "Plugin disabled",
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setOperation(id, null);
    }
  };

  const handlePluginRemove = async (id: string) => {
    if (
      !confirm(
        "Remove this plugin? It will be uninstalled from configured clients.",
      )
    ) {
      return;
    }
    setOperation(id, {
      message: "Removing…",
      detail: "Uninstalling plugin from enabled clients",
    });
    setRowResult(id, null);
    setNotice(null);
    try {
      const result = await uninstallPlugin(id);
      await load();
      setRowResult(id, {
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
      setNotice({
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setOperation(id, null);
    }
  };

  if (loading) return <LoadingState />;

  const renderResult = (result: RowResult) => (
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

  return (
    <div>
      <PageHeader
        title="Installed"
        description="Manage active integrations and plugins on your system."
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}
      {notice && (
        <div className="mb-4">
          {renderResult(notice)}
        </div>
      )}

      <div className="mb-6 flex w-fit gap-1 rounded-lg border border-border p-1">
        <Button
          type="button"
          variant={tab === "mcp" ? "default" : "ghost"}
          size="sm"
          onClick={() => setTab("mcp")}
        >
          MCP
        </Button>
        <Button
          type="button"
          variant={tab === "plugins" ? "default" : "ghost"}
          size="sm"
          onClick={() => setTab("plugins")}
        >
          Plugins
        </Button>
      </div>

      {tab === "mcp" && (
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
                          {new Date(inst.installed_at).toLocaleDateString(
                            "en-US",
                          )}
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
                          loading={operation?.message === "Enabling…" || operation?.message === "Disabling…"}
                          loadingLabel={inst.enabled ? "Disabling…" : "Enabling…"}
                          disabled={busy}
                          onClick={() => handleToggle(inst.id, !inst.enabled)}
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
                          onClick={() => handleHealth(inst.id)}
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
                          onClick={() => handleRemove(inst.id)}
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
                Every MCP server found in your clients' configs. You can remove
                any of them, including servers Taro did not install.
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
                          onClick={() => handleForceRemove(server)}
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
      )}

      {tab === "plugins" && (
        <>
          {pluginInstallations.length === 0 ? (
            <EmptyState
              title="No plugins installed"
              description="Browse the Plugins section to install skills and agent rules."
            />
          ) : (
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
                          {new Date(inst.installed_at).toLocaleDateString(
                            "en-US",
                          )}
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
                          loading={operation?.message === "Enabling…" || operation?.message === "Disabling…"}
                          loadingLabel={inst.enabled ? "Disabling…" : "Enabling…"}
                          disabled={busy}
                          onClick={() =>
                            handlePluginToggle(inst.id, !inst.enabled)
                          }
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
                          onClick={() => handlePluginRemove(inst.id)}
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
        </>
      )}
    </div>
  );
}
