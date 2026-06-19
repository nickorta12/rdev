use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::project::ProjectConfig;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub projects: Vec<ProjectConfig>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let contents = toml::to_string_pretty(self).context("failed to serialize config")?;
        fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
    }

    pub fn upsert_project(&mut self, project: ProjectConfig) {
        if let Some(existing) = self.projects.iter_mut().find(|p| p.name == project.name) {
            *existing = project;
        } else {
            self.projects.push(project);
        }
    }

    pub fn project(&self, name: &str) -> Result<ProjectConfig> {
        self.projects
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .ok_or_else(|| anyhow!("project '{name}' is not configured"))
    }

    pub fn remove_project(&mut self, name: &str) -> bool {
        let before = self.projects.len();
        self.projects.retain(|p| p.name != name);
        self.projects.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::default_ignore_patterns;

    #[test]
    fn config_round_trips_as_toml() {
        let config = Config {
            projects: vec![ProjectConfig {
                name: "foo".into(),
                host: "desktop".into(),
                remote_path: "/home/nick/src/foo".into(),
                local_path: "/home/nick/.cache/rdev/desktop/foo".into(),
                mutagen_session: "rdev-desktop-foo".into(),
                ignore: default_ignore_patterns(),
            }],
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("[[projects]]"));
        assert!(toml.contains("\".git/\""));

        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(parsed, config);
    }
}
