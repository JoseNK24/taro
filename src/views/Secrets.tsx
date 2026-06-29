import { useCallback, useEffect, useState } from "react";
import { ErrorBanner, EmptyState, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import {
  getSecretsStatus,
  removeSecret,
  saveSecret,
} from "../hooks/useTauri";
import type { SecretStatus } from "../types";

export function Secrets() {
  const [secrets, setSecrets] = useState<SecretStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<SecretStatus | null>(null);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setSecrets(await getSecretsStatus());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const grouped = secrets.reduce<Record<string, SecretStatus[]>>((acc, s) => {
    (acc[s.integration_id] ??= []).push(s);
    return acc;
  }, {});

  const handleSave = async () => {
    if (!editing || !value) return;
    setBusy(true);
    try {
      await saveSecret(editing.integration_id, editing.secret_key, value);
      setEditing(null);
      setValue("");
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleDelete = async (s: SecretStatus) => {
    if (!confirm(`¿Eliminar el secreto de ${s.integration_name}?`)) return;
    setBusy(true);
    try {
      await removeSecret(s.integration_id, s.secret_key);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (loading) return <LoadingState />;

  return (
    <div>
      <PageHeader
        title="Secretos"
        description="Credenciales almacenadas de forma segura en el Llavero de macOS."
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      {secrets.length === 0 ? (
        <EmptyState
          title="No hay secretos configurados"
          description="Las integraciones que requieren API keys aparecerán aquí."
        />
      ) : (
        <div className="space-y-4">
          {Object.entries(grouped).map(([integrationId, items]) => (
            <div
              key={integrationId}
              className="rounded-xl border border-neutral-200 bg-white p-4 shadow-sm"
            >
              <h3 className="font-medium text-neutral-900">
                {items[0].integration_name}
              </h3>
              <div className="mt-3 space-y-2">
                {items.map((s) => (
                  <div
                    key={s.secret_key}
                    className="flex items-center justify-between rounded-lg bg-neutral-50 px-3 py-2"
                  >
                    <div>
                      <p className="text-sm font-medium text-neutral-800">
                        {s.label}
                      </p>
                      <p className="text-xs text-neutral-500">
                        {s.connected ? (
                          <span className="text-emerald-600">Conectado</span>
                        ) : (
                          <span className="text-amber-600">API Key requerida</span>
                        )}
                      </p>
                    </div>
                    <div className="flex gap-2">
                      <button
                        type="button"
                        disabled={busy}
                        onClick={() => {
                          setEditing(s);
                          setValue("");
                        }}
                        className="rounded-lg border border-neutral-200 px-3 py-1 text-xs hover:bg-white"
                      >
                        {s.connected ? "Actualizar" : "Añadir"}
                      </button>
                      {s.connected && (
                        <button
                          type="button"
                          disabled={busy}
                          onClick={() => handleDelete(s)}
                          className="rounded-lg border border-red-200 px-3 py-1 text-xs text-red-600 hover:bg-red-50"
                        >
                          Eliminar
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {editing && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 p-4">
          <div className="w-full max-w-sm rounded-2xl bg-white p-6 shadow-xl">
            <h3 className="text-lg font-semibold">{editing.label}</h3>
            <p className="mt-1 text-sm text-neutral-500">
              {editing.integration_name}
            </p>
            <input
              type="password"
              autoFocus
              className="mt-4 w-full rounded-lg border border-neutral-300 px-3 py-2 text-sm"
              placeholder="Introduce tu clave API"
              value={value}
              onChange={(e) => setValue(e.target.value)}
            />
            <p className="mt-2 text-xs text-neutral-400">
              El valor no se mostrará después de guardar.
            </p>
            <div className="mt-4 flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setEditing(null)}
                className="rounded-lg px-4 py-2 text-sm text-neutral-600 hover:bg-neutral-100"
              >
                Cancelar
              </button>
              <button
                type="button"
                disabled={!value || busy}
                onClick={handleSave}
                className="rounded-lg bg-neutral-900 px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
              >
                Guardar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
