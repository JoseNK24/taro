import { Monitor, Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import type { ComponentType, SVGProps } from "react";
import { Button } from "@/components/ui/button";

type ThemeOption = {
  value: "light" | "dark" | "system";
  label: string;
  description: string;
  Icon: ComponentType<SVGProps<SVGSVGElement>>;
};

const THEME_OPTIONS: ThemeOption[] = [
  {
    value: "light",
    label: "Light",
    description: "Use the light interface.",
    Icon: Sun,
  },
  {
    value: "dark",
    label: "Dark",
    description: "Use the dark interface.",
    Icon: Moon,
  },
  {
    value: "system",
    label: "System",
    description: "Follow your macOS appearance.",
    Icon: Monitor,
  },
];

export function AppearancePanel() {
  const { setTheme, theme } = useTheme();
  const selectedTheme = theme === "light" || theme === "dark" ? theme : "system";

  return (
    <div className="max-w-2xl space-y-6">
      <div className="space-y-1">
        <h3 className="text-sm font-medium text-foreground">Appearance</h3>
        <p className="text-xs text-muted-foreground">
          Choose how Taro adapts to your display.
        </p>
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        {THEME_OPTIONS.map(({ value, label, description, Icon }) => {
          const selected = selectedTheme === value;

          return (
            <Button
              key={value}
              type="button"
              variant={selected ? "secondary" : "outline"}
              className="h-auto justify-start gap-3 px-3 py-3 text-left"
              aria-pressed={selected}
              onClick={() => setTheme(value)}
            >
              <Icon className="size-4" />
              <span className="min-w-0">
                <span className="block text-sm font-medium">{label}</span>
                <span className="block text-xs font-normal text-muted-foreground">
                  {description}
                </span>
              </span>
            </Button>
          );
        })}
      </div>
    </div>
  );
}
