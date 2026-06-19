use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::project::ProjectConfig;

pub fn create_args(project: &ProjectConfig) -> Vec<String> {
    let mut args = vec![
        "sync".to_owned(),
        "create".to_owned(),
        "--name".to_owned(),
        project.mutagen_session.clone(),
        "--sync-mode=two-way-safe".to_owned(),
        "--ignore-vcs".to_owned(),
    ];

    args.extend(
        project
            .ignore
            .iter()
            .filter(|pattern| pattern.trim_end_matches('/') != ".git")
            .map(|pattern| format!("--ignore={}", pattern.trim_end_matches('/'))),
    );

    args.push(project.local_path.clone());
    args.push(format!("{}:{}", project.host, project.remote_path));
    args
}

pub fn sync_exists(session: &str) -> Result<bool> {
    let status = Command::new("mutagen")
        .args(["sync", "list", session])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to query mutagen sync sessions")?;

    Ok(status.success())
}

pub fn start_or_resume(project: &ProjectConfig) -> Result<()> {
    if sync_exists(&project.mutagen_session)? {
        resume(project)
    } else {
        run_mutagen(create_args(project))
    }
}

pub fn flush(project: &ProjectConfig) -> Result<()> {
    run_mutagen(vec![
        "sync".into(),
        "flush".into(),
        project.mutagen_session.clone(),
    ])
}

pub fn pause(project: &ProjectConfig) -> Result<()> {
    run_mutagen(vec![
        "sync".into(),
        "pause".into(),
        project.mutagen_session.clone(),
    ])
}

pub fn resume(project: &ProjectConfig) -> Result<()> {
    run_mutagen(vec![
        "sync".into(),
        "resume".into(),
        project.mutagen_session.clone(),
    ])
}

pub fn terminate(project: &ProjectConfig) -> Result<()> {
    run_mutagen(vec![
        "sync".into(),
        "terminate".into(),
        project.mutagen_session.clone(),
    ])
}

pub fn status(project: &ProjectConfig) -> Result<String> {
    let output = Command::new("mutagen")
        .args(["sync", "list", &project.mutagen_session])
        .output()
        .context("failed to run mutagen sync list")?;

    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        bail!("mutagen status failed: {}", text.trim());
    }

    Ok(text)
}

fn run_mutagen(args: Vec<String>) -> Result<()> {
    let status = Command::new("mutagen")
        .args(args)
        .status()
        .context("failed to run mutagen")?;

    if !status.success() {
        bail!("mutagen failed with status {status}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{ProjectConfig, default_ignore_patterns};

    fn project() -> ProjectConfig {
        ProjectConfig {
            name: "foo".into(),
            host: "desktop".into(),
            remote_path: "/home/nick/src/foo".into(),
            local_path: "/home/nick/.cache/rdev/desktop/foo".into(),
            mutagen_session: "rdev-desktop-foo".into(),
            ignore: default_ignore_patterns(),
        }
    }

    #[test]
    fn builds_mutagen_create_arguments() {
        let args = create_args(&project());
        assert_eq!(args[0], "sync");
        assert_eq!(args[1], "create");
        assert!(args.contains(&"--name".to_owned()));
        assert!(args.contains(&"rdev-desktop-foo".to_owned()));
        assert!(args.contains(&"--sync-mode=two-way-safe".to_owned()));
        assert!(args.contains(&"--ignore-vcs".to_owned()));
        assert!(args.contains(&"--ignore=node_modules".to_owned()));
        assert!(!args.contains(&"--ignore=.git".to_owned()));
    }
}
