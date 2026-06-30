import { Button } from "@/components/ui/button";

type SegmentedTabsProps<T extends string> = {
  value: T;
  onChange: (v: T) => void;
  items: { id: T; label: string }[];
  className?: string;
};

export function SegmentedTabs<T extends string>({
  value,
  onChange,
  items,
  className,
}: SegmentedTabsProps<T>) {
  return (
    <div
      className={`mb-6 flex w-fit gap-1 rounded-lg border border-border p-1 ${className ?? ""}`}
    >
      {items.map((item) => (
        <Button
          key={item.id}
          type="button"
          variant={value === item.id ? "default" : "ghost"}
          size="sm"
          onClick={() => onChange(item.id)}
        >
          {item.label}
        </Button>
      ))}
    </div>
  );
}
