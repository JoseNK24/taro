const STATUS_LABELS: Record<string, { label: string; className: string }> = {
  connected: {
    label: "Conectado",
    className: "bg-emerald-50 text-emerald-700 ring-emerald-600/20",
  },
  needs_secret: {
    label: "API Key requerida",
    className: "bg-amber-50 text-amber-700 ring-amber-600/20",
  },
  error: {
    label: "Error",
    className: "bg-red-50 text-red-700 ring-red-600/20",
  },
  disabled: {
    label: "Desactivado",
    className: "bg-neutral-100 text-neutral-600 ring-neutral-500/20",
  },
  update_available: {
    label: "Actualización disponible",
    className: "bg-blue-50 text-blue-700 ring-blue-600/20",
  },
};

interface StatusBadgeProps {
  status: string;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const config = STATUS_LABELS[status] ?? {
    label: status,
    className: "bg-neutral-100 text-neutral-600 ring-neutral-500/20",
  };

  return (
    <span
      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ring-1 ring-inset ${config.className}`}
    >
      {config.label}
    </span>
  );
}
