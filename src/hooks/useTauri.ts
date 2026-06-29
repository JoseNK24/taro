import { invoke } from "@tauri-apps/api/core";
import type {
  ClientTargetRecord,
  DependencyStatus,
  DetectionResult,
  FirstRunStatus,
  HealthCheckRecord,
  InstallResult,
  InstallationRecord,
  IntegrationCatalogEntry,
  SecretStatus,
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
): Promise<void> {
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
