import { Button } from "@/components/ui/button";
import type { NavSection } from "../types";

const NAV_ITEMS: { id: NavSection; label: string }[] = [
  { id: "discover", label: "Discover" },
  { id: "installed", label: "Installed" },
  { id: "settings", label: "Settings" },
];

interface SidebarProps {
  active: NavSection;
  onNavigate: (section: NavSection) => void;
}

export function Sidebar({ active, onNavigate }: SidebarProps) {
  return (
    <aside className="relative flex w-52 shrink-0 flex-col border-r border-sidebar-border/70 bg-sidebar/85 pt-10 text-sidebar-foreground shadow-[inset_-1px_0_0_color-mix(in_oklch,var(--sidebar-foreground),transparent_94%)] backdrop-blur-xl dark:border-sidebar-border/50 dark:bg-sidebar/90 dark:backdrop-blur-xl">
      <div
        data-tauri-drag-region
        aria-hidden="true"
        className="absolute top-0 right-0 left-0 h-10"
      />
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
