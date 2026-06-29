import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ErrorBanner } from "../Feedback";
import { getSetting, setSetting } from "../../hooks/useTauri";

export function GeneralPanel() {
  const [githubToken, setGithubToken] = useState("");
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const token = await getSetting("github_token");
      if (token) setGithubToken(token);
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
      await setSetting("github_token", githubToken.trim());
      setSaved(true);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="max-w-lg space-y-6">
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      <div className="space-y-2">
        <Label htmlFor="github-token">GitHub token (optional)</Label>
        <Input
          id="github-token"
          type="password"
          placeholder="ghp_…"
          value={githubToken}
          disabled={loading}
          onChange={(e) => {
            setGithubToken(e.target.value);
            setSaved(false);
          }}
        />
        <p className="text-xs text-muted-foreground">
          Used for discovery sync rate limits. Stored locally in the app database.
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
    </div>
  );
}
