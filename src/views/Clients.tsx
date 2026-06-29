import { useCallback, useEffect, useState } from "react";
import { ErrorBanner, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import { detectClients, getDependencies } from "../hooks/useTauri";
import type { DependencyStatus, DetectionResult } from "../types";

export function Clients() {
  const [clients, setClients] = useState<DetectionResult[]>([]);
  const [deps, setDeps] = useState<DependencyStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [detected, dependencies] = await Promise.all([
        detectClients(),
        getDependencies(),
      ]);
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
  }, [load]);

  if (loading) return <LoadingState />;

  const detectedCount = clients.filter((c) => c.detected).length;

  return (
    <div>
      <PageHeader
        title="Clientes"
        description="Aplicaciones de IA detectadas en tu Mac y herramientas del sistema."
        action={
          <button
            type="button"
            onClick={load}
            className="rounded-lg border border-neutral-200 px-3 py-1.5 text-sm hover:bg-white"
          >
            Volver a detectar
          </button>
        }
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <section className="mb-8">
        <h3 className="mb-3 text-sm font-medium text-neutral-700">
          Clientes de IA ({detectedCount} detectados de {clients.length})
        </h3>
        <div className="grid gap-3 sm:grid-cols-2">
          {clients.map((client) => (
            <div
              key={client.client_id}
              className="rounded-xl border border-neutral-200 bg-white p-4 shadow-sm"
            >
              <div className="flex items-center justify-between gap-3">
                <div className="min-w-0 flex-1">
                  <h4 className="font-medium text-neutral-900">
                    {client.display_name}
                  </h4>
                  <p className="text-xs text-neutral-500">
                    {client.detected ? "Detectado" : "No detectado"}
                    {!client.sync_supported && client.detected
                      ? " · Sincronización próximamente"
                      : ""}
                  </p>
                </div>
                <span
                  className={`h-2.5 w-2.5 shrink-0 rounded-full ${
                    client.detected ? "bg-emerald-500" : "bg-neutral-300"
                  }`}
                />
              </div>
              {client.config_path && (
                <p className="mt-2 truncate text-xs text-neutral-400">
                  {client.config_exists
                    ? "Configuración encontrada"
                    : "Sin configuración"}{" "}
                  — {client.config_path}
                </p>
              )}
            </div>
          ))}
        </div>
      </section>

      <section>
        <h3 className="mb-3 text-sm font-medium text-neutral-700">
          Dependencias del sistema
        </h3>
        <div className="rounded-xl border border-neutral-200 bg-white shadow-sm">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-neutral-100 text-left text-neutral-500">
                <th className="px-4 py-3 font-medium">Herramienta</th>
                <th className="px-4 py-3 font-medium">Estado</th>
                <th className="px-4 py-3 font-medium">Ruta</th>
              </tr>
            </thead>
            <tbody>
              {deps.map((dep) => (
                <tr
                  key={dep.name}
                  className="border-b border-neutral-50 last:border-0"
                >
                  <td className="px-4 py-3 font-medium text-neutral-800">
                    {dep.name}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={
                        dep.available ? "text-emerald-600" : "text-amber-600"
                      }
                    >
                      {dep.available ? "Disponible" : "No encontrado"}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-xs text-neutral-400">
                    {dep.path ?? "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
