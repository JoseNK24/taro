import { useCallback, useEffect, useState } from "react";
import { ErrorBanner, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import {
  detectClients,
  getCatalog,
  getInstallations,
  getSecretsStatus,
  installIntegration,
  saveSecret,
} from "../hooks/useTauri";
import type {
  DetectionResult,
  IntegrationCatalogEntry,
  SecretStatus,
} from "../types";

interface DiscoverProps {
  onInstalled: () => void;
}

type WizardStep = "clients" | "secrets" | "installing" | "done";

export function Discover({ onInstalled }: DiscoverProps) {
  const [catalog, setCatalog] = useState<IntegrationCatalogEntry[]>([]);
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set());
  const [clients, setClients] = useState<DetectionResult[]>([]);
  const [secrets, setSecrets] = useState<SecretStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [selected, setSelected] = useState<IntegrationCatalogEntry | null>(null);
  const [wizardStep, setWizardStep] = useState<WizardStep>("clients");
  const [selectedClients, setSelectedClients] = useState<Set<string>>(new Set());
  const [secretValues, setSecretValues] = useState<Record<string, string>>({});
  const [installMessage, setInstallMessage] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [cat, installs, detected, secretStatus] = await Promise.all([
        getCatalog(),
        getInstallations(),
        detectClients(),
        getSecretsStatus(),
      ]);
      setCatalog(cat);
      setInstalledIds(new Set(installs.map((i) => i.integration_id)));
      setClients(
        detected.filter((c) => c.detected && c.sync_supported),
      );
      setSelectedClients(
        new Set(
          detected
            .filter((c) => c.detected && c.sync_supported)
            .map((c) => c.client_id),
        ),
      );
      setSecrets(secretStatus);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const openWizard = (entry: IntegrationCatalogEntry) => {
    if (entry.coming_soon) return;
    setSelected(entry);
    setWizardStep("clients");
    setInstallMessage("");
    const needed = secrets.filter(
      (s) => s.integration_id === entry.id && s.required && !s.connected,
    );
    setWizardStep(needed.length > 0 ? "secrets" : "clients");
    setSecretValues({});
  };

  const closeWizard = () => {
    setSelected(null);
    load();
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
      setWizardStep(integrationSecrets.some((s) => s.required && !s.connected) ? "secrets" : "clients");
    }
  };

  if (loading) return <LoadingState />;

  return (
    <div>
      <PageHeader
        title="Descubrir"
        description="Explora e instala integraciones para tus asistentes de IA."
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {catalog.map((entry) => {
          const isInstalled = installedIds.has(entry.id);
          return (
            <article
              key={entry.id}
              className="flex flex-col rounded-xl border border-neutral-200 bg-white p-4 shadow-sm"
            >
              <div>
                <h3 className="font-medium text-neutral-900">{entry.name}</h3>
                <p className="mt-1 text-sm text-neutral-500 line-clamp-2">
                  {entry.description}
                </p>
              </div>
              <div className="mt-3 flex flex-wrap gap-1">
                {entry.tags.map((tag) => (
                  <span
                    key={tag}
                    className="rounded-md bg-neutral-100 px-2 py-0.5 text-xs text-neutral-600"
                  >
                    {tag}
                  </span>
                ))}
              </div>
              <div className="mt-4">
                {entry.coming_soon ? (
                  <span className="inline-flex w-full items-center justify-center rounded-lg bg-neutral-100 px-3 py-2 text-sm text-neutral-500">
                    Próximamente
                  </span>
                ) : isInstalled ? (
                  <span className="inline-flex w-full items-center justify-center rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
                    Instalada
                  </span>
                ) : (
                  <button
                    type="button"
                    onClick={() => openWizard(entry)}
                    className="w-full rounded-lg bg-neutral-900 px-3 py-2 text-sm font-medium text-white hover:bg-neutral-800"
                  >
                    Instalar
                  </button>
                )}
              </div>
            </article>
          );
        })}
      </div>

      {selected && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 p-4">
          <div className="w-full max-w-md rounded-2xl bg-white p-6 shadow-xl">
            <h3 className="text-lg font-semibold">Instalar {selected.name}</h3>

            {wizardStep === "secrets" && (
              <div className="mt-4 space-y-4">
                <p className="text-sm text-neutral-500">
                  Esta integración requiere credenciales. Se guardarán de forma segura en el Llavero de macOS.
                </p>
                {integrationSecrets.map((s) => (
                  <div key={s.secret_key}>
                    <label className="block text-sm font-medium text-neutral-700">
                      {s.label}
                    </label>
                    {s.connected ? (
                      <p className="mt-1 text-sm text-emerald-600">Conectado</p>
                    ) : (
                      <input
                        type="password"
                        className="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2 text-sm"
                        placeholder="Introduce tu clave API"
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
              <div className="mt-4">
                {wizardStep === "clients" && (
                  <>
                    <p className="text-sm text-neutral-500">
                      Selecciona en qué clientes activar esta integración.
                    </p>
                    <div className="mt-3 space-y-2">
                      {clients.length === 0 ? (
                        <p className="text-sm text-amber-600">
                          No se detectaron clientes compatibles con sincronización. Revisa la sección Clientes.
                        </p>
                      ) : (
                        clients.map((c) => (
                          <label
                            key={c.client_id}
                            className="flex items-center gap-2 rounded-lg border border-neutral-200 px-3 py-2"
                          >
                            <input
                              type="checkbox"
                              checked={selectedClients.has(c.client_id)}
                              onChange={(e) => {
                                const next = new Set(selectedClients);
                                if (e.target.checked) next.add(c.client_id);
                                else next.delete(c.client_id);
                                setSelectedClients(next);
                              }}
                            />
                            <span className="text-sm">{c.display_name}</span>
                          </label>
                        ))
                      )}
                    </div>
                  </>
                )}
                {wizardStep === "installing" && (
                  <p className="py-8 text-center text-sm text-neutral-500">
                    Instalando integración…
                  </p>
                )}
                {wizardStep === "done" && (
                  <p className="py-4 text-center text-sm text-emerald-700">
                    {installMessage}
                  </p>
                )}
              </div>
            )}

            <div className="mt-6 flex justify-end gap-2">
              {wizardStep === "done" ? (
                <button
                  type="button"
                  onClick={() => {
                    closeWizard();
                    onInstalled();
                  }}
                  className="rounded-lg bg-neutral-900 px-4 py-2 text-sm font-medium text-white"
                >
                  Ver instaladas
                </button>
              ) : (
                <>
                  <button
                    type="button"
                    onClick={closeWizard}
                    disabled={wizardStep === "installing"}
                    className="rounded-lg px-4 py-2 text-sm text-neutral-600 hover:bg-neutral-100"
                  >
                    Cancelar
                  </button>
                  {wizardStep === "secrets" && (
                    <button
                      type="button"
                      onClick={() => {
                        const missing = integrationSecrets.filter(
                          (s) =>
                            s.required &&
                            !s.connected &&
                            !secretValues[s.secret_key]?.trim(),
                        );
                        if (missing.length > 0) {
                          setError("Completa todos los secretos requeridos");
                          return;
                        }
                        setWizardStep("clients");
                      }}
                      className="rounded-lg bg-neutral-900 px-4 py-2 text-sm font-medium text-white"
                    >
                      Continuar
                    </button>
                  )}
                  {wizardStep === "clients" && (
                    <button
                      type="button"
                      onClick={handleInstall}
                      disabled={selectedClients.size === 0}
                      className="rounded-lg bg-neutral-900 px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
                    >
                      Instalar
                    </button>
                  )}
                </>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
