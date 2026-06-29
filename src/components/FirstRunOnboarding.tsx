import { useCallback, useEffect, useState } from "react";
import {
  completeFirstRun,
  getFirstRunStatus,
} from "../hooks/useTauri";
import type { FirstRunStatus } from "../types";

interface FirstRunOnboardingProps {
  onComplete: () => void;
}

export function FirstRunOnboarding({ onComplete }: FirstRunOnboardingProps) {
  const [status, setStatus] = useState<FirstRunStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [step, setStep] = useState<"welcome" | "scan" | "import">("welcome");

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const s = await getFirstRunStatus();
      setStatus(s);
      if (s.completed) {
        onComplete();
      }
    } finally {
      setLoading(false);
    }
  }, [onComplete]);

  useEffect(() => {
    load();
  }, [load]);

  const handleComplete = async () => {
    await completeFirstRun();
    onComplete();
  };

  if (loading || !status) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-neutral-100">
        <p className="text-sm text-neutral-500">Preparando Taro…</p>
      </div>
    );
  }

  const detected = status.detected_clients.filter((c) => c.detected);

  return (
    <div className="flex min-h-screen items-center justify-center bg-neutral-100 p-6">
      <div className="w-full max-w-lg rounded-2xl bg-white p-8 shadow-lg">
        {step === "welcome" && (
          <>
            <h1 className="text-2xl font-semibold text-neutral-900">
              Bienvenido a Taro
            </h1>
            <p className="mt-2 text-sm text-neutral-500">
              Taro gestiona tus integraciones MCP para Cursor, Claude Desktop y más.
              Te ayudaremos a configurar todo en unos pasos.
            </p>
            <button
              type="button"
              onClick={() => setStep("scan")}
              className="mt-6 w-full rounded-lg bg-neutral-900 px-4 py-2.5 text-sm font-medium text-white"
            >
              Empezar
            </button>
          </>
        )}

        {step === "scan" && (
          <>
            <h2 className="text-xl font-semibold">Clientes detectados</h2>
            <p className="mt-1 text-sm text-neutral-500">
              Hemos escaneado tu sistema en busca de aplicaciones compatibles.
            </p>
            <div className="mt-4 space-y-2">
              {status.detected_clients.map((c) => (
                <div
                  key={c.client_id}
                  className="flex items-center justify-between rounded-lg border border-neutral-200 px-3 py-2"
                >
                  <span className="text-sm">{c.display_name}</span>
                  <span
                    className={`text-xs ${
                      c.detected ? "text-emerald-600" : "text-neutral-400"
                    }`}
                  >
                    {c.detected ? "Detectado" : "No encontrado"}
                  </span>
                </div>
              ))}
            </div>
            {detected.length === 0 && (
              <p className="mt-3 text-sm text-amber-600">
                No detectamos clientes instalados. Puedes instalar integraciones igualmente y configurarlas después.
              </p>
            )}
            <button
              type="button"
              onClick={() => setStep("import")}
              className="mt-6 w-full rounded-lg bg-neutral-900 px-4 py-2.5 text-sm font-medium text-white"
            >
              Continuar
            </button>
          </>
        )}

        {step === "import" && (
          <>
            <h2 className="text-xl font-semibold">Configuración existente</h2>
            <p className="mt-1 text-sm text-neutral-500">
              Servidores MCP encontrados en tus clientes (solo lectura en v0.1).
            </p>
            {status.existing_servers.length === 0 ? (
              <p className="mt-4 text-sm text-neutral-500">
                No se encontraron servidores MCP configurados.
              </p>
            ) : (
              <ul className="mt-4 max-h-48 space-y-2 overflow-y-auto">
                {status.existing_servers.map((s, i) => (
                  <li
                    key={`${s.client_id}-${s.server_id}-${i}`}
                    className="rounded-lg bg-neutral-50 px-3 py-2 text-xs"
                  >
                    <span className="font-medium">{s.client_name}</span>
                    <span className="text-neutral-400"> — {s.server_id}</span>
                    <br />
                    <span className="text-neutral-500">{s.command}</span>
                  </li>
                ))}
              </ul>
            )}
            <button
              type="button"
              onClick={handleComplete}
              className="mt-6 w-full rounded-lg bg-neutral-900 px-4 py-2.5 text-sm font-medium text-white"
            >
              Ir a Taro
            </button>
          </>
        )}
      </div>
    </div>
  );
}
