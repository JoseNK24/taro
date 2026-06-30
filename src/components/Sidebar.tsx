import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import type { NavSection } from "../types";

const NAV_ITEMS: { id: NavSection; label: string }[] = [
  { id: "discover", label: "Discover" },
  { id: "plugins", label: "Plugins" },
  { id: "installed", label: "Installed" },
  { id: "health", label: "Health" },
  { id: "settings", label: "Settings" },
];

interface SidebarProps {
  active: NavSection;
  onNavigate: (section: NavSection) => void;
}

export function Sidebar({ active, onNavigate }: SidebarProps) {
  return (
    <aside className="flex w-52 shrink-0 flex-col border-r border-sidebar-border/70 bg-sidebar/55 pt-10 text-sidebar-foreground shadow-[inset_-1px_0_0_color-mix(in_oklch,var(--sidebar-foreground),transparent_94%)] backdrop-blur-2xl dark:border-sidebar-border/60 dark:bg-sidebar/45">
      <div className="px-5 pb-6">
        <h1 className="text-lg font-semibold tracking-tight text-sidebar-foreground">
          Taro
        </h1>
        <p className="mt-0.5 text-xs text-sidebar-foreground/60">
          MCP & Plugins
        </p>
      </div>
      <nav className="flex flex-1 flex-col gap-0.5 px-3">
        {NAV_ITEMS.map((item) => (
          <Button
            key={item.id}
            type="button"
            variant={active === item.id ? "secondary" : "ghost"}
            className="w-full justify-start"
            onClick={() => onNavigate(item.id)}
          >
            {item.label}
          </Button>
        ))}
      </nav>
    </aside>
  );
}

interface PageHeaderProps {
  title: string;
  description?: string;
  action?: ReactNode;
}

export function PageHeader({ title, description, action }: PageHeaderProps) {
  return (
    <div className="mb-6 flex items-start justify-between gap-4">
      <div>
        <h2 className="text-xl font-semibold text-foreground">{title}</h2>
        {description && (
          <p className="mt-1 text-sm text-muted-foreground">{description}</p>
        )}
      </div>
      {action}
    </div>
  );
}
