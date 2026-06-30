import { cn } from "@/lib/utils";

type WindowDragRegionProps = {
  /** `fixed` for fullscreen overlays; `absolute` inside a positioned parent */
  variant?: "fixed" | "absolute";
  className?: string;
};

/** Invisible drag handle — no background, integrated into existing chrome. */
export function WindowDragRegion({
  variant = "absolute",
  className,
}: WindowDragRegionProps) {
  return (
    <div
      data-tauri-drag-region
      aria-hidden="true"
      className={cn(
        "h-10 shrink-0",
        variant === "fixed"
          ? "fixed top-0 right-0 left-0 z-50"
          : "absolute top-0 right-0 left-0 z-10",
        className,
      )}
    />
  );
}
