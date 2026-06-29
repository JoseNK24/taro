import { useCallback, useEffect, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { FirstRunOnboarding } from "./components/FirstRunOnboarding";
import { Discover } from "./views/Discover";
import { Installed } from "./views/Installed";
import { Clients } from "./views/Clients";
import { Secrets } from "./views/Secrets";
import { Health } from "./views/Health";
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
      <div className="flex min-h-screen items-center justify-center bg-neutral-100">
        <p className="text-sm text-neutral-500">Cargando Taro…</p>
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
        return <Discover onInstalled={() => setSection("installed")} />;
      case "installed":
        return <Installed />;
      case "clients":
        return <Clients />;
      case "secrets":
        return <Secrets />;
      case "health":
        return <Health />;
    }
  };

  return (
    <div className="flex min-h-screen bg-neutral-100">
      <Sidebar active={section} onNavigate={setSection} />
      <main className="flex-1 overflow-y-auto p-8">{renderView()}</main>
    </div>
  );
}

export default App;
