import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import type { DependencyStatus, DetectionResult } from "../../types";

function syncLabel(client: DetectionResult): string {
  if (!client.detected) return "—";
  if (client.sync_supported) return "Supported";
  return "Coming soon";
}

function configLabel(client: DetectionResult): string {
  if (!client.config_path) return "—";
  return client.config_exists ? "Found" : "No configuration";
}

interface McpClientsTableProps {
  clients: DetectionResult[];
  deps: DependencyStatus[];
  onRefresh: () => void;
  loading?: boolean;
}

export function McpClientsTable({
  clients,
  deps,
  onRefresh,
  loading,
}: McpClientsTableProps) {
  const detectedCount = clients.filter((c) => c.detected).length;
  const sortedClients = [...clients].sort((a, b) => {
    if (a.detected !== b.detected) return a.detected ? -1 : 1;
    return a.display_name.localeCompare(b.display_name, "en");
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-foreground">MCP Clients</h3>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onRefresh}
          disabled={loading}
        >
          Re-detect
        </Button>
      </div>

      <p className="text-xs text-muted-foreground">
        {detectedCount} detected of {clients.length} supported clients
      </p>

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
                  {client.display_name}
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

      <div>
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
      </div>
    </div>
  );
}
