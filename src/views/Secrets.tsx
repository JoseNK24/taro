import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ErrorBanner, EmptyState, LoadingState } from "../components/Feedback";
import { PageHeader } from "../components/Sidebar";
import {
  getSecretsStatus,
  removeSecret,
  saveSecret,
} from "../hooks/useTauri";
import type { SecretStatus } from "../types";

export function Secrets({ embedded = false }: { embedded?: boolean }) {
  const [secrets, setSecrets] = useState<SecretStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<SecretStatus | null>(null);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setSecrets(await getSecretsStatus());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const grouped = secrets.reduce<Record<string, SecretStatus[]>>((acc, s) => {
    (acc[s.integration_id] ??= []).push(s);
    return acc;
  }, {});

  const handleSave = async () => {
    if (!editing || !value) return;
    setBusy(true);
    try {
      await saveSecret(editing.integration_id, editing.secret_key, value);
      setEditing(null);
      setValue("");
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleDelete = async (s: SecretStatus) => {
    if (!confirm(`Remove the secret for ${s.integration_name}?`)) return;
    setBusy(true);
    try {
      await removeSecret(s.integration_id, s.secret_key);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (loading) return <LoadingState />;

  return (
    <div>
      {!embedded && (
        <PageHeader
          title="Secrets"
          description="Credentials stored securely in the macOS Keychain."
        />
      )}
      {error && <ErrorBanner message={error} onDismiss={() => setError(null)} />}

      {secrets.length === 0 ? (
        <EmptyState
          title="No secrets configured"
          description="Integrations that require API keys will appear here."
        />
      ) : (
        <div className="space-y-4">
          {Object.entries(grouped).map(([integrationId, items]) => (
            <Card key={integrationId}>
              <CardHeader>
                <CardTitle>{items[0].integration_name}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {items.map((s) => (
                  <div
                    key={s.secret_key}
                    className="flex items-center justify-between rounded-lg bg-muted px-3 py-2"
                  >
                    <div>
                      <p className="text-sm font-medium text-foreground">
                        {s.label}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {s.connected ? (
                          <span className="text-emerald-600 dark:text-emerald-400">Connected</span>
                        ) : (
                          <span className="text-amber-600 dark:text-amber-400">API Key required</span>
                        )}
                      </p>
                    </div>
                    <div className="flex gap-2">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={busy}
                        onClick={() => {
                          setEditing(s);
                          setValue("");
                        }}
                      >
                        {s.connected ? "Update" : "Add"}
                      </Button>
                      {s.connected && (
                        <Button
                          type="button"
                          variant="destructive"
                          size="sm"
                          disabled={busy}
                          onClick={() => handleDelete(s)}
                        >
                          Remove
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      <Dialog open={editing !== null} onOpenChange={(open) => !open && setEditing(null)}>
        <DialogContent className="sm:max-w-sm">
          {editing && (
            <>
              <DialogHeader>
                <DialogTitle>{editing.label}</DialogTitle>
                <DialogDescription>{editing.integration_name}</DialogDescription>
              </DialogHeader>
              <div className="space-y-2">
                <Label htmlFor="secret-value">API Key</Label>
                <Input
                  id="secret-value"
                  type="password"
                  autoFocus
                  placeholder="Enter your API key"
                  value={value}
                  onChange={(e) => setValue(e.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  The value will not be shown after saving.
                </p>
              </div>
              <DialogFooter>
                <Button type="button" variant="outline" onClick={() => setEditing(null)}>
                  Cancel
                </Button>
                <Button
                  type="button"
                  disabled={!value || busy}
                  onClick={handleSave}
                >
                  Save
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
