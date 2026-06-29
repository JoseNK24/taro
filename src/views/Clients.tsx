import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ClientName } from "@/components/ClientLogo";
import { ErrorBanner, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import { filterSupportedClients } from "@/lib/clients";
import { detectClients, getDependencies } from "../hooks/useTauri";
import type { DependencyStatus, DetectionResult } from "../types";

function syncLabel(client: DetectionResult): string {
  if (!client.detected) return "—";
  if (client.sync_supported) return "Supported";
  return "Coming soon";
}

function configLabel(client: DetectionResult): string {
  if (!client.config_path) return "—";
  return client.config_exists ? "Found" : "No configuration";
}

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

  const supportedClients = filterSupportedClients(clients);
  const detectedCount = supportedClients.filter((c) => c.detected).length;
  const sortedClients = [...supportedClients].sort((a, b) => {
    if (a.detected !== b.detected) return a.detected ? -1 : 1;
    return 0;
  });

  return (
    <div>
      <PageHeader
        title="Clients"
        description="AI applications detected on your Mac and system tools."
        action={
          <Button type="button" variant="outline" size="sm" onClick={load}>
            Re-detect
          </Button>
        }
      />
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <section className="mb-8">
        <h3 className="mb-3 text-sm font-medium text-foreground">
          AI clients ({detectedCount} detected of {supportedClients.length})
        </h3>
        <Card className="overflow-hidden py-0">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border text-left text-muted-foreground">
                <th className="px-4 py-3 font-medium">Client</th>
                <th className="px-4 py-3 font-medium">Detection</th>
                <th className="px-4 py-3 font-medium">Sync</th>
                <th className="px-4 py-3 font-medium">Configuration</th>
                <th className="px-4 py-3 font-medium">Path</th>
              </tr>
            </thead>
            <tbody>
              {sortedClients.map((client) => (
                <tr
                  key={client.client_id}
                  className="border-b border-border/50 last:border-0"
                >
                  <td className="px-4 py-3 font-medium text-foreground">
                    <ClientName
                      clientId={client.client_id}
                      name={client.display_name}
                    />
                  </td>
                  <td className="px-4 py-3 text-foreground">
                    {client.detected ? "Detected" : "Not detected"}
                  </td>
                  <td className="px-4 py-3 text-foreground">
                    {syncLabel(client)}
                  </td>
                  <td className="px-4 py-3 text-foreground">
                    {configLabel(client)}
                  </td>
                  <td
                    className="max-w-xs truncate px-4 py-3 text-xs text-muted-foreground"
                    title={client.config_path ?? undefined}
                  >
                    {client.config_path ?? "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      </section>

      <section>
        <h3 className="mb-3 text-sm font-medium text-foreground">
          System dependencies
        </h3>
        <Card className="overflow-hidden py-0">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border text-left text-muted-foreground">
                <th className="px-4 py-3 font-medium">Tool</th>
                <th className="px-4 py-3 font-medium">Status</th>
                <th className="px-4 py-3 font-medium">Path</th>
              </tr>
            </thead>
            <tbody>
              {deps.map((dep) => (
                <tr
                  key={dep.name}
                  className="border-b border-border/50 last:border-0"
                >
                  <td className="px-4 py-3 font-medium text-foreground">
                    {dep.name}
                  </td>
                  <td className="px-4 py-3 text-foreground">
                    {dep.available ? "Available" : "Not found"}
                  </td>
                  <td className="px-4 py-3 text-xs text-muted-foreground">
                    {dep.path ?? "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      </section>
    </div>
  );
}
