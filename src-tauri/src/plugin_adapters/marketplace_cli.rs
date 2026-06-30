use std::process::Command;

use crate::adapters::json_mcp::command_exists;
use crate::models::PluginCatalogEntry;

fn cli_command(client_id: &str) -> Result<&'static str, String> {
    match client_id {
        "claude-code" => Ok("claude"),
        "codex" => Ok("codex"),
        _ => Err(format!(
            "Cliente no soportado para marketplace CLI: {client_id}"
        )),
    }
}

fn run_cli(client_id: &str, args: &[&str]) -> Result<(), String> {
    let cmd = cli_command(client_id)?;
    if !command_exists(cmd) {
        return Err(format!(
            "No se encontró el comando '{cmd}' en PATH. Instálalo antes de continuar."
        ));
    }

    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Error al ejecutar {cmd}: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "Comando falló ({cmd} {}): {}{}",
            args.join(" "),
            stderr,
            stdout
        ))
    }
}

pub fn install(entry: &PluginCatalogEntry, client_id: &str) -> Result<(), String> {
    let marketplace = &entry.marketplace;
    let plugin_ref = format!("{}@{}", marketplace.plugin_id, marketplace.marketplace_name);

    run_cli(
        client_id,
        &["plugin", "marketplace", "add", &marketplace.source],
    )?;

    run_cli(client_id, &["plugin", "install", &plugin_ref])
}

pub fn uninstall(entry: &PluginCatalogEntry, client_id: &str) -> Result<(), String> {
    let marketplace = &entry.marketplace;
    let plugin_ref = format!("{}@{}", marketplace.plugin_id, marketplace.marketplace_name);

    run_cli(client_id, &["plugin", "uninstall", &plugin_ref])
}
