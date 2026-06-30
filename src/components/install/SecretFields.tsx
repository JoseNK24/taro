import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export type SecretFieldItem = {
  key: string;
  label: string;
  connected?: boolean;
};

interface SecretFieldsProps {
  fields: SecretFieldItem[];
  values: Record<string, string>;
  onChange: (values: Record<string, string>) => void;
}

export function SecretFields({ fields, values, onChange }: SecretFieldsProps) {
  return (
    <div className="space-y-4">
      {fields.map((field) => (
        <div key={field.key} className="space-y-2">
          <Label htmlFor={field.key}>{field.label}</Label>
          {field.connected ? (
            <p className="text-sm text-emerald-600 dark:text-emerald-400">
              Connected
            </p>
          ) : (
            <Input
              id={field.key}
              type="password"
              placeholder="Enter your API key"
              value={values[field.key] ?? ""}
              onChange={(e) =>
                onChange({ ...values, [field.key]: e.target.value })
              }
            />
          )}
        </div>
      ))}
    </div>
  );
}
