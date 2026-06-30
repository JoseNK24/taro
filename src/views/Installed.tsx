import { useCallback, useEffect, useState } from "react";
import { ErrorBanner, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/PageHeader";
import { SegmentedTabs } from "../components/SegmentedTabs";
import { HealthPanel } from "../components/installed/HealthPanel";
import { InstalledMcpTab } from "../components/installed/InstalledMcpTab";
import { InstalledPluginsTab } from "../components/installed/InstalledPluginsTab";
import {
  RemovalDialog,
  type PendingRemoval,
} from "../components/installed/RemovalDialog";
import {
  RemovalToast,
  type RemovalNotification,
} from "../components/installed/RemovalToast";
import {
  getCatalog,
  getClientTargets,
  getInstallations,
  getPluginCatalog,
  getPluginClientTargets,
  getPluginInstallations,
  forceRemoveMcpServer,
  listCommunityInstallDetails,
  runHealthCheck,
  scanMcpServers,
  toggleInstallation,
  togglePluginInstallation,
  uninstallIntegrationFromClients,
  uninstallPluginFromClients,
} from "../hooks/useTauri";
import { type ClientTargetRecord, type ClientOperationResult, type CommunityInstallMeta, type ExistingServer, type InstallationRecord, type IntegrationCatalogEntry, type PluginClientTargetRecord, type PluginCatalogEntry, type PluginInstallationRecord } from "../types";

type InstalledTab = "mcp" | "plugins" | "health";
type RowOperation = { message: string; detail?: string };
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
  const [removalNotification, setRemovalNotification] =
    useState<RemovalNotification | null>(null);
  const [pendingRemoval, setPendingRemoval] = useState<PendingRemoval | null>(
    null,
  );
  const [removalClients, setRemovalClients] = useState<
    Array<{ client_id: string; enabled: boolean }>
  >([]);
  const [selectedRemovalClients, setSelectedRemovalClients] = useState<
    string[]
  >([]);
  const [removalTargetsLoading, setRemovalTargetsLoading] = useState(false);

  const load = useCallback(async (showLoading = true) => {
    if (showLoading) setLoading(true);
    setError(null);
    try {
      const [inst, cat, pluginInst, pluginCat, servers, communityDetails] =
        await Promise.all([
          getInstallations(),
          getCatalog(),
          getPluginInstallations(),
          getPluginCatalog(),
          scanMcpServers(),
          listCommunityInstallDetails(),
        ]);
      setInstallations(inst);
      setCatalog(cat);
      setPluginInstallations(pluginInst);
      setPluginCatalog(pluginCat);
      setAllServers(servers);
      const metaMap: Record<string, CommunityInstallMeta> = {};
      const names: Record<string, string> = {};
      for (const detail of communityDetails) {
        metaMap[detail.installation_id] = detail.meta;
        names[detail.installation_id] = detail.discovered_name;
      }
      setCommunityMeta(metaMap);
      setCommunityNames(names);
    } catch (e) {
      if (!showLoading) throw e;
      setError(String(e));
    } finally {
      if (showLoading) setLoading(false);
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

  const showRemovalStarted = (message: string) => {
    setRemovalNotification({
      status: "pending",
      title: "Removal in progress",
      message,
    });
  };

  const showRemovalFinished = (
    result: RowResult,
    successMessage = "Uninstall complete and list updated.",
  ) => {
    setRemovalNotification({
      status: result.success ? "success" : "error",
      title: result.success ? "Removal complete" : "Could not remove",
      message: result.success ? successMessage : result.message,
      clientResults: result.clientResults,
    });
  };

  const resetPendingRemoval = () => {
    setPendingRemoval(null);
    setRemovalClients([]);
    setSelectedRemovalClients([]);
    setRemovalTargetsLoading(false);
  };

  const prepareRemovalClients = async (
    loadTargets: () => Promise<
      Array<ClientTargetRecord | PluginClientTargetRecord>
    >,
  ) => {
    setRemovalTargetsLoading(true);
    setRemovalClients([]);
    setSelectedRemovalClients([]);
    try {
      const targets = await loadTargets();
      const enabledTargets = targets.filter((target) => target.enabled);
      setRemovalClients(
        enabledTargets.map((target) => ({
          client_id: target.client_id,
          enabled: target.enabled,
        })),
      );
      setSelectedRemovalClients(
        enabledTargets.map((target) => target.client_id),
      );
    } catch (e) {
      const message = String(e);
      setError(message);
      showRemovalFinished({ success: false, message });
    } finally {
      setRemovalTargetsLoading(false);
    }
  };

  const toggleRemovalClient = (clientId: string, checked: boolean) => {
    setSelectedRemovalClients((prev) =>
      checked
        ? Array.from(new Set([...prev, clientId]))
        : prev.filter((id) => id !== clientId),
    );
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    setOperation(id, {
      message: enabled ? "Enabling…" : "Disabling…",
      detail: "Syncing client configuration",
    });
    setRowResult(id, null);
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
    setPendingRemoval({
      kind: "integration",
      id,
      title: "Remove integration",
      description: "Choose which clients should remove this integration.",
    });
    await prepareRemovalClients(() => getClientTargets(id));
  };

  const removeIntegration = async (id: string, clientIds: string[]) => {
    setOperation(id, {
      message: "Removing…",
      detail: "Removing MCP configuration from selected clients",
    });
    setRowResult(id, null);
    showRemovalStarted("Uninstalling the integration from selected clients.");
    try {
      const result = await uninstallIntegrationFromClients(id, clientIds);
      await load(false);
      setRowResult(id, {
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
      showRemovalFinished({
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
    } catch (e) {
      const message = String(e);
      setError(message);
      showRemovalFinished({ success: false, message });
    } finally {
      setOperation(id, null);
    }
  };

  const handleForceRemove = async (server: ExistingServer) => {
    setPendingRemoval({
      kind: "server",
      server,
      title: "Remove MCP server",
      description: server.managed
        ? `'${server.server_id}' will be removed from ${server.client_name}.`
        : `'${server.server_id}' was not installed by Taro. It will still be removed from ${server.client_name}.`,
    });
    setRemovalClients([{ client_id: server.client_id, enabled: true }]);
    setSelectedRemovalClients([server.client_id]);
  };

  const forceRemoveServer = async (server: ExistingServer) => {
    const rowId = `${server.client_id}:${server.server_id}`;
    setOperation(rowId, {
      message: "Removing…",
      detail: `Removing ${server.server_id} from ${server.client_name}`,
    });
    showRemovalStarted(`Removing '${server.server_id}' from ${server.client_name}.`);
    try {
      const result = await forceRemoveMcpServer(
        server.client_id,
        server.server_id,
      );
      await load(false);
      showRemovalFinished(
        {
          success: result.success,
          message: result.message,
          clientResults: result.client_results,
        },
        "Server removed and list updated.",
      );
    } catch (e) {
      const message = String(e);
      setError(message);
      showRemovalFinished({ success: false, message });
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
    setPendingRemoval({
      kind: "plugin",
      id,
      title: "Remove plugin",
      description: "Choose which clients should uninstall this plugin.",
    });
    await prepareRemovalClients(() => getPluginClientTargets(id));
  };

  const removePlugin = async (id: string, clientIds: string[]) => {
    setOperation(id, {
      message: "Removing…",
      detail: "Uninstalling plugin from selected clients",
    });
    setRowResult(id, null);
    showRemovalStarted("Uninstalling the plugin from selected clients.");
    try {
      const result = await uninstallPluginFromClients(id, clientIds);
      await load(false);
      setRowResult(id, {
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
      showRemovalFinished({
        success: result.success,
        message: result.message,
        clientResults: result.client_results,
      });
    } catch (e) {
      const message = String(e);
      setError(message);
      showRemovalFinished({ success: false, message });
    } finally {
      setOperation(id, null);
    }
  };

  const confirmPendingRemoval = async () => {
    const removal = pendingRemoval;
    if (!removal) return;
    const clientIds = selectedRemovalClients;
    resetPendingRemoval();
    if (removal.kind === "integration") {
      await removeIntegration(removal.id, clientIds);
    } else if (removal.kind === "server") {
      await forceRemoveServer(removal.server);
    } else {
      await removePlugin(removal.id, clientIds);
    }
  };

  if (loading) return <LoadingState />;

  return (
    <div>
      <PageHeader
        title="Installed"
        description="Manage active integrations and plugins on your system."
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}
      {removalNotification && (
        <RemovalToast
          notification={removalNotification}
          onDismiss={() => setRemovalNotification(null)}
        />
      )}

      <RemovalDialog
        pendingRemoval={pendingRemoval}
        removalClients={removalClients}
        selectedRemovalClients={selectedRemovalClients}
        removalTargetsLoading={removalTargetsLoading}
        onClose={resetPendingRemoval}
        onConfirm={() => void confirmPendingRemoval()}
        onToggleClient={toggleRemovalClient}
      />

      <SegmentedTabs
        value={tab}
        onChange={setTab}
        items={[
          { id: "mcp", label: "MCP" },
          { id: "plugins", label: "Plugins" },
          { id: "health", label: "Health" },
        ]}
      />

      {tab === "mcp" && (
        <InstalledMcpTab
          installations={installations}
          catalogMap={catalogMap}
          communityMeta={communityMeta}
          communityNames={communityNames}
          allServers={allServers}
          operations={operations}
          results={results}
          onToggle={handleToggle}
          onHealth={handleHealth}
          onRemove={handleRemove}
          onForceRemove={handleForceRemove}
        />
      )}

      {tab === "plugins" && (
        <InstalledPluginsTab
          pluginInstallations={pluginInstallations}
          pluginCatalogMap={pluginCatalogMap}
          operations={operations}
          results={results}
          onToggle={handlePluginToggle}
          onRemove={handlePluginRemove}
        />
      )}

      {tab === "health" && <HealthPanel />}
    </div>
  );
}
