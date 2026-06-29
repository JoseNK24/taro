use std::path::PathBuf;

use crate::adapters::{
    backup_file, expand_home_path, read_servers_from_path, remove_server_from_path,
    write_server_to_path, AdapterResult, ClientAdapter,
};
use crate::models::{DetectionResult, McpServer};

pub struct JsonMcpAdapter {
    id: String,
    display_name: String,
    config_path: PathBuf,
    app_paths: Vec<PathBuf>,
    cli_commands: Vec<String>,
}

impl JsonMcpAdapter {
    pub fn new(
        id: &str,
        display_name: &str,
        config_path: &str,
        app_paths: &[&str],
        cli_commands: &[&str],
    ) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            config_path: expand_home_path(config_path),
            app_paths: app_paths.iter().map(|p| PathBuf::from(*p)).collect(),
            cli_commands: cli_commands.iter().map(|c| c.to_string()).collect(),
        }
    }
}

impl ClientAdapter for JsonMcpAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    fn detect(&self) -> DetectionResult {
        let app_exists = self.app_paths.iter().any(|p| p.exists());
        let cli_exists = self.cli_commands.iter().any(|cmd| command_exists(cmd));
        DetectionResult {
            client_id: self.id.clone(),
            display_name: self.display_name.clone(),
            detected: app_exists || cli_exists,
            config_path: Some(self.config_path.display().to_string()),
            config_exists: self.config_path.exists(),
            sync_supported: true,
        }
    }

    fn read_servers(&self) -> AdapterResult<Vec<McpServer>> {
        read_servers_from_path(&self.config_path)
    }

    fn write_server(&self, server: &McpServer) -> AdapterResult<()> {
        write_server_to_path(&self.config_path, server)
    }

    fn remove_server(&self, server_id: &str) -> AdapterResult<()> {
        remove_server_from_path(&self.config_path, server_id)
    }

    fn backup_config(&self) -> AdapterResult<PathBuf> {
        backup_file(&self.config_path)
    }
}

pub fn command_exists(cmd: &str) -> bool {
    if std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    let Some(home) = dirs::home_dir() else {
        return false;
    };

    [
        home.join(".opencode/bin").join(cmd),
        home.join("bin").join(cmd),
        home.join(".local/bin").join(cmd),
    ]
    .iter()
    .any(|path| path.is_file())
}
