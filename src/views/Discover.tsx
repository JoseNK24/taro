import { useCallback, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { ClientName } from "@/components/ClientLogo";
import { DiscoveredMcpCard } from "../components/DiscoveredMcpCard";
import { DiscoverySearchBar } from "../components/DiscoverySearchBar";
import {
  ErrorBanner,
  LoadingButton,
  OperationStatus,
} from "../components/Feedback";
import { PluginsDiscoverPanel } from "../components/PluginsDiscoverPanel";
import { PageHeader } from "../components/Sidebar";
import {
  detectClients,
  getCatalog,
  getDiscoveryStatus,
  getInstallations,
  getSecretsStatus,
  installIntegration,
  saveSecret,
  searchDiscoveredMcps,
  syncDiscoveredCatalog,
} from "../hooks/useTauri";
import { filterSupportedClients } from "@/lib/clients";
import type {
  DetectionResult,
  DiscoveredMcpEntry,
  DiscoverySort,
  IntegrationCatalogEntry,
  SecretStatus,
} from "../types";

interface DiscoverProps {
  onInstalled: () => void;
  onOpenSettings?: () => void;
}

type WizardStep = "clients" | "secrets" | "installing" | "done";
type ProductTab = "mcp" | "plugins";
type McpTab = "curated" | "community";

const PAGE_SIZE = 24;

function CatalogGridSkeleton() {
  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {Array.from({ length: 6 }).map((_, i) => (
        <Skeleton key={i} className="h-44 rounded-xl" />
      ))}
    </div>
  );
}

export function Discover({ onInstalled, onOpenSettings }: DiscoverProps) {
  const [productTab, setProductTab] = useState<ProductTab>("mcp");
  const [mcpTab, setMcpTab] = useState<McpTab>("curated");

  const [catalog, setCatalog] = useState<IntegrationCatalogEntry[]>([]);
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set());
  const [catalogLoading, setCatalogLoading] = useState(true);
  const [catalogLoaded, setCatalogLoaded] = useState(false);

  const [clients, setClients] = useState<DetectionResult[]>([]);
  const [secrets, setSecrets] = useState<SecretStatus[]>([]);
  const [wizardDataLoading, setWizardDataLoading] = useState(false);
  const [wizardDataLoaded, setWizardDataLoaded] = useState(false);

  const [error, setError] = useState<string | null>(null);

  const [communityQuery, setCommunityQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [communitySort, setCommunitySort] = useState<DiscoverySort>("popular");
  const [communityResults, setCommunityResults] = useState<DiscoveredMcpEntry[]>([]);
  const [communityTotal, setCommunityTotal] = useState(0);
  const [communityLoading, setCommunityLoading] = useState(false);
  const [lastSyncedAt, setLastSyncedAt] = useState<string | null>(null);
  const [syncing, setSyncing] = useState(false);

  const [selected, setSelected] = useState<IntegrationCatalogEntry | null>(null);
  const [wizardStep, setWizardStep] = useState<WizardStep>("clients");
  const [selectedClients, setSelectedClients] = useState<Set<string>>(new Set());
  const [secretValues, setSecretValues] = useState<Record<string, string>>({});
  const [installMessage, setInstallMessage] = useState("");

  const loadCatalog = useCallback(async () => {
    setCatalogLoading(true);
    setError(null);
    try {
      const [cat, installs, status] = await Promise.all([
        getCatalog(),
        getInstallations(),
        getDiscoveryStatus(),
      ]);
      setCatalog(cat);
      setInstalledIds(new Set(installs.map((i) => i.integration_id)));
      setLastSyncedAt(status.last_synced_at ?? null);
      setCatalogLoaded(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setCatalogLoading(false);
    }
  }, []);

  const loadWizardData = useCallback(async (): Promise<{
    clients: DetectionResult[];
    secrets: SecretStatus[];
  } | null> => {
    if (wizardDataLoaded) {
      return { clients, secrets };
    }
    setWizardDataLoading(true);
    try {
      const [detected, secretStatus] = await Promise.all([
        detectClients(),
        getSecretsStatus(),
      ]);
      const supported = filterSupportedClients(detected).filter(
        (c) => c.detected && c.sync_supported,
      );
      setClients(supported);
      setSelectedClients(new Set(supported.map((c) => c.client_id)));
      setSecrets(secretStatus);
      setWizardDataLoaded(true);
      return { clients: supported, secrets: secretStatus };
    } catch (e) {
      setError(String(e));
      return null;
    } finally {
      setWizardDataLoading(false);
    }
  }, [wizardDataLoaded, clients, secrets]);

  useEffect(() => {
    loadCatalog();
  }, [loadCatalog]);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(communityQuery), 300);
    return () => clearTimeout(timer);
  }, [communityQuery]);

  const loadCommunity = useCallback(async () => {
    if (productTab !== "mcp" || mcpTab !== "community") return;
    setCommunityLoading(true);
    try {
      const result = await searchDiscoveredMcps(
        debouncedQuery,
        communitySort,
        PAGE_SIZE,
        0,
      );
      setCommunityResults(result.entries);
      setCommunityTotal(result.total);
    } catch (e) {
      setError(String(e));
    } finally {
      setCommunityLoading(false);
    }
  }, [productTab, mcpTab, debouncedQuery, communitySort]);

  useEffect(() => {
    loadCommunity();
  }, [loadCommunity]);

  const handleSync = async () => {
    setSyncing(true);
    setError(null);
    try {
      await syncDiscoveredCatalog();
      const status = await getDiscoveryStatus();
      setLastSyncedAt(status.last_synced_at ?? null);
      await loadCommunity();
    } catch (e) {
      setError(String(e));
    } finally {
      setSyncing(false);
    }
  };

  const openWizard = async (entry: IntegrationCatalogEntry) => {
    if (entry.coming_soon) return;
    setSelected(entry);
    setInstallMessage("");
    setSecretValues({});
    const data = await loadWizardData();
    if (!data) return;
    const needed = data.secrets.filter(
      (s) => s.integration_id === entry.id && s.required && !s.connected,
    );
    setWizardStep(needed.length > 0 ? "secrets" : "clients");
  };

  const closeWizard = () => {
    setSelected(null);
    loadCatalog();
  };

  const integrationSecrets = selected
    ? secrets.filter((s) => s.integration_id === selected.id)
    : [];

  const handleInstall = async () => {
    if (!selected) return;
    setWizardStep("installing");
    setError(null);
    try {
      for (const s of integrationSecrets) {
        const val = secretValues[s.secret_key];
        if (val && !s.connected) {
          await saveSecret(selected.id, s.secret_key, val);
        }
      }
      const result = await installIntegration(
        selected.id,
        Array.from(selectedClients),
      );
      setInstallMessage(result.message);
      setWizardStep("done");
    } catch (e) {
      setError(String(e));
      setWizardStep(
        integrationSecrets.some((s) => s.required && !s.connected)
          ? "secrets"
          : "clients",
      );
    }
  };

  const formatSyncTime = (iso: string | null) => {
    if (!iso) return "Never";
    try {
      return new Date(iso).toLocaleString("en-US");
    } catch {
      return iso;
    }
  };

  return (
    <div>
      <PageHeader
        title="Discover"
        description="Browse and install MCP servers and plugins for your AI assistants."
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <div className="mb-6 flex w-fit gap-1 rounded-lg border border-border p-1">
        <Button
          type="button"
          variant={productTab === "mcp" ? "default" : "ghost"}
          size="sm"
          onClick={() => setProductTab("mcp")}
        >
          MCPs
        </Button>
        <Button
          type="button"
          variant={productTab === "plugins" ? "default" : "ghost"}
          size="sm"
          onClick={() => setProductTab("plugins")}
        >
          Plugins
        </Button>
      </div>

      {productTab === "mcp" && (
        <>
          <div className="mb-6 flex w-fit gap-1 rounded-lg border border-border p-1">
            <Button
              type="button"
              variant={mcpTab === "curated" ? "default" : "ghost"}
              size="sm"
              onClick={() => setMcpTab("curated")}
            >
              Curated
            </Button>
            <Button
              type="button"
              variant={mcpTab === "community" ? "default" : "ghost"}
              size="sm"
              onClick={() => setMcpTab("community")}
            >
              Community
            </Button>
          </div>

          {mcpTab === "curated" && (
            catalogLoading && !catalogLoaded ? (
              <CatalogGridSkeleton />
            ) : (
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {catalog.map((entry) => {
                  const isInstalled = installedIds.has(entry.id);
                  return (
                    <Card key={entry.id}>
                      <CardHeader>
                        <CardTitle>{entry.name}</CardTitle>
                        <CardDescription className="line-clamp-2">
                          {entry.description}
                        </CardDescription>
                      </CardHeader>
                      <CardContent>
                        <div className="flex flex-wrap gap-1">
                          {entry.tags.map((tag) => (
                            <Badge key={tag} variant="secondary">
                              {tag}
                            </Badge>
                          ))}
                        </div>
                      </CardContent>
                      <CardFooter className="border-t-0 bg-transparent">
                        {entry.coming_soon ? (
                          <Badge variant="outline" className="w-full justify-center py-2">
                            Coming soon
                          </Badge>
                        ) : isInstalled ? (
                          <Badge className="w-full justify-center bg-emerald-500/10 py-2 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400">
                            Installed
                          </Badge>
                        ) : (
                          <Button
                            type="button"
                            className="w-full"
                            onClick={() => void openWizard(entry)}
                          >
                            Install
                          </Button>
                        )}
                      </CardFooter>
                    </Card>
                  );
                })}
              </div>
            )
          )}

          {mcpTab === "community" && (
            <div className="space-y-4">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <DiscoverySearchBar
                  query={communityQuery}
                  sort={communitySort}
                  onQueryChange={setCommunityQuery}
                  onSortChange={setCommunitySort}
                />
                <div className="flex shrink-0 flex-col items-end gap-1">
                  <LoadingButton
                    type="button"
                    variant="outline"
                    onClick={handleSync}
                    loading={syncing}
                    loadingLabel="Updating…"
                  >
                    Refresh index
                  </LoadingButton>
                  <span className="text-xs text-muted-foreground">
                    Last sync: {formatSyncTime(lastSyncedAt)}
                  </span>
                </div>
              </div>

              {communityLoading ? (
                <OperationStatus
                  className="py-8"
                  message="Searching community MCPs…"
                  detail="Loading matching entries from the local discovery index."
                />
              ) : communityResults.length === 0 ? (
                <Card className="border-dashed py-8 text-center shadow-none">
                  <CardContent>
                    <p className="text-sm text-muted-foreground">No results</p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      Try different keywords or refresh the index.
                    </p>
                  </CardContent>
                </Card>
              ) : (
                <>
                  <p className="text-xs text-muted-foreground">
                    {communityTotal.toLocaleString()} MCP
                    {communityTotal !== 1 ? "s" : ""} found
                  </p>
                  <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    {communityResults.map((entry) => (
                      <DiscoveredMcpCard
                        key={entry.id}
                        entry={entry}
                        onOpenSettings={onOpenSettings}
                        onInstalled={onInstalled}
                      />
                    ))}
                  </div>
                </>
              )}
            </div>
          )}
        </>
      )}

      <PluginsDiscoverPanel
        active={productTab === "plugins"}
        onInstalled={onInstalled}
      />

      <Dialog
        open={selected !== null}
        onOpenChange={(open) => {
          if (!open && wizardStep !== "installing") closeWizard();
        }}
      >
        <DialogContent className="sm:max-w-md" showCloseButton={wizardStep !== "installing"}>
          {selected && (
            <>
              <DialogHeader>
                <DialogTitle>Install {selected.name}</DialogTitle>
                {wizardStep === "secrets" && (
                  <DialogDescription>
                    This integration requires credentials. They will be stored securely in the macOS Keychain.
                  </DialogDescription>
                )}
                {wizardStep === "clients" && (
                  <DialogDescription>
                    Select which clients to enable this integration on.
                  </DialogDescription>
                )}
              </DialogHeader>

              {wizardDataLoading && wizardStep !== "installing" && wizardStep !== "done" ? (
                <OperationStatus
                  className="py-6"
                  message="Preparing install…"
                  detail="Detecting clients and checking required secrets."
                />
              ) : (
                <>
                  {wizardStep === "secrets" && (
                    <div className="space-y-4">
                      {integrationSecrets.map((s) => (
                        <div key={s.secret_key} className="space-y-2">
                          <Label htmlFor={s.secret_key}>{s.label}</Label>
                          {s.connected ? (
                            <p className="text-sm text-emerald-600 dark:text-emerald-400">Connected</p>
                          ) : (
                            <Input
                              id={s.secret_key}
                              type="password"
                              placeholder="Enter your API key"
                              value={secretValues[s.secret_key] ?? ""}
                              onChange={(e) =>
                                setSecretValues((prev) => ({
                                  ...prev,
                                  [s.secret_key]: e.target.value,
                                }))
                              }
                            />
                          )}
                        </div>
                      ))}
                    </div>
                  )}

                  {(wizardStep === "clients" || wizardStep === "installing" || wizardStep === "done") && (
                    <div>
                      {wizardStep === "clients" && (
                        <div className="space-y-2">
                          {clients.length === 0 ? (
                            <p className="text-sm text-amber-600 dark:text-amber-400">
                              No sync-capable clients were detected. Check Settings → Connections.
                            </p>
                          ) : (
                            clients.map((c) => (
                              <label
                                key={c.client_id}
                                className="flex items-center gap-2 rounded-lg border border-border px-3 py-2"
                              >
                                <Checkbox
                                  checked={selectedClients.has(c.client_id)}
                                  onCheckedChange={(checked) => {
                                    const next = new Set(selectedClients);
                                    if (checked) next.add(c.client_id);
                                    else next.delete(c.client_id);
                                    setSelectedClients(next);
                                  }}
                                />
                                <ClientName
                                  clientId={c.client_id}
                                  name={c.display_name}
                                  className="text-sm"
                                />
                              </label>
                            ))
                          )}
                        </div>
                      )}
                      {wizardStep === "installing" && (
                        <OperationStatus
                          className="py-6"
                          message="Installing integration…"
                          detail="Saving secrets and syncing the MCP configuration to selected clients."
                        />
                      )}
                      {wizardStep === "done" && (
                        <p className="py-4 text-center text-sm text-emerald-700 dark:text-emerald-400">
                          {installMessage}
                        </p>
                      )}
                    </div>
                  )}
                </>
              )}

              <DialogFooter>
                {wizardStep === "done" ? (
                  <Button
                    type="button"
                    onClick={() => {
                      closeWizard();
                      onInstalled();
                    }}
                  >
                    View installed
                  </Button>
                ) : (
                  <>
                    <Button
                      type="button"
                      variant="outline"
                      onClick={closeWizard}
                      disabled={wizardStep === "installing"}
                    >
                      Cancel
                    </Button>
                    {wizardStep === "secrets" && (
                      <Button
                        type="button"
                        onClick={() => {
                          const missing = integrationSecrets.filter(
                            (s) =>
                              s.required &&
                              !s.connected &&
                              !secretValues[s.secret_key]?.trim(),
                          );
                          if (missing.length > 0) {
                            setError("Complete all required secrets");
                            return;
                          }
                          setWizardStep("clients");
                        }}
                      >
                        Continue
                      </Button>
                    )}
                    {wizardStep === "clients" && (
                      <Button
                        type="button"
                        onClick={handleInstall}
                        disabled={selectedClients.size === 0 || wizardDataLoading}
                      >
                        Install
                      </Button>
                    )}
                  </>
                )}
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
