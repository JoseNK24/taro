use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::adapters::{backup_file, expand_home_path, rollback_from_backup};
use crate::models::PluginCatalogEntry;

fn sparse_clone_repo(repo_url: &str, paths: &[String], dest: &Path) -> Result<(), String> {
    if dest.exists() {
        fs::remove_dir_all(dest).map_err(|e| format!("No se pudo limpiar directorio temporal: {e}"))?;
    }

    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            "--sparse",
            repo_url,
        ])
        .arg(dest)
        .status()
        .map_err(|e| format!("Error al ejecutar git clone: {e}"))?;

    if !status.success() {
        return Err(format!("git clone falló para {repo_url}"));
    }

    let sparse_paths: Vec<&str> = paths.iter().map(String::as_str).collect();
    let output = Command::new("git")
        .args(["sparse-checkout", "set"])
        .args(&sparse_paths)
        .current_dir(dest)
        .output()
        .map_err(|e| format!("Error al configurar sparse-checkout: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("sparse-checkout falló: {stderr}"));
    }

    Ok(())
}

fn copy_file_with_backup(src: &Path, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("No se pudo crear directorio {}: {e}", parent.display()))?;
    }

    let backup = backup_file(dest).map_err(|e| e.to_string())?;
    if let Err(e) = fs::copy(src, dest) {
        let _ = rollback_from_backup(dest, &backup);
        return Err(format!("No se pudo copiar a {}: {e}", dest.display()));
    }
    Ok(())
}

pub fn install(entry: &PluginCatalogEntry, files: &[String]) -> Result<(), String> {
    let temp_dir = std::env::temp_dir().join(format!("taro-plugin-{}", entry.id));
    sparse_clone_repo(&entry.github_url, files, &temp_dir)?;

    for file in files {
        let src = temp_dir.join(file);
        let dest = expand_home_path(file);
        if !src.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(format!(
                "Archivo no encontrado en el repositorio: {file}"
            ));
        }
        copy_file_with_backup(&src, &dest)?;
    }

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}

pub fn uninstall(files: &[String]) -> Result<(), String> {
    for file in files {
        let dest = expand_home_path(file);
        if dest.exists() {
            fs::remove_file(&dest)
                .map_err(|e| format!("No se pudo eliminar {}: {e}", dest.display()))?;
        }
        let backup = backup_path_for(&dest);
        if backup.exists() {
            let _ = fs::remove_file(&backup);
        }
    }
    Ok(())
}

fn backup_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    path.with_file_name(format!("{file_name}.taro-backup"))
}
