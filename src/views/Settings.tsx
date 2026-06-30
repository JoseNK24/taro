import { useState } from "react";
import { PageHeader } from "../components/PageHeader";
import { ConnectionsPanel } from "../components/settings/ConnectionsPanel";
import { PreferencesPanel } from "../components/settings/PreferencesPanel";
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
        {section === "preferences" && <PreferencesPanel />}
        {section === "connections" && <ConnectionsPanel />}
        {section === "secrets" && <Secrets embedded />}
      </SettingsLayout>
    </div>
  );
}
