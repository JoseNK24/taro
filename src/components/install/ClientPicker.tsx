import { Checkbox } from "@/components/ui/checkbox";
import { ClientName } from "@/components/ClientLogo";
import type { DetectionResult } from "../../types";

interface ClientPickerProps {
  clients: DetectionResult[];
  selectedClients: Set<string>;
  onSelectionChange: (next: Set<string>) => void;
  emptyMessage?: string;
}

export function ClientPicker({
  clients,
  selectedClients,
  onSelectionChange,
  emptyMessage = "No sync-capable clients were detected. Check Settings → Connections.",
}: ClientPickerProps) {
  if (clients.length === 0) {
    return (
      <p className="text-sm text-amber-600 dark:text-amber-400">{emptyMessage}</p>
    );
  }

  return (
    <div className="space-y-2">
      {clients.map((c) => (
        <label
          key={c.client_id}
          className="flex items-center gap-2 rounded-lg border border-border px-3 py-2"
        >
          <Checkbox
            checked={selectedClients.has(c.client_id)}
            onCheckedChange={(checked) => {
              const next = new Set(selectedClients);
              if (checked) next.add(c.client_id);
              else next.delete(c.client_id);
              onSelectionChange(next);
            }}
          />
          <ClientName
            clientId={c.client_id}
            name={c.display_name}
            className="text-sm"
          />
        </label>
      ))}
    </div>
  );
}
