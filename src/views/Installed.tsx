import { useCallback, useEffect, useState } from "react";
import { StatusBadge } from "../components/StatusBadge";
import { ErrorBanner, EmptyState, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import {
  getCatalog,
  getInstallations,
  runHealthCheck,
  toggleInstallation,
  uninstallIntegration,
} from "../hooks/useTauri";
import type { InstallationRecord, IntegrationCatalogEntry } from "../types";

export function Installed() {
  const [installations, setInstallations] = useState<InstallationRecord[]>([]);
  const [catalog, setCatalog] = useState<IntegrationCatalogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [inst, cat] = await Promise.all([getInstallations(), getCatalog()]);
      setInstallations(inst);
      setCatalog(cat);
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
    if (!confirm("¿Eliminar esta integración? Se quitará de los clientes configurados.")) {
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

  if (loading) return <LoadingState />;

  return (
    <div>
      <PageHeader
        title="Instaladas"
        description="Gestiona las integraciones activas en tu sistema."
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      {installations.length === 0 ? (
        <EmptyState
          title="No hay integraciones instaladas"
          description="Explora el catálogo en Descubrir para instalar tu primera integración."
        />
      ) : (
        <div className="space-y-3">
          {installations.map((inst) => {
            const entry = catalogMap[inst.integration_id];
            return (
              <div
                key={inst.id}
                className="flex items-center gap-4 rounded-xl border border-neutral-200 bg-white p-4 shadow-sm"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium text-neutral-900">
                      {entry?.name ?? inst.integration_id}
                    </h3>
                    <StatusBadge status={inst.status} />
                  </div>
                  {inst.error_message && (
                    <p className="mt-1 text-xs text-red-600">{inst.error_message}</p>
                  )}
                  <p className="mt-0.5 text-xs text-neutral-400">
                    Instalada el {new Date(inst.installed_at).toLocaleDateString("es-ES")}
                  </p>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <button
                    type="button"
                    disabled={busyId === inst.id}
                    onClick={() => handleToggle(inst.id, !inst.enabled)}
                    className="rounded-lg border border-neutral-200 px-3 py-1.5 text-xs hover:bg-neutral-50"
                  >
                    {inst.enabled ? "Desactivar" : "Activar"}
                  </button>
                  <button
                    type="button"
                    disabled={busyId === inst.id}
                    onClick={() => handleHealth(inst.id)}
                    className="rounded-lg border border-neutral-200 px-3 py-1.5 text-xs hover:bg-neutral-50"
                  >
                    Comprobar
                  </button>
                  <button
                    type="button"
                    disabled={busyId === inst.id}
                    onClick={() => handleRemove(inst.id)}
                    className="rounded-lg border border-red-200 px-3 py-1.5 text-xs text-red-600 hover:bg-red-50"
                  >
                    Eliminar
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
