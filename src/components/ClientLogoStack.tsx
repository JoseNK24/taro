import { cn } from "@/lib/utils";
import { getClientLabel, type SupportedClientId } from "@/lib/clients";
import { ClientLogo } from "./ClientLogo";

interface ClientLogoStackProps {
  clientIds: SupportedClientId[];
  className?: string;
  logoClassName?: string;
}

export function ClientLogoStack({
  clientIds,
  className,
  logoClassName = "size-5",
}: ClientLogoStackProps) {
  if (clientIds.length === 0) return null;

  const labels = clientIds.map(getClientLabel).join(", ");

  return (
    <div
      className={cn("flex items-center", className)}
      title={labels}
      aria-label={labels}
    >
      {clientIds.map((id, index) => (
        <ClientLogo
          key={id}
          clientId={id}
          title={getClientLabel(id)}
          className={cn(
            "rounded-sm ring-2 ring-card",
            logoClassName,
            index > 0 && "-ml-1.5",
          )}
        />
      ))}
    </div>
  );
}
