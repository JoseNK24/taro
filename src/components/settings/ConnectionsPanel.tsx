import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ErrorBanner } from "../Feedback";
import {
  createHarnessInstance,
  deleteHarnessInstance,
  detectClients,
  getDependencies,
  listHarnessDrivers,
  listHarnessInstances,
  probeHarnesses,
  setDefaultInstallAgent,
  updateHarnessInstance,
} from "../../hooks/useTauri";
import type {
  DependencyStatus,
  DetectionResult,
  HarnessDriverInfo,
  HarnessInstanceRecord,
  HarnessSnapshot,
} from "../../types";
import { AddHarnessDialog } from "./AddHarnessDialog";
import { HarnessInstanceCard } from "./HarnessInstanceCard";
import { McpClientsTable } from "./McpClientsTable";

const PROBE_INTERVAL_MS = 5 * 60 * 1000;

export function ConnectionsPanel() {
  const [instances, setInstances] = useState<HarnessInstanceRecord[]>([]);
  const [snapshots, setSnapshots] = useState<HarnessSnapshot[]>([]);
  const [drivers, setDrivers] = useState<HarnessDriverInfo[]>([]);
  const [clients, setClients] = useState<DetectionResult[]>([]);
  const [deps, setDeps] = useState<DependencyStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [addOpen, setAddOpen] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      const [inst, probes, driverList, detected, dependencies] = await Promise.all([
        listHarnessInstances(),
        probeHarnesses(),
        listHarnessDrivers(),
        detectClients(),
        getDependencies(),
      ]);
      setInstances(inst);
      setSnapshots(probes);
      setDrivers(driverList);
      setClients(detected);
      setDeps(dependencies);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
    const timer = setInterval(() => {
      probeHarnesses()
        .then(setSnapshots)
        .catch(() => undefined);
    }, PROBE_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [load]);

  const snapshotMap = Object.fromEntries(
    snapshots.map((s) => [s.instance_id, s]),
  );

  const handleAdd = async (driverKind: string, displayName: string) => {
    await createHarnessInstance(driverKind, displayName);
    await load();
  };

  const handleToggle = async (id: string, enabled: boolean, inst: HarnessInstanceRecord) => {
    setBusyId(id);
    try {
      await updateHarnessInstance(id, inst.display_name, enabled, inst.config_json);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
    }
  };

  const handleSetDefault = async (id: string) => {
    setBusyId(id);
    try {
      await setDefaultInstallAgent(id);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
    }
  };

  const confirmDelete = async () => {
    if (!pendingDeleteId) return;
    const id = pendingDeleteId;
    setPendingDeleteId(null);
    setBusyId(id);
    try {
      await deleteHarnessInstance(id);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(null);
    }
  };

  return (
    <div className="space-y-8">
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <section>
        <div className="mb-4 flex items-center justify-between">
          <div>
            <h3 className="text-sm font-medium text-foreground">Harnesses</h3>
            <p className="text-xs text-muted-foreground">
              Connect AI agents on your Mac to analyze and install community MCPs.
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={load}
              disabled={loading}
            >
              Re-detect
            </Button>
            <Button type="button" size="sm" onClick={() => setAddOpen(true)}>
              Add harness
            </Button>
          </div>
        </div>

        {instances.length === 0 ? (
          <p className="rounded-lg border border-dashed border-border px-4 py-8 text-center text-sm text-muted-foreground">
            No harnesses connected. Add one to enable community MCP installs.
          </p>
        ) : (
          <div className="space-y-3">
            {instances.map((inst) => (
              <HarnessInstanceCard
                key={inst.id}
                instance={inst}
                snapshot={snapshotMap[inst.id]}
                busy={busyId === inst.id}
                onToggleEnabled={(enabled) => handleToggle(inst.id, enabled, inst)}
                onSetDefault={() => handleSetDefault(inst.id)}
                onDelete={() => setPendingDeleteId(inst.id)}
              />
            ))}
          </div>
        )}
      </section>

      <McpClientsTable
        clients={clients}
        deps={deps}
        onRefresh={load}
        loading={loading}
      />

      <AddHarnessDialog
        open={addOpen}
        drivers={drivers}
        onClose={() => setAddOpen(false)}
        onAdd={handleAdd}
      />

      <Dialog
        open={pendingDeleteId !== null}
        onOpenChange={(open) => !open && setPendingDeleteId(null)}
      >
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Remove harness</DialogTitle>
            <DialogDescription>
              Remove this harness connection? Community MCP installs will no
              longer use it.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setPendingDeleteId(null)}
            >
              Cancel
            </Button>
            <Button
              type="button"
              variant="destructive"
              disabled={busyId !== null}
              onClick={() => void confirmDelete()}
            >
              Remove
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
