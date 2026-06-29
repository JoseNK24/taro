import { useCallback, useEffect, useState } from "react";
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
import { Skeleton } from "@/components/ui/skeleton";
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
      <div className="flex min-h-screen items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-3">
          <Skeleton className="h-4 w-32" />
          <p className="text-sm text-muted-foreground">Preparing Taro…</p>
        </div>
      </div>
    );
  }

  const detected = status.detected_clients.filter((c) => c.detected);

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-lg">
        {step === "welcome" && (
          <>
            <CardHeader>
              <CardTitle className="text-2xl">Welcome to Taro</CardTitle>
              <CardDescription>
                Taro manages your MCP integrations for Cursor, Claude Desktop, and more.
                We'll help you set everything up in a few steps.
              </CardDescription>
            </CardHeader>
            <CardFooter>
              <Button type="button" className="w-full" onClick={() => setStep("scan")}>
                Get started
              </Button>
            </CardFooter>
          </>
        )}

        {step === "scan" && (
          <>
            <CardHeader>
              <CardTitle>Detected clients</CardTitle>
              <CardDescription>
                We scanned your system for compatible applications.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {status.detected_clients.map((c) => (
                <div
                  key={c.client_id}
                  className="flex items-center justify-between rounded-lg border border-border px-3 py-2"
                >
                  <span className="text-sm">{c.display_name}</span>
                  <Badge
                    variant={c.detected ? "default" : "outline"}
                    className={
                      c.detected
                        ? "bg-emerald-500/10 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400"
                        : undefined
                    }
                  >
                    {c.detected ? "Detected" : "Not found"}
                  </Badge>
                </div>
              ))}
              {detected.length === 0 && (
                <p className="text-sm text-amber-600 dark:text-amber-400">
                  No clients were detected. You can still install integrations and configure them later.
                </p>
              )}
            </CardContent>
            <CardFooter>
              <Button type="button" className="w-full" onClick={() => setStep("import")}>
                Continue
              </Button>
            </CardFooter>
          </>
        )}

        {step === "import" && (
          <>
            <CardHeader>
              <CardTitle>Existing configuration</CardTitle>
              <CardDescription>
                MCP servers found in your clients (read-only in v0.1).
              </CardDescription>
            </CardHeader>
            <CardContent>
              {status.existing_servers.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  No configured MCP servers were found.
                </p>
              ) : (
                <ul className="max-h-48 space-y-2 overflow-y-auto">
                  {status.existing_servers.map((s, i) => (
                    <li
                      key={`${s.client_id}-${s.server_id}-${i}`}
                      className="rounded-lg bg-muted px-3 py-2 text-xs"
                    >
                      <span className="font-medium">{s.client_name}</span>
                      <span className="text-muted-foreground"> — {s.server_id}</span>
                      <br />
                      <span className="text-muted-foreground">{s.command}</span>
                    </li>
                  ))}
                </ul>
              )}
            </CardContent>
            <CardFooter>
              <Button type="button" className="w-full" onClick={handleComplete}>
                Go to Taro
              </Button>
            </CardFooter>
          </>
        )}
      </Card>
    </div>
  );
}
