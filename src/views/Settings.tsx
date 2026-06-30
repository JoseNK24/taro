import { useState } from "react";
import { PageHeader } from "../components/Sidebar";
import { AppearancePanel } from "../components/settings/AppearancePanel";
import { ConnectionsPanel } from "../components/settings/ConnectionsPanel";
import { GeneralPanel } from "../components/settings/GeneralPanel";
import { SettingsLayout } from "../components/settings/SettingsLayout";
import { Secrets } from "./Secrets";
import type { SettingsSection } from "../types";

export function Settings() {
  const [section, setSection] = useState<SettingsSection>("connections");

  return (
    <div>
      <PageHeader
        title="Settings"
        description="Preferences, harness connections, and secrets."
      />
      <SettingsLayout active={section} onNavigate={setSection}>
        {section === "general" && <GeneralPanel />}
        {section === "appearance" && <AppearancePanel />}
        {section === "connections" && <ConnectionsPanel />}
        {section === "secrets" && <Secrets embedded />}
      </SettingsLayout>
    </div>
  );
}
