use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::project::ProjectConfig;

pub fn bootstrap_args(project: &ProjectConfig) -> Vec<String> {
    let mut args = vec!["-az".to_owned(), "--delete".to_owned()];
    args.extend(
        project
            .ignore
            .iter()
            .map(|pattern| format!("--exclude={pattern}")),
    );
    args.push(format!(
        "{}:{}/",
        project.host,
        trim_trailing_slash(&project.remote_path)
    ));
    args.push(format!("{}/", trim_trailing_slash(&project.local_path)));
    args
}

pub fn bootstrap(project: &ProjectConfig) -> Result<()> {
    let status = Command::new("rsync")
        .args(bootstrap_args(project))
        .status()
        .context("failed to run rsync")?;

    if !status.success() {
        bail!("rsync bootstrap failed with status {status}");
    }

    Ok(())
}

fn trim_trailing_slash(path: &str) -> &str {
    path.trim_end_matches('/')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{ProjectConfig, default_ignore_patterns};

    fn project() -> ProjectConfig {
        ProjectConfig {
            name: "foo".into(),
            host: "desktop".into(),
            remote_path: "/home/nick/src/foo/".into(),
            local_path: "/home/nick/.cache/rdev/desktop/foo".into(),
            mutagen_session: "rdev-desktop-foo".into(),
            ignore: default_ignore_patterns(),
        }
    }

    #[test]
    fn builds_rsync_bootstrap_arguments() {
        let args = bootstrap_args(&project());
        assert_eq!(args[0], "-az");
        assert_eq!(args[1], "--delete");
        assert!(args.contains(&"--exclude=.git/".to_owned()));
        assert!(args.contains(&"--exclude=node_modules/".to_owned()));
        assert!(args.contains(&"desktop:/home/nick/src/foo/".to_owned()));
        assert!(args.contains(&"/home/nick/.cache/rdev/desktop/foo/".to_owned()));
    }
}
