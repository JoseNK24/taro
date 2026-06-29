pub mod extension_cli;
pub mod marketplace_cli;
pub mod repo_files;
pub mod unsupported;

use thiserror::Error;

use crate::models::{PluginCatalogEntry, PluginInstallStrategy};

#[derive(Debug, Error)]
pub enum PluginAdapterError {
    #[error("{0}")]
    Message(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type PluginAdapterResult<T> = Result<T, PluginAdapterError>;

impl PluginAdapterError {
    fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }
}

pub fn install_plugin_for_client(
    entry: &PluginCatalogEntry,
    client_id: &str,
    strategy: &PluginInstallStrategy,
) -> PluginAdapterResult<()> {
    match strategy {
        PluginInstallStrategy::MarketplaceCli => {
            marketplace_cli::install(entry, client_id).map_err(|e| PluginAdapterError::msg(e))
        }
        PluginInstallStrategy::RepoFiles { files } => {
            repo_files::install(entry, files).map_err(|e| PluginAdapterError::msg(e))
        }
        PluginInstallStrategy::ExtensionCli { package } => {
            extension_cli::install(package).map_err(|e| PluginAdapterError::msg(e))
        }
        PluginInstallStrategy::ComingSoon => Err(PluginAdapterError::msg(
            "Este cliente aún no soporta plugins automáticos",
        )),
        PluginInstallStrategy::Unsupported => Err(PluginAdapterError::msg(
            "Sincronización de plugins no disponible para este cliente",
        )),
    }
}

pub fn uninstall_plugin_for_client(
    entry: &PluginCatalogEntry,
    client_id: &str,
    strategy: &PluginInstallStrategy,
) -> PluginAdapterResult<()> {
    match strategy {
        PluginInstallStrategy::MarketplaceCli => {
            marketplace_cli::uninstall(entry, client_id).map_err(|e| PluginAdapterError::msg(e))
        }
        PluginInstallStrategy::RepoFiles { files } => {
            repo_files::uninstall(files).map_err(|e| PluginAdapterError::msg(e))
        }
        PluginInstallStrategy::ExtensionCli { package } => {
            extension_cli::uninstall(package).map_err(|e| PluginAdapterError::msg(e))
        }
        PluginInstallStrategy::ComingSoon | PluginInstallStrategy::Unsupported => Ok(()),
    }
}

pub fn strategy_for_client<'a>(
    entry: &'a PluginCatalogEntry,
    client_id: &str,
) -> Option<&'a PluginInstallStrategy> {
    entry.client_install.get(client_id)
}

pub fn is_strategy_supported(strategy: &PluginInstallStrategy) -> bool {
    !matches!(
        strategy,
        PluginInstallStrategy::ComingSoon | PluginInstallStrategy::Unsupported
    )
}

pub fn effective_strategy(
    entry: &PluginCatalogEntry,
    client_id: &str,
    sync_supported: bool,
) -> PluginInstallStrategy {
    match entry.client_install.get(client_id) {
        Some(strategy) => strategy.clone(),
        None if sync_supported => PluginInstallStrategy::Unsupported,
        None => PluginInstallStrategy::Unsupported,
    }
}

pub fn check_plugin_dependencies(strategy: &PluginInstallStrategy) -> Result<(), String> {
    use crate::adapters::json_mcp::command_exists;

    match strategy {
        PluginInstallStrategy::MarketplaceCli => {
            if !command_exists("git") {
                return Err("Se requiere git en PATH para instalar plugins".to_string());
            }
        }
        PluginInstallStrategy::RepoFiles { .. } => {
            if !command_exists("git") {
                return Err("Se requiere git en PATH para instalar plugins".to_string());
            }
        }
        PluginInstallStrategy::ExtensionCli { .. } => {}
        _ => {}
    }
    Ok(())
}

pub fn check_cli_for_client(client_id: &str, strategy: &PluginInstallStrategy) -> Result<(), String> {
    use crate::adapters::json_mcp::command_exists;

    match strategy {
        PluginInstallStrategy::MarketplaceCli => {
            let cmd = match client_id {
                "claude-code" => "claude",
                "codex" => "codex",
                other => {
                    return Err(format!("Cliente no soportado para marketplace CLI: {other}"));
                }
            };
            if !command_exists(cmd) {
                return Err(format!(
                    "No se encontró el comando '{cmd}' en PATH. Instálalo antes de continuar."
                ));
            }
        }
        PluginInstallStrategy::ExtensionCli { .. } => {
            if !command_exists("gemini") {
                return Err(
                    "No se encontró el comando 'gemini' en PATH. Instala Gemini CLI antes de continuar."
                        .to_string(),
                );
            }
        }
        _ => {}
    }
    Ok(())
}
