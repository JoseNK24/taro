import { useCallback, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { StatusBadge } from "../components/StatusBadge";
import { ErrorBanner, EmptyState, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import {
  getCatalog,
  getCommunityInstallMeta,
  getDiscoveredMcp,
  getInstallations,
  getPluginCatalog,
  getPluginInstallations,
  runHealthCheck,
  toggleInstallation,
  togglePluginInstallation,
  uninstallIntegration,
  uninstallPlugin,
} from "../hooks/useTauri";
import type {
  CommunityInstallMeta,
  InstallationRecord,
  IntegrationCatalogEntry,
  PluginInstallationRecord,
  PluginCatalogEntry,
} from "../types";

type InstalledTab = "mcp" | "plugins";

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
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [inst, cat, pluginInst, pluginCat] = await Promise.all([
        getInstallations(),
        getCatalog(),
        getPluginInstallations(),
        getPluginCatalog(),
      ]);
      setInstallations(inst);
      setCatalog(cat);
      setPluginInstallations(pluginInst);
      setPluginCatalog(pluginCat);
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

  const handleToggle = async (id: string, enabled: boolean) => {
    setBusyId(id);
    try {
      await toggleInstallation(id, enabled);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
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
    setBusyId(id);
    try {
      await uninstallIntegration(id);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
    }
  };

  const handleHealth = async (id: string) => {
    setBusyId(id);
    try {
      await runHealthCheck(id);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
    }
  };

  const handlePluginToggle = async (id: string, enabled: boolean) => {
    setBusyId(id);
    try {
      await togglePluginInstallation(id, enabled);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
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
    setBusyId(id);
    try {
      await uninstallPlugin(id);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
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
                      </div>
                      <div className="flex shrink-0 items-center gap-2">
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          disabled={busyId === inst.id}
                          onClick={() => handleToggle(inst.id, !inst.enabled)}
                        >
                          {inst.enabled ? "Disable" : "Enable"}
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          disabled={busyId === inst.id}
                          onClick={() => handleHealth(inst.id)}
                        >
                          Check
                        </Button>
                        <Button
                          type="button"
                          variant="destructive"
                          size="sm"
                          disabled={busyId === inst.id}
                          onClick={() => handleRemove(inst.id)}
                        >
                          Remove
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                );
              })}
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
                      </div>
                      <div className="flex shrink-0 items-center gap-2">
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          disabled={busyId === inst.id}
                          onClick={() =>
                            handlePluginToggle(inst.id, !inst.enabled)
                          }
                        >
                          {inst.enabled ? "Disable" : "Enable"}
                        </Button>
                        <Button
                          type="button"
                          variant="destructive"
                          size="sm"
                          disabled={busyId === inst.id}
                          onClick={() => handlePluginRemove(inst.id)}
                        >
                          Remove
                        </Button>
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
