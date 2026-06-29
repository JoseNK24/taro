export type IntegrationCatalogEntry = {
  id: string;
  name: string;
  description: string;
  tags: string[];
  icon: string;
  coming_soon?: boolean;
  server: {
    command: string;
    args: string[];
    requires: string[];
  };
  secrets: Array<{
    key: string;
    label: string;
    required: boolean;
  }>;
};

export type InstallationRecord = {
  id: string;
  integration_id: string;
  enabled: boolean;
  status: string;
  installed_at: string;
  error_message?: string | null;
};

export type DetectionResult = {
  client_id: string;
  display_name: string;
  detected: boolean;
  config_path?: string | null;
  config_exists: boolean;
  sync_supported: boolean;
};

export type SecretStatus = {
  integration_id: string;
  integration_name: string;
  secret_key: string;
  label: string;
  required: boolean;
  connected: boolean;
};

export type HealthCheckRecord = {
  installation_id: string;
  integration_id: string;
  integration_name: string;
  latency_ms?: number | null;
  ok: boolean;
  checked_at: string;
  detail?: string | null;
};

export type DependencyStatus = {
  name: string;
  available: boolean;
  path?: string | null;
};

export type FirstRunStatus = {
  completed: boolean;
  detected_clients: DetectionResult[];
  existing_servers: Array<{
    client_id: string;
    client_name: string;
    server_id: string;
    command: string;
  }>;
};

export type InstallResult = {
  installation_id: string;
  success: boolean;
  message: string;
};

export type ClientTargetRecord = {
  installation_id: string;
  client_id: string;
  enabled: boolean;
};

export type NavSection =
  | "discover"
  | "installed"
  | "clients"
  | "secrets"
  | "health";
