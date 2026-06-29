import { useCallback, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ErrorBanner } from "./Feedback";
import { ClientName } from "@/components/ClientLogo";
import {
  cancelCommunityInstall,
  communityMissingDependencies,
  confirmCommunityInstall,
  detectClients,
  getCommunityInstallJob,
  installDependencies,
  listHarnessInstances,
  onCommunityInstallProgress,
  probeHarnesses,
  saveSecret,
  startCommunityInstall,
} from "../hooks/useTauri";
import { filterSupportedClients } from "@/lib/clients";
import type {
  CommunityInstallJob,
  DetectionResult,
  DiscoveredMcpEntry,
  HarnessInstanceRecord,
  HarnessSnapshot,
  MissingDependency,
  ResolvedMcpConfig,
} from "../types";

type WizardStep =
  | "agent"
  | "analyzing"
  | "review"
  | "clients"
  | "installing"
  | "done";

interface CommunityInstallWizardProps {
  entry: DiscoveredMcpEntry;
  open: boolean;
  onClose: () => void;
  onInstalled: () => void;
  onOpenSettings?: () => void;
}

export function CommunityInstallWizard({
  entry,
  open,
  onClose,
  onInstalled,
  onOpenSettings,
}: CommunityInstallWizardProps) {
  const [step, setStep] = useState<WizardStep>("agent");
  const [instances, setInstances] = useState<HarnessInstanceRecord[]>([]);
  const [snapshots, setSnapshots] = useState<HarnessSnapshot[]>([]);
  const [selectedHarness, setSelectedHarness] = useState<string>("");
  const [job, setJob] = useState<CommunityInstallJob | null>(null);
  const [agentLog, setAgentLog] = useState("");
  const [resolved, setResolved] = useState<ResolvedMcpConfig | null>(null);
  const [clients, setClients] = useState<DetectionResult[]>([]);
  const [selectedClients, setSelectedClients] = useState<Set<string>>(new Set());
  const [secretValues, setSecretValues] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  const [installMessage, setInstallMessage] = useState("");
  const [missingDeps, setMissingDeps] = useState<MissingDependency[]>([]);
  const [installStatus, setInstallStatus] = useState("Installing and syncing to clients…");

  const loadHarnesses = useCallback(async () => {
    const [inst, probes, detected] = await Promise.all([
      listHarnessInstances(),
      probeHarnesses(),
      detectClients(),
    ]);
    setInstances(inst.filter((i) => i.enabled));
    setSnapshots(probes);
    setClients(
      filterSupportedClients(detected).filter((c) => c.detected && c.sync_supported),
    );
    setSelectedClients(
      new Set(
        detected
          .filter((c) => c.detected && c.sync_supported)
          .map((c) => c.client_id),
      ),
    );

    const defaultHarness = inst.find((i) => i.is_default_install_agent && i.enabled);
    const capable = inst.filter((i) => {
      const snap = probes.find((p) => p.instance_id === i.id);
      return i.enabled && snap?.agent_capable && snap.detected;
    });
    if (defaultHarness && capable.some((c) => c.id === defaultHarness.id)) {
      setSelectedHarness(defaultHarness.id);
    } else if (capable[0]) {
      setSelectedHarness(capable[0].id);
    }
  }, []);

  useEffect(() => {
    if (open) {
      setStep("agent");
      setJob(null);
      setAgentLog("");
      setResolved(null);
      setError(null);
      loadHarnesses().catch((e) => setError(String(e)));
    }
  }, [open, loadHarnesses]);

  useEffect(() => {
    if (!open || !job) return;
    let cancelled = false;
    const poll = async () => {
      while (!cancelled) {
        const current = await getCommunityInstallJob(job.id);
        if (!current) break;
        setJob(current);
        setAgentLog(current.agent_log);
        if (current.resolved_config) setResolved(current.resolved_config);
        if (current.status === "awaiting_review") {
          setStep("review");
          break;
        }
        if (current.status === "failed") {
          setError(current.error ?? "Agent analysis failed");
          setStep("agent");
          break;
        }
        if (current.status === "done") break;
        await new Promise((r) => setTimeout(r, 800));
      }
    };
    poll();
    return () => {
      cancelled = true;
    };
  }, [open, job?.id]);

  useEffect(() => {
    if (!open) return;
    let unlisten: (() => void) | undefined;
    onCommunityInstallProgress((event) => {
      if (job && event.job_id === job.id) {
        setAgentLog((prev) => prev + event.message + "\n");
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [open, job?.id]);

  const snapshotMap = Object.fromEntries(
    snapshots.map((s) => [s.instance_id, s]),
  );

  const agentCapable = instances.filter((inst) => {
    const snap = snapshotMap[inst.id];
    return snap?.agent_capable && snap.detected;
  });

  const startAnalysis = async () => {
    if (!selectedHarness) return;
    setError(null);
    setStep("analyzing");
    try {
      const newJob = await startCommunityInstall(entry.id, selectedHarness);
      setJob(newJob);
      setAgentLog(newJob.agent_log);
    } catch (e) {
      setError(String(e));
      setStep("agent");
    }
  };

  const goToClients = async () => {
    setStep("clients");
    setMissingDeps([]);
    if (resolved) {
      try {
        setMissingDeps(await communityMissingDependencies(resolved));
      } catch {
        // Non-fatal: the install confirm step still reports missing deps.
      }
    }
  };

  const handleConfirmInstall = async () => {
    if (!job || !resolved) return;
    setStep("installing");
    setError(null);
    try {
      const toInstall = missingDeps.filter((d) => d.installable).map((d) => d.name);
      if (toInstall.length > 0) {
        setInstallStatus(`Installing dependencies: ${toInstall.join(", ")}…`);
        await installDependencies(toInstall);
      }
      setInstallStatus("Installing and syncing to clients…");
      const integrationId = `community-${entry.id}`;
      for (const [key, value] of Object.entries(secretValues)) {
        if (value.trim()) {
          await saveSecret(integrationId, key, value.trim());
        }
      }
      const result = await confirmCommunityInstall(
        job.id,
        Array.from(selectedClients),
        secretValues,
        resolved,
      );
      setInstallMessage(result.message);
      setStep("done");
    } catch (e) {
      setError(String(e));
      setStep("clients");
    }
  };

  const handleCancel = async () => {
    if (job && step === "analyzing") {
      await cancelCommunityInstall(job.id).catch(() => undefined);
    }
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && handleCancel()}>
      <DialogContent className="sm:max-w-lg" showCloseButton={step !== "installing"}>
        <DialogHeader>
          <DialogTitle>Install {entry.name}</DialogTitle>
          <DialogDescription>
            Delegate MCP config analysis to a connected harness, then review before
            syncing to clients.
          </DialogDescription>
        </DialogHeader>

        {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

        {step === "agent" && (
          <div className="space-y-3">
            {agentCapable.length === 0 ? (
              <div className="rounded-lg border border-dashed border-border p-4 text-sm text-muted-foreground">
                No agent-capable harness connected.{" "}
                {onOpenSettings ? (
                  <button
                    type="button"
                    className="text-primary underline"
                    onClick={onOpenSettings}
                  >
                    Connect a harness in Settings → Connections
                  </button>
                ) : (
                  "Connect a harness in Settings → Connections."
                )}
              </div>
            ) : (
              agentCapable.map((inst) => {
                const snap = snapshotMap[inst.id];
                return (
                  <label
                    key={inst.id}
                    className="flex cursor-pointer items-start gap-2 rounded-lg border border-border px-3 py-2"
                  >
                    <input
                      type="radio"
                      name="harness"
                      checked={selectedHarness === inst.id}
                      onChange={() => setSelectedHarness(inst.id)}
                      className="mt-1"
                    />
                    <div>
                      <ClientName
                        clientId=""
                        driverKind={inst.driver_kind}
                        name={inst.display_name}
                        className="text-sm font-medium"
                      />
                      <div className="text-xs text-muted-foreground">
                        {snap?.version ? `v${snap.version} · ` : ""}
                        {snap?.auth_status === "unauthenticated"
                          ? "Warning: may not be authenticated"
                          : "Ready for agent install"}
                      </div>
                    </div>
                  </label>
                );
              })
            )}
            {entry.install_hint && (
              <p className="text-xs text-muted-foreground">
                Manual hint: {entry.install_hint}
              </p>
            )}
          </div>
        )}

        {step === "analyzing" && (
          <div className="space-y-3">
            <p className="text-sm text-muted-foreground">Analyzing with agent…</p>
            <pre className="max-h-48 overflow-auto rounded-lg bg-muted p-3 text-xs whitespace-pre-wrap">
              {agentLog || "Waiting for agent output…"}
            </pre>
          </div>
        )}

        {step === "review" && resolved && (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <Badge variant="secondary">Confidence: {resolved.confidence}</Badge>
            </div>
            <div className="space-y-2">
              <Label>Command</Label>
              <Input
                value={resolved.command}
                onChange={(e) =>
                  setResolved({ ...resolved, command: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label>Args (space-separated)</Label>
              <Input
                value={resolved.args.join(" ")}
                onChange={(e) =>
                  setResolved({
                    ...resolved,
                    args: e.target.value.split(/\s+/).filter(Boolean),
                  })
                }
              />
            </div>
            {resolved.env_keys.length > 0 && (
              <p className="text-xs text-muted-foreground">
                Required env keys: {resolved.env_keys.join(", ")}
              </p>
            )}
            {resolved.notes && (
              <p className="text-xs text-muted-foreground">{resolved.notes}</p>
            )}
            <p className="text-xs text-amber-700 dark:text-amber-400">
              Review carefully. Taro will only write configs after you confirm.
            </p>
          </div>
        )}

        {step === "clients" && resolved && (
          <div className="space-y-3">
            {missingDeps.length > 0 && (
              <div className="rounded-lg border border-border bg-muted/50 p-3 text-xs">
                <p className="mb-1 font-medium">
                  Taro will install these prerequisites for you:
                </p>
                <ul className="space-y-0.5">
                  {missingDeps.map((d) => (
                    <li key={d.name} className="text-muted-foreground">
                      {d.installable ? "• " : "⚠ "}
                      {d.install_label}
                      {!d.installable && " (install manually)"}
                    </li>
                  ))}
                </ul>
              </div>
            )}
            {resolved.env_keys.map((key) => (
              <div key={key} className="space-y-2">
                <Label htmlFor={key}>{key}</Label>
                <Input
                  id={key}
                  type="password"
                  value={secretValues[key] ?? ""}
                  onChange={(e) =>
                    setSecretValues((prev) => ({ ...prev, [key]: e.target.value }))
                  }
                />
              </div>
            ))}
            <div className="space-y-2">
              {clients.length === 0 ? (
                <p className="text-sm text-amber-600 dark:text-amber-400">
                  No sync-capable clients detected. Check Settings → Connections.
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
          </div>
        )}

        {step === "installing" && (
          <p className="py-6 text-center text-sm text-muted-foreground">
            {installStatus}
          </p>
        )}

        {step === "done" && (
          <p className="py-4 text-center text-sm text-emerald-700 dark:text-emerald-400">
            {installMessage}
          </p>
        )}

        <DialogFooter>
          {step === "done" ? (
            <Button
              type="button"
              onClick={() => {
                onClose();
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
                onClick={handleCancel}
                disabled={step === "installing"}
              >
                Cancel
              </Button>
              {step === "agent" && (
                <Button
                  type="button"
                  disabled={!selectedHarness || agentCapable.length === 0}
                  onClick={startAnalysis}
                >
                  Analyze with agent
                </Button>
              )}
              {step === "review" && (
                <Button type="button" onClick={goToClients}>
                  Continue
                </Button>
              )}
              {step === "clients" && (
                <Button
                  type="button"
                  disabled={selectedClients.size === 0}
                  onClick={handleConfirmInstall}
                >
                  {missingDeps.some((d) => d.installable)
                    ? "Install prerequisites & confirm"
                    : "Confirm install"}
                </Button>
              )}
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
