import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import type { SettingsSection } from "../../types";

const SETTINGS_NAV: { id: SettingsSection; label: string }[] = [
  { id: "general", label: "General" },
  { id: "connections", label: "Connections" },
  { id: "secrets", label: "Secrets" },
];

interface SettingsLayoutProps {
  active: SettingsSection;
  onNavigate: (section: SettingsSection) => void;
  children: ReactNode;
}

export function SettingsLayout({
  active,
  onNavigate,
  children,
}: SettingsLayoutProps) {
  return (
    <div className="flex gap-8">
      <nav className="flex w-40 shrink-0 flex-col gap-0.5">
        {SETTINGS_NAV.map((item) => (
          <Button
            key={item.id}
            type="button"
            variant={active === item.id ? "secondary" : "ghost"}
            className="justify-start"
            onClick={() => onNavigate(item.id)}
          >
            {item.label}
          </Button>
        ))}
      </nav>
      <div className="min-w-0 flex-1">{children}</div>
    </div>
  );
}
