import { useCallback, useEffect, useState } from "react";
import { Skeleton } from "@/components/ui/skeleton";
import { Sidebar } from "./components/Sidebar";
import { FirstRunOnboarding } from "./components/FirstRunOnboarding";
import { Discover } from "./views/Discover";
import { PluginsDiscover } from "./views/PluginsDiscover";
import { Installed } from "./views/Installed";
import { Health } from "./views/Health";
import { Settings } from "./views/Settings";
import { getFirstRunStatus } from "./hooks/useTauri";
import type { NavSection } from "./types";

function App() {
  const [section, setSection] = useState<NavSection>("discover");
  const [firstRun, setFirstRun] = useState<boolean | null>(null);

  const checkFirstRun = useCallback(async () => {
    try {
      const status = await getFirstRunStatus();
      setFirstRun(!status.completed);
    } catch {
      setFirstRun(false);
    }
  }, []);

  useEffect(() => {
    checkFirstRun();
  }, [checkFirstRun]);

  if (firstRun === null) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-3">
          <Skeleton className="h-4 w-32" />
          <p className="text-sm text-muted-foreground">Loading Taro…</p>
        </div>
      </div>
    );
  }

  if (firstRun) {
    return (
      <FirstRunOnboarding
        onComplete={() => {
          setFirstRun(false);
        }}
      />
    );
  }

  const renderView = () => {
    switch (section) {
      case "discover":
        return (
          <Discover
            onInstalled={() => setSection("installed")}
            onOpenSettings={() => setSection("settings")}
          />
        );
      case "plugins":
        return <PluginsDiscover onInstalled={() => setSection("installed")} />;
      case "installed":
        return <Installed />;
      case "health":
        return <Health />;
      case "settings":
        return <Settings />;
    }
  };

  return (
    <div className="flex min-h-screen bg-background">
      <Sidebar active={section} onNavigate={setSection} />
      <main className="flex-1 overflow-y-auto p-8">{renderView()}</main>
    </div>
  );
}

export default App;
