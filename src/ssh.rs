use std::process::Command;

use anyhow::{Context, Result, bail};
use shell_words::quote;

use crate::project::{ProjectConfig, ignore_patterns_from_gitignore};

pub fn shell_script(remote_path: &str) -> String {
    format!(
        "cd {} && exec \"${{SHELL:-/bin/sh}}\" -l",
        quote(remote_path).into_owned()
    )
}

pub fn run_script(remote_path: &str, command: &[String]) -> String {
    let quoted_command = command
        .iter()
        .map(|arg| quote(arg).into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "cd {} && exec {}",
        quote(remote_path).into_owned(),
        quoted_command
    )
}

pub fn gitignore_script(remote_path: &str) -> String {
    format!(
        "cd {} && if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then root=$(git rev-parse --show-toplevel) && if test -f \"$root/.gitignore\"; then cat \"$root/.gitignore\"; fi; else exit 10; fi",
        quote(remote_path).into_owned()
    )
}

pub fn remote_gitignore_patterns(host: &str, remote_path: &str) -> Result<Option<Vec<String>>> {
    let output = Command::new("ssh")
        .arg(host)
        .arg(gitignore_script(remote_path))
        .output()
        .context("failed to query remote .gitignore")?;

    if output.status.success() {
        let contents = String::from_utf8_lossy(&output.stdout);
        return Ok(Some(ignore_patterns_from_gitignore(&contents)));
    }

    if output.status.code() == Some(10) {
        return Ok(None);
    }

    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    bail!("failed to query remote .gitignore: {}", text.trim());
}

pub fn interactive_shell(project: &ProjectConfig) -> Result<()> {
    let status = Command::new("ssh")
        .arg("-t")
        .arg(&project.host)
        .arg(shell_script(&project.remote_path))
        .status()
        .context("failed to run ssh")?;

    if !status.success() {
        bail!("ssh shell failed with status {status}");
    }

    Ok(())
}

pub fn run(project: &ProjectConfig, command: &[String]) -> Result<()> {
    let status = Command::new("ssh")
        .arg(&project.host)
        .arg(run_script(&project.remote_path, command))
        .status()
        .context("failed to run ssh")?;

    if !status.success() {
        bail!("remote command failed with status {status}");
    }

    Ok(())
}

pub fn git_status(project: &ProjectConfig) -> Result<String> {
    let output = Command::new("ssh")
        .arg(&project.host)
        .arg(run_script(
            &project.remote_path,
            &["git".into(), "status".into(), "--short".into()],
        ))
        .output()
        .context("failed to run remote git status")?;

    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        bail!("remote git status failed: {}", text.trim());
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_remote_shell_script() {
        let script = shell_script("/home/nick/src/my repo");
        assert_eq!(
            script,
            "cd '/home/nick/src/my repo' && exec \"${SHELL:-/bin/sh}\" -l"
        );
    }

    #[test]
    fn quotes_remote_command_arguments() {
        let script = run_script(
            "/home/nick/src/my repo",
            &["cargo".into(), "test".into(), "name with spaces".into()],
        );
        assert_eq!(
            script,
            "cd '/home/nick/src/my repo' && exec cargo test 'name with spaces'"
        );
    }

    #[test]
    fn quotes_gitignore_script() {
        let script = gitignore_script("/home/nick/src/my repo");
        assert!(script.starts_with("cd '/home/nick/src/my repo' && if git rev-parse"));
        assert!(script.contains("cat \"$root/.gitignore\""));
        assert!(script.ends_with("else exit 10; fi"));
    }
}
