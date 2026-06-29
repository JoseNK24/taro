import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ClientLogo } from "@/components/ClientLogo";
import {
  CLIENT_LABELS,
  isSupportedClientId,
  type SupportedClientId,
} from "@/lib/clients";
import type { PluginCatalogEntry } from "../types";

function isStrategySupported(
  strategy: PluginCatalogEntry["client_install"][string] | undefined,
): boolean {
  if (!strategy) return false;
  return strategy.method !== "coming_soon" && strategy.method !== "unsupported";
}

function supportedClientIds(entry: PluginCatalogEntry): SupportedClientId[] {
  return Object.entries(entry.client_install)
    .filter(([id, strategy]) => isSupportedClientId(id) && isStrategySupported(strategy))
    .map(([id]) => id as SupportedClientId);
}

function formatStars(stars: number): string {
  if (stars >= 1000) {
    return `${(stars / 1000).toFixed(stars >= 10000 ? 0 : 1)}k`;
  }
  return stars.toString();
}

interface PluginCardProps {
  entry: PluginCatalogEntry;
  installed: boolean;
  onInstall: (entry: PluginCatalogEntry) => void;
}

export function PluginCard({ entry, installed, onInstall }: PluginCardProps) {
  const clients = supportedClientIds(entry);

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-2">
          <CardTitle>{entry.name}</CardTitle>
          {entry.github_stars > 0 && (
            <span className="shrink-0 text-xs text-muted-foreground">
              ★ {formatStars(entry.github_stars)}
            </span>
          )}
        </div>
        <CardDescription className="line-clamp-2">
          {entry.description}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex flex-wrap gap-1">
          {entry.tags.map((tag) => (
            <Badge key={tag} variant="secondary">
              {tag}
            </Badge>
          ))}
        </div>
        {clients.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {clients.map((id) => (
              <Badge key={id} variant="outline" className="gap-1.5 pr-2.5 text-xs">
                <ClientLogo clientId={id} className="size-3.5 rounded-sm" />
                {CLIENT_LABELS[id]}
              </Badge>
            ))}
          </div>
        )}
      </CardContent>
      <CardFooter className="border-t-0 bg-transparent">
        {entry.coming_soon ? (
          <Badge variant="outline" className="w-full justify-center py-2">
            Coming soon
          </Badge>
        ) : installed ? (
          <Badge className="w-full justify-center bg-emerald-500/10 py-2 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400">
            Installed
          </Badge>
        ) : (
          <Button
            type="button"
            className="w-full"
            onClick={() => onInstall(entry)}
          >
            Install
          </Button>
        )}
      </CardFooter>
    </Card>
  );
}

export { CLIENT_LABELS, isStrategySupported, supportedClientIds };
