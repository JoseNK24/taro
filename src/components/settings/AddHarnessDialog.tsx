import { useState } from "react";
import { Button } from "@/components/ui/button";
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
import type { HarnessDriverInfo } from "../../types";

interface AddHarnessDialogProps {
  open: boolean;
  drivers: HarnessDriverInfo[];
  onClose: () => void;
  onAdd: (driverKind: string, displayName: string) => Promise<void>;
}

export function AddHarnessDialog({
  open,
  drivers,
  onClose,
  onAdd,
}: AddHarnessDialogProps) {
  const [step, setStep] = useState<1 | 2>(1);
  const [selectedKind, setSelectedKind] = useState<string | null>(null);
  const [displayName, setDisplayName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectedDriver = drivers.find((d) => d.kind === selectedKind);

  const reset = () => {
    setStep(1);
    setSelectedKind(null);
    setDisplayName("");
    setError(null);
  };

  const handleClose = () => {
    reset();
    onClose();
  };

  const handleContinue = () => {
    if (!selectedDriver) return;
    setDisplayName(selectedDriver.display_name);
    setStep(2);
  };

  const handleAdd = async () => {
    if (!selectedKind || !displayName.trim()) return;
    setBusy(true);
    setError(null);
    try {
      await onAdd(selectedKind, displayName.trim());
      handleClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && handleClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Add harness</DialogTitle>
          <DialogDescription>
            Connect an AI harness installed on your Mac to delegate community MCP
            installs.
          </DialogDescription>
        </DialogHeader>

        {step === 1 && (
          <div className="max-h-72 space-y-2 overflow-y-auto">
            {drivers.map((driver) => (
              <button
                key={driver.kind}
                type="button"
                disabled={!driver.detected}
                onClick={() => setSelectedKind(driver.kind)}
                className={`w-full rounded-lg border px-3 py-2 text-left text-sm transition-colors ${
                  selectedKind === driver.kind
                    ? "border-primary bg-primary/5"
                    : "border-border"
                } ${!driver.detected ? "opacity-50" : "hover:bg-muted/50"}`}
              >
                <div className="font-medium text-foreground">
                  {driver.display_name}
                </div>
                <div className="text-xs text-muted-foreground">
                  {driver.detected
                    ? driver.agent_capable
                      ? "Detected · can run agent installs"
                      : "Detected · probe only"
                    : driver.install_hint ?? "Not detected on this Mac"}
                </div>
              </button>
            ))}
          </div>
        )}

        {step === 2 && selectedDriver && (
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="harness-name">Display name</Label>
              <Input
                id="harness-name"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
              />
            </div>
            {error && (
              <p className="text-sm text-destructive">{error}</p>
            )}
          </div>
        )}

        <DialogFooter>
          <Button type="button" variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          {step === 1 ? (
            <Button
              type="button"
              disabled={!selectedDriver?.detected}
              onClick={handleContinue}
            >
              Continue
            </Button>
          ) : (
            <Button
              type="button"
              disabled={!displayName.trim() || busy}
              onClick={handleAdd}
            >
              {busy ? "Adding…" : "Add harness"}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
