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
  source?: string;
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

export type MissingDependency = {
  name: string;
  install_label: string;
  installable: boolean;
};

export type ExistingServer = {
  client_id: string;
  client_name: string;
  server_id: string;
  command: string;
  managed: boolean;
};

export type FirstRunStatus = {
  completed: boolean;
  detected_clients: DetectionResult[];
  existing_servers: ExistingServer[];
};

export type InstallResult = {
  installation_id: string;
  success: boolean;
  message: string;
};

export type ClientOperationResult = {
  client_id: string;
  success: boolean;
  message: string;
};

export type UninstallResult = {
  success: boolean;
  message: string;
  client_results: ClientOperationResult[];
};

export type ClientTargetRecord = {
  installation_id: string;
  client_id: string;
  enabled: boolean;
};

export type NavSection =
  | "discover"
  | "plugins"
  | "installed"
  | "health"
  | "settings";

export type SettingsSection = "general" | "appearance" | "connections" | "secrets";

export type DiscoveredMcpEntry = {
  id: string;
  name: string;
  description: string;
  tags: string[];
  github_url?: string | null;
  homepage_url?: string | null;
  registry_url?: string | null;
  github_stars: number;
  github_forks: number;
  github_updated_at?: string | null;
  discovered_at: string;
  sources: string[];
  popularity_score: number;
  install_hint?: string | null;
};

export type DiscoverySearchResult = {
  entries: DiscoveredMcpEntry[];
  total: number;
};

export type DiscoveryStatus = {
  last_synced_at?: string | null;
  total_count: number;
  sync_in_progress: boolean;
};

export type DiscoverySyncStats = {
  added: number;
  updated: number;
  errors: number;
};

export type DiscoverySort = "popular" | "stars" | "recent";

export type PluginMarketplace = {
  source: string;
  plugin_id: string;
  marketplace_name: string;
};

export type PluginInstallStrategy =
  | { method: "marketplace_cli" }
  | { method: "repo_files"; files: string[] }
  | { method: "extension_cli"; package: string }
  | { method: "coming_soon" }
  | { method: "unsupported" };

export type PluginCatalogEntry = {
  id: string;
  name: string;
  description: string;
  tags: string[];
  icon: string;
  coming_soon?: boolean;
  github_url: string;
  github_stars: number;
  marketplace: PluginMarketplace;
  client_install: Record<string, PluginInstallStrategy>;
};

export type PluginInstallationRecord = {
  id: string;
  plugin_id: string;
  enabled: boolean;
  status: string;
  installed_at: string;
  error_message?: string | null;
};

export type PluginClientTargetRecord = {
  installation_id: string;
  client_id: string;
  enabled: boolean;
  status: string;
  error_message?: string | null;
};

export type PluginClientInstallResult = {
  client_id: string;
  success: boolean;
  message: string;
};

export type PluginInstallResult = {
  installation_id: string;
  success: boolean;
  message: string;
  client_results: PluginClientInstallResult[];
};

export type HarnessInstanceRecord = {
  id: string;
  driver_kind: string;
  display_name: string;
  enabled: boolean;
  is_default_install_agent: boolean;
  config_json: string;
  created_at: string;
};

export type HarnessSnapshot = {
  instance_id: string;
  driver_kind: string;
  display_name: string;
  enabled: boolean;
  detected: boolean;
  version?: string | null;
  auth_status: string;
  agent_capable: boolean;
  probe_detail?: string | null;
  probed_at: string;
};

export type HarnessDriverInfo = {
  kind: string;
  display_name: string;
  detected: boolean;
  agent_capable: boolean;
  install_hint?: string | null;
};

export type ResolvedMcpConfig = {
  command: string;
  args: string[];
  env_keys: string[];
  requires: string[];
  confidence: string;
  notes?: string | null;
};

export type CommunityInstallJob = {
  id: string;
  discovered_mcp_id: string;
  harness_instance_id: string;
  status: string;
  resolved_config?: ResolvedMcpConfig | null;
  agent_log: string;
  error?: string | null;
  discovered_name?: string | null;
};

export type CommunityInstallMeta = {
  installation_id: string;
  discovered_mcp_id: string;
  harness_instance_id: string;
  resolved_server: {
    id: string;
    command: string;
    args: string[];
    env: Record<string, string>;
  };
  env_keys: string[];
  agent_log?: string | null;
};

export type CommunityInstallProgressEvent = {
  job_id: string;
  message: string;
};
