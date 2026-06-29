import type { ReactNode } from "react";
import type { NavSection } from "../types";

const NAV_ITEMS: { id: NavSection; label: string }[] = [
  { id: "discover", label: "Descubrir" },
  { id: "installed", label: "Instaladas" },
  { id: "clients", label: "Clientes" },
  { id: "secrets", label: "Secretos" },
  { id: "health", label: "Salud" },
];

interface SidebarProps {
  active: NavSection;
  onNavigate: (section: NavSection) => void;
}

export function Sidebar({ active, onNavigate }: SidebarProps) {
  return (
    <aside className="flex w-52 shrink-0 flex-col border-r border-neutral-200 bg-neutral-50 pt-10">
      <div className="px-5 pb-6">
        <h1 className="text-lg font-semibold tracking-tight text-neutral-900">
          Taro
        </h1>
        <p className="mt-0.5 text-xs text-neutral-500">
          Integraciones MCP
        </p>
      </div>
      <nav className="flex flex-1 flex-col gap-0.5 px-3">
        {NAV_ITEMS.map((item) => (
          <button
            key={item.id}
            type="button"
            onClick={() => onNavigate(item.id)}
            className={`rounded-lg px-3 py-2 text-left text-sm transition-colors ${
              active === item.id
                ? "bg-white font-medium text-neutral-900 shadow-sm"
                : "text-neutral-600 hover:bg-neutral-100 hover:text-neutral-900"
            }`}
          >
            {item.label}
          </button>
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
        <h2 className="text-xl font-semibold text-neutral-900">{title}</h2>
        {description && (
          <p className="mt-1 text-sm text-neutral-500">{description}</p>
        )}
      </div>
      {action}
    </div>
  );
}
