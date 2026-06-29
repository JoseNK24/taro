import { useCallback, useEffect, useState } from "react";
import { StatusBadge } from "../components/StatusBadge";
import { ErrorBanner, EmptyState, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import {
  getHealthStatus,
  getInstallations,
  runAllHealthChecks,
  runHealthCheck,
} from "../hooks/useTauri";
import type { HealthCheckRecord, InstallationRecord } from "../types";

export function Health() {
  const [checks, setChecks] = useState<HealthCheckRecord[]>([]);
  const [installations, setInstallations] = useState<InstallationRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [checking, setChecking] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [health, installs] = await Promise.all([
        getHealthStatus(),
        getInstallations(),
      ]);
      setChecks(health);
      setInstallations(installs);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleCheckAll = async () => {
    setChecking(true);
    setError(null);
    try {
      await runAllHealthChecks();
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setChecking(false);
    }
  };

  const handleCheckOne = async (installationId: string) => {
    setChecking(true);
    try {
      await runHealthCheck(installationId);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setChecking(false);
    }
  };

  if (loading) return <LoadingState />;

  return (
    <div>
      <PageHeader
        title="Salud"
        description="Estado de conexión de tus integraciones instaladas."
        action={
          <button
            type="button"
            disabled={checking || installations.length === 0}
            onClick={handleCheckAll}
            className="rounded-lg bg-neutral-900 px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
          >
            {checking ? "Comprobando…" : "Comprobar todo"}
          </button>
        }
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      {installations.length === 0 ? (
        <EmptyState
          title="Sin integraciones para comprobar"
          description="Instala una integración para ver su estado de salud aquí."
        />
      ) : (
        <div className="overflow-hidden rounded-xl border border-neutral-200 bg-white shadow-sm">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-neutral-100 bg-neutral-50 text-left text-neutral-500">
                <th className="px-4 py-3 font-medium">Integración</th>
                <th className="px-4 py-3 font-medium">Estado</th>
                <th className="px-4 py-3 font-medium">Latencia</th>
                <th className="px-4 py-3 font-medium">Última comprobación</th>
                <th className="px-4 py-3 font-medium">Detalle</th>
                <th className="px-4 py-3 font-medium" />
              </tr>
            </thead>
            <tbody>
              {installations.map((inst) => {
                const check = checks.find(
                  (c) => c.installation_id === inst.id,
                );
                return (
                  <tr
                    key={inst.id}
                    className="border-b border-neutral-50 last:border-0"
                  >
                    <td className="px-4 py-3 font-medium text-neutral-800">
                      {check?.integration_name ?? inst.integration_id}
                    </td>
                    <td className="px-4 py-3">
                      <StatusBadge
                        status={check?.ok ? "connected" : check ? "error" : "disabled"}
                      />
                    </td>
                    <td className="px-4 py-3 text-neutral-600">
                      {check?.latency_ms != null
                        ? `${check.latency_ms} ms`
                        : "—"}
                    </td>
                    <td className="px-4 py-3 text-xs text-neutral-500">
                      {check
                        ? new Date(check.checked_at).toLocaleString("es-ES")
                        : "—"}
                    </td>
                    <td className="max-w-xs truncate px-4 py-3 text-xs text-neutral-500">
                      {check?.detail ?? "—"}
                    </td>
                    <td className="px-4 py-3">
                      <button
                        type="button"
                        disabled={checking}
                        onClick={() => handleCheckOne(inst.id)}
                        className="text-xs text-neutral-600 hover:text-neutral-900"
                      >
                        Comprobar
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
