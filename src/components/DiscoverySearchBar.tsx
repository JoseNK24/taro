import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { DiscoverySort } from "../types";

interface DiscoverySearchBarProps {
  query: string;
  sort: DiscoverySort;
  onQueryChange: (query: string) => void;
  onSortChange: (sort: DiscoverySort) => void;
}

const SORT_OPTIONS: { value: DiscoverySort; label: string }[] = [
  { value: "popular", label: "Popular" },
  { value: "stars", label: "Stars" },
  { value: "recent", label: "Recent" },
];

export function DiscoverySearchBar({
  query,
  sort,
  onQueryChange,
  onSortChange,
}: DiscoverySearchBarProps) {
  return (
    <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <Input
        type="search"
        placeholder="Search community MCPs…"
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
        className="sm:max-w-md"
      />
      <div className="flex gap-1 rounded-lg border border-border p-1">
        {SORT_OPTIONS.map((opt) => (
          <Button
            key={opt.value}
            type="button"
            variant={sort === opt.value ? "default" : "ghost"}
            size="sm"
            onClick={() => onSortChange(opt.value)}
          >
            {opt.label}
          </Button>
        ))}
      </div>
    </div>
  );
}
