use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub name: String,
    pub host: String,
    pub remote_path: String,
    pub local_path: String,
    pub mutagen_session: String,
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSpec {
    pub host: String,
    pub remote_path: String,
}

pub fn parse_remote_spec(input: &str) -> Result<RemoteSpec> {
    let (host, remote_path) = input
        .split_once(':')
        .ok_or_else(|| anyhow!("remote must be shaped like <host>:<remote_path>"))?;

    if host.is_empty() {
        return Err(anyhow!("remote host cannot be empty"));
    }
    if remote_path.is_empty() {
        return Err(anyhow!("remote path cannot be empty"));
    }

    Ok(RemoteSpec {
        host: host.to_owned(),
        remote_path: remote_path.to_owned(),
    })
}

pub fn default_ignore_patterns() -> Vec<String> {
    [
        ".git/",
        "node_modules/",
        "target/",
        ".direnv/",
        "dist/",
        "build/",
        ".next/",
        ".cache/",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub fn derive_mutagen_session(host: &str, name: &str) -> String {
    let raw = format!("rdev-{host}-{name}");
    let sanitized: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();

    sanitized
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_remote_spec() {
        let spec = parse_remote_spec("desktop:/home/nick/src/foo").unwrap();
        assert_eq!(spec.host, "desktop");
        assert_eq!(spec.remote_path, "/home/nick/src/foo");
    }

    #[test]
    fn rejects_invalid_remote_spec() {
        assert!(parse_remote_spec("desktop").is_err());
        assert!(parse_remote_spec(":/tmp/foo").is_err());
        assert!(parse_remote_spec("desktop:").is_err());
    }

    #[test]
    fn derives_mutagen_session_name() {
        assert_eq!(derive_mutagen_session("desktop", "foo"), "rdev-desktop-foo");
        assert_eq!(
            derive_mutagen_session("desk.example", "my repo"),
            "rdev-desk-example-my-repo"
        );
    }

    #[test]
    fn default_ignores_include_git() {
        assert!(default_ignore_patterns().contains(&".git/".to_owned()));
    }
}
