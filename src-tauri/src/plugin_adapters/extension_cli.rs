use std::process::Command;

use crate::adapters::json_mcp::command_exists;

fn run_gemini(args: &[&str]) -> Result<(), String> {
    if !command_exists("gemini") {
        return Err(
            "No se encontró el comando 'gemini' en PATH. Instala Gemini CLI antes de continuar."
                .to_string(),
        );
    }

    let output = Command::new("gemini")
        .args(args)
        .output()
        .map_err(|e| format!("Error al ejecutar gemini: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "Comando gemini falló ({}): {}{}",
            args.join(" "),
            stderr,
            stdout
        ))
    }
}

pub fn install(package: &str) -> Result<(), String> {
    run_gemini(&["extensions", "install", package])
}

pub fn uninstall(package: &str) -> Result<(), String> {
    if !command_exists("gemini") {
        return Ok(());
    }
    run_gemini(&["extensions", "uninstall", package])
}
