import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { StatusBadge } from "../StatusBadge";
import { ErrorBanner, EmptyState, LoadingState } from "../Feedback";
import {
  getHealthStatus,
  getInstallations,
  runAllHealthChecks,
  runHealthCheck,
} from "../../hooks/useTauri";
import type { HealthCheckRecord, InstallationRecord } from "../../types";

export function HealthPanel() {
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
      <div className="mb-4 flex items-center justify-end">
        <Button
          type="button"
          disabled={checking || installations.length === 0}
          onClick={handleCheckAll}
        >
          {checking ? "Checking…" : "Check all"}
        </Button>
      </div>
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      {installations.length === 0 ? (
        <EmptyState
          title="No integrations to check"
          description="Install an integration to see its health status here."
        />
      ) : (
        <Card className="overflow-hidden py-0">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border bg-muted/50 text-left text-muted-foreground">
                <th className="px-4 py-3 font-medium">Integration</th>
                <th className="px-4 py-3 font-medium">Status</th>
                <th className="px-4 py-3 font-medium">Latency</th>
                <th className="px-4 py-3 font-medium">Last checked</th>
                <th className="px-4 py-3 font-medium">Detail</th>
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
                    className="border-b border-border/50 last:border-0"
                  >
                    <td className="px-4 py-3 font-medium text-foreground">
                      {check?.integration_name ?? inst.integration_id}
                    </td>
                    <td className="px-4 py-3">
                      <StatusBadge
                        status={
                          check?.ok
                            ? "connected"
                            : check
                              ? "error"
                              : "disabled"
                        }
                      />
                    </td>
                    <td className="px-4 py-3 text-foreground">
                      {check?.latency_ms != null
                        ? `${check.latency_ms} ms`
                        : "—"}
                    </td>
                    <td className="px-4 py-3 text-xs text-muted-foreground">
                      {check
                        ? new Date(check.checked_at).toLocaleString("en-US")
                        : "—"}
                    </td>
                    <td className="max-w-xs truncate px-4 py-3 text-xs text-muted-foreground">
                      {check?.detail ?? "—"}
                    </td>
                    <td className="px-4 py-3">
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        disabled={checking}
                        onClick={() => handleCheckOne(inst.id)}
                      >
                        Check
                      </Button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </Card>
      )}
    </div>
  );
}
