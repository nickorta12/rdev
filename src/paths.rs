use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn config_path() -> Result<PathBuf> {
    config_path_with(dirs::config_dir().context("could not determine config directory")?)
}

pub fn cache_root() -> Result<PathBuf> {
    cache_root_with(dirs::cache_dir().context("could not determine cache directory")?)
}

pub fn config_path_with(config_home: PathBuf) -> Result<PathBuf> {
    Ok(config_home.join("rdev").join("config.toml"))
}

pub fn cache_root_with(cache_home: PathBuf) -> Result<PathBuf> {
    Ok(cache_home.join("rdev"))
}

pub fn project_cache_path(cache_root: PathBuf, host: &str, name: &str) -> PathBuf {
    cache_root.join(host).join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_default_config_path_from_home() {
        let path = config_path_with(PathBuf::from("/home/nick/.config")).unwrap();
        assert_eq!(path, PathBuf::from("/home/nick/.config/rdev/config.toml"));
    }

    #[test]
    fn computes_default_cache_path_from_home() {
        let root = cache_root_with(PathBuf::from("/home/nick/.cache")).unwrap();
        let path = project_cache_path(root, "desktop", "foo");
        assert_eq!(path, PathBuf::from("/home/nick/.cache/rdev/desktop/foo"));
    }
}
