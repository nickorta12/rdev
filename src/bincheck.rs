use std::env;
use std::path::Path;

use anyhow::{Result, anyhow};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn require_binaries(names: &[&str]) -> Result<()> {
    let missing = names
        .iter()
        .filter(|name| !binary_exists(name))
        .copied()
        .collect::<Vec<_>>();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "missing required external binaries: {}",
            missing.join(", ")
        ))
    }
}

fn binary_exists(name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path).any(|dir| is_executable(&dir.join(name)))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    path.is_file()
        && path
            .metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
