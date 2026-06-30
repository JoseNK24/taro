import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ClientTargetRecord,
  CommunityInstallJob,
  CommunityInstallMeta,
  CommunityInstallProgressEvent,
  DependencyStatus,
  DetectionResult,
  DiscoveredMcpEntry,
  DiscoverySearchResult,
  DiscoverySort,
  DiscoveryStatus,
  DiscoverySyncStats,
  FirstRunStatus,
  HarnessDriverInfo,
  HarnessInstanceRecord,
  HarnessSnapshot,
  HealthCheckRecord,
  InstallResult,
  InstallationRecord,
  IntegrationCatalogEntry,
  MissingDependency,
  PluginCatalogEntry,
  PluginClientTargetRecord,
  PluginInstallResult,
  PluginInstallationRecord,
  ResolvedMcpConfig,
  SecretStatus,
  UninstallResult,
} from "../types";

export async function getCatalog(): Promise<IntegrationCatalogEntry[]> {
  return invoke("get_catalog");
}

export async function getInstallations(): Promise<InstallationRecord[]> {
  return invoke("get_installations");
}

export async function installIntegration(
  integrationId: string,
  clientIds: string[],
): Promise<InstallResult> {
  return invoke("install_integration", {
    integrationId,
    clientIds,
  });
}

export async function uninstallIntegration(
  installationId: string,
): Promise<UninstallResult> {
  return invoke("uninstall_integration", { installationId });
}

export async function toggleInstallation(
  installationId: string,
  enabled: boolean,
): Promise<void> {
  return invoke("toggle_installation", { installationId, enabled });
}

export async function syncInstallation(installationId: string): Promise<void> {
  return invoke("sync_installation", { installationId });
}

export async function setClientTarget(
  installationId: string,
  clientId: string,
  enabled: boolean,
): Promise<void> {
  return invoke("set_client_target", { installationId, clientId, enabled });
}

export async function detectClients(): Promise<DetectionResult[]> {
  return invoke("detect_clients");
}

export async function getDependencies(): Promise<DependencyStatus[]> {
  return invoke("get_dependencies");
}

export async function getFirstRunStatus(): Promise<FirstRunStatus> {
  return invoke("get_first_run_status");
}

export async function completeFirstRun(): Promise<void> {
  return invoke("complete_first_run");
}

export async function getSecretsStatus(): Promise<SecretStatus[]> {
  return invoke("get_secrets_status");
}

export async function saveSecret(
  integrationId: string,
  secretKey: string,
  value: string,
): Promise<void> {
  return invoke("save_secret", { integrationId, secretKey, value });
}

export async function removeSecret(
  integrationId: string,
  secretKey: string,
): Promise<void> {
  return invoke("remove_secret", { integrationId, secretKey });
}

export async function runHealthCheck(
  installationId: string,
): Promise<HealthCheckRecord> {
  return invoke("run_health_check", { installationId });
}

export async function runAllHealthChecks(): Promise<HealthCheckRecord[]> {
  return invoke("run_all_health_checks");
}

export async function getHealthStatus(): Promise<HealthCheckRecord[]> {
  return invoke("get_health_status");
}

export async function getClientTargets(
  installationId: string,
): Promise<ClientTargetRecord[]> {
  return invoke("get_client_targets", { installationId });
}

export async function searchDiscoveredMcps(
  query: string,
  sort: DiscoverySort,
  limit: number,
  offset: number,
): Promise<DiscoverySearchResult> {
  return invoke("search_discovered_mcps", { query, sort, limit, offset });
}

export async function getDiscoveredMcp(
  id: string,
): Promise<DiscoveredMcpEntry | null> {
  return invoke("get_discovered_mcp", { id });
}

export async function syncDiscoveredCatalog(): Promise<DiscoverySyncStats> {
  return invoke("sync_discovered_catalog");
}

export async function getDiscoveryStatus(): Promise<DiscoveryStatus> {
  return invoke("get_discovery_status");
}

export async function getPluginCatalog(): Promise<PluginCatalogEntry[]> {
  return invoke("get_plugin_catalog");
}

export async function getPluginInstallations(): Promise<PluginInstallationRecord[]> {
  return invoke("get_plugin_installations");
}

export async function installPlugin(
  pluginId: string,
  clientIds: string[],
): Promise<PluginInstallResult> {
  return invoke("install_plugin", { pluginId, clientIds });
}

export async function uninstallPlugin(
  installationId: string,
): Promise<UninstallResult> {
  return invoke("uninstall_plugin", { installationId });
}

export async function togglePluginInstallation(
  installationId: string,
  enabled: boolean,
): Promise<void> {
  return invoke("toggle_plugin_installation", { installationId, enabled });
}

export async function getPluginClientTargets(
  installationId: string,
): Promise<PluginClientTargetRecord[]> {
  return invoke("get_plugin_client_targets", { installationId });
}

export async function listHarnessInstances(): Promise<HarnessInstanceRecord[]> {
  return invoke("list_harness_instances");
}

export async function listHarnessDrivers(): Promise<HarnessDriverInfo[]> {
  return invoke("list_harness_drivers");
}

export async function createHarnessInstance(
  driverKind: string,
  displayName: string,
  configJson = "{}",
): Promise<HarnessInstanceRecord> {
  return invoke("create_harness_instance", {
    driverKind,
    displayName,
    configJson,
  });
}

export async function updateHarnessInstance(
  instanceId: string,
  displayName: string,
  enabled: boolean,
  configJson: string,
): Promise<void> {
  return invoke("update_harness_instance", {
    instanceId,
    displayName,
    enabled,
    configJson,
  });
}

export async function deleteHarnessInstance(instanceId: string): Promise<void> {
  return invoke("delete_harness_instance", { instanceId });
}

export async function probeHarnesses(): Promise<HarnessSnapshot[]> {
  return invoke("probe_harnesses");
}

export async function setDefaultInstallAgent(instanceId: string): Promise<void> {
  return invoke("set_default_install_agent", { instanceId });
}

export async function startCommunityInstall(
  discoveredMcpId: string,
  harnessInstanceId: string,
): Promise<CommunityInstallJob> {
  return invoke("start_community_install", {
    discoveredMcpId,
    harnessInstanceId,
  });
}

export async function getCommunityInstallJob(
  jobId: string,
): Promise<CommunityInstallJob | null> {
  return invoke("get_community_install_job", { jobId });
}

export async function cancelCommunityInstall(jobId: string): Promise<void> {
  return invoke("cancel_community_install", { jobId });
}

export async function confirmCommunityInstall(
  jobId: string,
  clientIds: string[],
  secrets: Record<string, string>,
  resolvedOverride?: ResolvedMcpConfig,
): Promise<InstallResult> {
  return invoke("confirm_community_install_cmd", {
    jobId,
    clientIds,
    secrets,
    resolvedOverride: resolvedOverride ?? null,
  });
}

export async function getCommunityInstallMeta(
  installationId: string,
): Promise<CommunityInstallMeta | null> {
  return invoke("get_community_install_meta", { installationId });
}

export async function communityMissingDependencies(
  config: ResolvedMcpConfig,
): Promise<MissingDependency[]> {
  return invoke("community_missing_dependencies", { config });
}

export async function installDependencies(names: string[]): Promise<string> {
  return invoke("install_dependencies_cmd", { names });
}

export async function getSetting(key: string): Promise<string | null> {
  const value = await invoke<string | null>("get_setting", { key });
  return value;
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

export function onCommunityInstallProgress(
  handler: (event: CommunityInstallProgressEvent) => void,
): Promise<() => void> {
  return listen<CommunityInstallProgressEvent>(
    "community_install_progress",
    (e) => handler(e.payload),
  );
}
