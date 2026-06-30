import { cn } from "@/lib/utils";
import { getClientLabel, type SupportedClientId } from "@/lib/clients";
import { ClientLogo } from "./ClientLogo";

const MAX_VISIBLE_CLIENTS = 4;

interface ClientLogoStackProps {
  clientIds: SupportedClientId[];
  className?: string;
  logoClassName?: string;
}

export function ClientLogoStack({
  clientIds,
  className,
  logoClassName = "size-4",
}: ClientLogoStackProps) {
  if (clientIds.length === 0) return null;

  const visible = clientIds.slice(0, MAX_VISIBLE_CLIENTS);
  const overflow = clientIds.length - visible.length;
  const labels = clientIds.map(getClientLabel).join(", ");

  return (
    <div
      className={cn("flex min-w-0 items-center gap-1.5", className)}
      title={labels}
      aria-label={labels}
    >
      {visible.map((id) => (
        <ClientLogo
          key={id}
          clientId={id}
          title={getClientLabel(id)}
          className={cn("shrink-0 rounded-sm ring-1 ring-border/60", logoClassName)}
        />
      ))}
      {overflow > 0 && (
        <span className="shrink-0 text-xs text-muted-foreground">+{overflow}</span>
      )}
    </div>
  );
}
