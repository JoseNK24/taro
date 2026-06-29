import { useCallback, useEffect, useState } from "react";
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
import { ErrorBanner, LoadingState } from "../components/Feedback";
import {
  CLIENT_LABELS,
  isStrategySupported,
  PluginCard,
} from "../components/PluginCard";
import { PageHeader } from "../components/Sidebar";
import {
  detectClients,
  getPluginCatalog,
  getPluginInstallations,
  installPlugin,
} from "../hooks/useTauri";
import type { DetectionResult, PluginCatalogEntry } from "../types";

interface PluginsDiscoverProps {
  onInstalled: () => void;
}

type WizardStep = "summary" | "clients" | "installing" | "done";

function compatibleClients(
  entry: PluginCatalogEntry,
  detected: DetectionResult[],
): DetectionResult[] {
  return detected.filter((c) => {
    const strategy = entry.client_install[c.client_id];
    return c.detected && isStrategySupported(strategy);
  });
}

export function PluginsDiscover({ onInstalled }: PluginsDiscoverProps) {
  const [catalog, setCatalog] = useState<PluginCatalogEntry[]>([]);
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set());
  const [clients, setClients] = useState<DetectionResult[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [selected, setSelected] = useState<PluginCatalogEntry | null>(null);
  const [wizardStep, setWizardStep] = useState<WizardStep>("summary");
  const [selectedClients, setSelectedClients] = useState<Set<string>>(new Set());
  const [installMessage, setInstallMessage] = useState("");
  const [clientResults, setClientResults] = useState<
    Array<{ client_id: string; success: boolean; message: string }>
  >([]);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [cat, installs, detected] = await Promise.all([
        getPluginCatalog(),
        getPluginInstallations(),
        detectClients(),
      ]);
      setCatalog(cat);
      setInstalledIds(new Set(installs.map((i) => i.plugin_id)));
      setClients(detected);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const openWizard = (entry: PluginCatalogEntry) => {
    if (entry.coming_soon) return;
    const compatible = compatibleClients(entry, clients);
    setSelected(entry);
    setWizardStep("summary");
    setInstallMessage("");
    setClientResults([]);
    setSelectedClients(new Set(compatible.map((c) => c.client_id)));
  };

  const closeWizard = () => {
    setSelected(null);
    load();
  };

  const wizardClients = selected ? compatibleClients(selected, clients) : [];

  const handleInstall = async () => {
    if (!selected) return;
    setWizardStep("installing");
    setError(null);
    try {
      const result = await installPlugin(
        selected.id,
        Array.from(selectedClients),
      );
      setInstallMessage(result.message);
      setClientResults(result.client_results);
      setWizardStep("done");
    } catch (e) {
      setError(String(e));
      setWizardStep("clients");
    }
  };

  if (loading) return <LoadingState />;

  return (
    <div>
      <PageHeader
        title="Plugins"
        description="Skills, rules and agent behavior for your AI assistants."
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {catalog.map((entry) => (
          <PluginCard
            key={entry.id}
            entry={entry}
            installed={installedIds.has(entry.id)}
            onInstall={openWizard}
          />
        ))}
      </div>

      <Dialog
        open={selected !== null}
        onOpenChange={(open) => {
          if (!open && wizardStep !== "installing") closeWizard();
        }}
      >
        <DialogContent
          className="sm:max-w-md"
          showCloseButton={wizardStep !== "installing"}
        >
          {selected && (
            <>
              <DialogHeader>
                <DialogTitle>Install {selected.name}</DialogTitle>
                {wizardStep === "summary" && (
                  <DialogDescription>{selected.description}</DialogDescription>
                )}
                {wizardStep === "clients" && (
                  <DialogDescription>
                    Select which detected clients to install this plugin on.
                  </DialogDescription>
                )}
              </DialogHeader>

              {wizardStep === "summary" && (
                <div className="space-y-3 text-sm text-muted-foreground">
                  <p>
                    This plugin adds skills and rules to guide agent behavior
                    across your AI coding tools.
                  </p>
                  {selected.github_url && (
                    <a
                      href={selected.github_url}
                      target="_blank"
                      rel="noreferrer"
                      className="text-primary underline-offset-4 hover:underline"
                    >
                      View on GitHub
                    </a>
                  )}
                </div>
              )}

              {(wizardStep === "clients" ||
                wizardStep === "installing" ||
                wizardStep === "done") && (
                <div>
                  {wizardStep === "clients" && (
                    <div className="space-y-2">
                      {wizardClients.length === 0 ? (
                        <p className="text-sm text-amber-600 dark:text-amber-400">
                          No compatible clients detected. Check the Clients
                          section.
                        </p>
                      ) : (
                        wizardClients.map((c) => (
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
                            <span className="text-sm">{c.display_name}</span>
                          </label>
                        ))
                      )}
                    </div>
                  )}
                  {wizardStep === "installing" && (
                    <p className="py-8 text-center text-sm text-muted-foreground">
                      Installing plugin…
                    </p>
                  )}
                  {wizardStep === "done" && (
                    <div className="space-y-3 py-2">
                      <p className="text-center text-sm text-emerald-700 dark:text-emerald-400">
                        {installMessage}
                      </p>
                      {clientResults.length > 0 && (
                        <ul className="space-y-1 text-xs text-muted-foreground">
                          {clientResults.map((r) => (
                            <li
                              key={r.client_id}
                              className={
                                r.success
                                  ? "text-emerald-600 dark:text-emerald-400"
                                  : "text-destructive"
                              }
                            >
                              {CLIENT_LABELS[r.client_id] ?? r.client_id}:{" "}
                              {r.message}
                            </li>
                          ))}
                        </ul>
                      )}
                    </div>
                  )}
                </div>
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
                    {wizardStep === "summary" && (
                      <Button
                        type="button"
                        onClick={() => setWizardStep("clients")}
                      >
                        Continue
                      </Button>
                    )}
                    {wizardStep === "clients" && (
                      <Button
                        type="button"
                        onClick={handleInstall}
                        disabled={selectedClients.size === 0}
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
