import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-shell";
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
import { CommunityInstallWizard } from "./CommunityInstallWizard";
import {
  listHarnessInstances,
  probeHarnesses,
} from "../hooks/useTauri";
import type { DiscoveredMcpEntry } from "../types";

interface DiscoveredMcpCardProps {
  entry: DiscoveredMcpEntry;
  onOpenSettings?: () => void;
  onInstalled?: () => void;
}

export function DiscoveredMcpCard({
  entry,
  onOpenSettings,
  onInstalled,
}: DiscoveredMcpCardProps) {
  const [wizardOpen, setWizardOpen] = useState(false);
  const [canInstall, setCanInstall] = useState(false);

  const checkHarnesses = useCallback(async () => {
    try {
      const [instances, snapshots] = await Promise.all([
        listHarnessInstances(),
        probeHarnesses(),
      ]);
      const capable = instances.some((inst) => {
        if (!inst.enabled) return false;
        const snap = snapshots.find((s) => s.instance_id === inst.id);
        return snap?.agent_capable && snap.detected;
      });
      setCanInstall(capable);
    } catch {
      setCanInstall(false);
    }
  }, []);

  useEffect(() => {
    checkHarnesses();
  }, [checkHarnesses]);

  const openUrl = async (url: string) => {
    await open(url);
  };

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{entry.name}</CardTitle>
          <CardDescription className="line-clamp-3">{entry.description}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            {entry.github_stars > 0 && (
              <Badge
                variant="secondary"
                className="bg-amber-500/10 text-amber-800 dark:bg-amber-500/20 dark:text-amber-400"
              >
                ★ {entry.github_stars.toLocaleString()}
              </Badge>
            )}
            {entry.sources.map((src) => (
              <Badge key={src} variant="secondary">
                {src}
              </Badge>
            ))}
            {entry.tags.slice(0, 4).map((tag) => (
              <Badge key={tag} variant="outline">
                {tag}
              </Badge>
            ))}
          </div>
          {entry.install_hint && (
            <p className="text-xs text-muted-foreground">
              Install: {entry.install_hint}
            </p>
          )}
        </CardContent>
        <CardFooter className="flex flex-wrap gap-2 border-t-0 bg-transparent">
          <Button
            type="button"
            size="sm"
            disabled={!canInstall}
            title={
              canInstall
                ? undefined
                : "Connect an agent-capable harness in Settings → Connections"
            }
            onClick={() => setWizardOpen(true)}
          >
            Install with agent
          </Button>
          {entry.github_url && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => openUrl(entry.github_url!)}
            >
              Open on GitHub
            </Button>
          )}
          {entry.registry_url && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => openUrl(entry.registry_url!)}
            >
              View in Registry
            </Button>
          )}
          {entry.homepage_url && !entry.github_url && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => openUrl(entry.homepage_url!)}
            >
              Open website
            </Button>
          )}
        </CardFooter>
      </Card>

      <CommunityInstallWizard
        entry={entry}
        open={wizardOpen}
        onClose={() => setWizardOpen(false)}
        onInstalled={() => onInstalled?.()}
        onOpenSettings={onOpenSettings}
      />
    </>
  );
}
