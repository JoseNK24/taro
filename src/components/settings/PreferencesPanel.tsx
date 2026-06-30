import { useCallback, useEffect, useState } from "react";
import { Monitor, Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import type { ComponentType, SVGProps } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ErrorBanner } from "../Feedback";
import { getGithubToken, setGithubToken } from "../../hooks/useTauri";

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

export function PreferencesPanel() {
  const { setTheme, theme } = useTheme();
  const selectedTheme =
    theme === "light" || theme === "dark" ? theme : "system";

  const [githubToken, setGithubTokenValue] = useState("");
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const token = await getGithubToken();
      if (token) setGithubTokenValue(token);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleSave = async () => {
    setError(null);
    setSaved(false);
    try {
      await setGithubToken(githubToken.trim());
      setSaved(true);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="max-w-3xl space-y-6">
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <section className="space-y-6">
        <div className="space-y-1">
          <h3 className="text-sm font-medium text-foreground">General</h3>
          <p className="text-xs text-muted-foreground">
            Discovery sync and other app preferences.
          </p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="github-token">GitHub token (optional)</Label>
          <Input
            id="github-token"
            type="password"
            placeholder="ghp_…"
            value={githubToken}
            disabled={loading}
            onChange={(e) => {
              setGithubTokenValue(e.target.value);
              setSaved(false);
            }}
          />
          <p className="text-xs text-muted-foreground">
            Used for discovery sync rate limits. Stored securely in macOS
            Keychain.
          </p>
        </div>

        <Button type="button" onClick={handleSave} disabled={loading}>
          Save preferences
        </Button>
        {saved && (
          <p className="text-sm text-emerald-600 dark:text-emerald-400">
            Preferences saved.
          </p>
        )}
      </section>

      <section className="space-y-6">
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
      </section>
    </div>
  );
}
