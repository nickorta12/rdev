use std::process::Command;

use anyhow::{Context, Result, bail};
use shell_words::quote;

use crate::project::ProjectConfig;

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
}
