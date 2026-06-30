import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { GitHubLink } from "./GitHubLink";
import type { DiscoveredMcpEntry } from "../types";

const MAX_CARD_TAGS = 3;
const REDUNDANT_CARD_TAGS = new Set(["github", "ai", "mcp"]);

function cardTagLabels(entry: DiscoveredMcpEntry): string[] {
  const reserved = entry.github_stars > 0 ? 1 : 0;
  const limit = MAX_CARD_TAGS - reserved;

  return entry.tags
    .filter((tag) => !REDUNDANT_CARD_TAGS.has(tag.toLowerCase()))
    .slice(0, limit);
}

interface DiscoveredMcpCardProps {
  entry: DiscoveredMcpEntry;
  canInstall: boolean;
  onInstallClick: (entry: DiscoveredMcpEntry) => void;
}

export function DiscoveredMcpCard({
  entry,
  canInstall,
  onInstallClick,
}: DiscoveredMcpCardProps) {
  const tagLabels = cardTagLabels(entry);
  const showStars = entry.github_stars > 0;
  const hasTags = showStars || tagLabels.length > 0;

  return (
    <Card className="flex h-full flex-col">
      <CardHeader className="grid-rows-[auto_auto_auto]">
        <CardTitle>{entry.name}</CardTitle>
        {entry.github_url && (
          <CardAction>
            <GitHubLink url={entry.github_url} />
          </CardAction>
        )}
        {hasTags && (
          <div className="col-span-2 flex flex-nowrap items-center gap-2 overflow-hidden">
            {showStars && (
              <Badge
                variant="secondary"
                className="shrink-0 bg-amber-500/10 text-amber-800 dark:bg-amber-500/20 dark:text-amber-400"
              >
                ★ {entry.github_stars.toLocaleString()}
              </Badge>
            )}
            {tagLabels.map((tag) => (
              <Badge key={tag} variant="outline" className="shrink-0">
                {tag}
              </Badge>
            ))}
          </div>
        )}
        <CardDescription className="col-span-2 line-clamp-3">
          {entry.description}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex-1 space-y-2">
        {entry.install_hint && (
          <p className="text-xs text-muted-foreground">
            Install: {entry.install_hint}
          </p>
        )}
      </CardContent>
      <CardFooter className="mt-auto border-t-0 bg-transparent">
        <Button
          type="button"
          className="w-full"
          disabled={!canInstall}
          title={
            canInstall
              ? undefined
              : "Connect an agent-capable harness in Settings → Connections"
          }
          onClick={() => onInstallClick(entry)}
        >
          Install
        </Button>
      </CardFooter>
    </Card>
  );
}
