mod bincheck;
mod cli;
mod completions;
mod config;
mod mutagen;
mod paths;
mod project;
mod rsync;
mod ssh;

use std::fs;
use std::io::{self, Write};
use std::process::Command as ProcessCommand;

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::config::Config;
use crate::project::{ProjectConfig, default_ignore_patterns, derive_mutagen_session};

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { name, remote } => init(&name, &remote),
        Command::Edit { name } => edit(&name),
        Command::Shell { name } => remote_shell(&name),
        Command::Run { name, command } => remote_run(&name, &command),
        Command::Status { name } => status(&name),
        Command::Flush { name } => with_project(&name, |project| mutagen::flush(&project)),
        Command::Pause { name } => with_project(&name, |project| mutagen::pause(&project)),
        Command::Resume { name } => {
            with_project(&name, |project| mutagen::start_or_resume(&project))
        }
        Command::Stop { name } => with_project(&name, |project| mutagen::terminate(&project)),
        Command::ResetFromRemote { name, yes } => reset_from_remote(&name, yes),
        Command::List => list(),
        Command::Remove { name } => remove(&name),
        Command::Doctor => doctor(),
        Command::Completions { shell } => completions(shell),
        Command::ProjectNames => project_names(),
    }
}

fn completions(shell: clap_complete::Shell) -> Result<()> {
    completions::generate(shell).context("failed to generate completions")?;
    Ok(())
}

fn init(name: &str, remote: &str) -> Result<()> {
    bincheck::require_binaries(&["ssh"])?;
    let remote = project::parse_remote_spec(remote)?;
    let config_path = paths::config_path()?;
    let cache_root = paths::cache_root()?;
    let local_path = paths::project_cache_path(cache_root, &remote.host, name);

    fs::create_dir_all(&local_path)
        .with_context(|| format!("failed to create {}", local_path.display()))?;

    let ignore = ssh::remote_gitignore_patterns(&remote.host, &remote.remote_path)?
        .unwrap_or_else(default_ignore_patterns);

    let project = ProjectConfig {
        name: name.to_owned(),
        host: remote.host.clone(),
        remote_path: remote.remote_path,
        local_path: local_path.to_string_lossy().into_owned(),
        mutagen_session: derive_mutagen_session(&remote.host, name),
        ignore,
    };

    let mut config = Config::load(&config_path)?;
    config.upsert_project(project);
    config.save(&config_path)?;

    println!("configured project '{name}' in {}", config_path.display());
    Ok(())
}

fn edit(name: &str) -> Result<()> {
    bincheck::require_binaries(&["rsync", "mutagen"])?;
    let project = load_project(name)?;
    ensure_local_cache(&project)?;
    bootstrap_if_empty(&project)?;
    mutagen::start_or_resume(&project)?;
    local_shell(&project)
}

fn remote_shell(name: &str) -> Result<()> {
    bincheck::require_binaries(&["ssh", "mutagen"])?;
    let project = load_project(name)?;
    mutagen::start_or_resume(&project)?;
    mutagen::flush(&project)?;
    ssh::interactive_shell(&project)
}

fn remote_run(name: &str, command: &[String]) -> Result<()> {
    bincheck::require_binaries(&["ssh", "mutagen"])?;
    let project = load_project(name)?;
    mutagen::start_or_resume(&project)?;
    mutagen::flush(&project)?;
    ssh::run(&project, command)
}

fn status(name: &str) -> Result<()> {
    bincheck::require_binaries(&["ssh", "mutagen"])?;
    let project = load_project(name)?;

    println!("project: {}", project.name);
    println!("host: {}", project.host);
    println!("remote path: {}", project.remote_path);
    println!("local path: {}", project.local_path);
    println!("mutagen session: {}", project.mutagen_session);
    println!();
    println!("mutagen status:");
    if mutagen::sync_exists(&project.mutagen_session)? {
        match mutagen::status(&project) {
            Ok(text) if text.trim().is_empty() => println!("  running, no status output"),
            Ok(text) => print_indented(&text),
            Err(err) => println!("  unavailable: {err:#}"),
        }
    } else {
        println!("  not running");
        println!("  start it with: rdev resume {}", project.name);
    }
    println!();
    println!("remote git status --short:");
    match ssh::git_status(&project) {
        Ok(text) if text.trim().is_empty() => println!("  clean"),
        Ok(text) => print_indented(&text),
        Err(err) => println!("  unavailable: {err:#}"),
    }

    Ok(())
}

fn reset_from_remote(name: &str, yes: bool) -> Result<()> {
    bincheck::require_binaries(&["rsync", "mutagen"])?;
    let project = load_project(name)?;

    if !yes && !confirm_reset(&project)? {
        bail!("reset cancelled");
    }

    if mutagen::sync_exists(&project.mutagen_session)? {
        mutagen::terminate(&project)?;
    }

    clear_directory_contents(&project.local_path)?;
    fs::create_dir_all(&project.local_path)
        .with_context(|| format!("failed to create {}", project.local_path))?;
    rsync::bootstrap(&project)?;
    mutagen::start_or_resume(&project)?;

    println!("reset '{}' from remote", project.name);
    Ok(())
}

fn list() -> Result<()> {
    let config = Config::load(&paths::config_path()?)?;
    for project in config.projects {
        println!(
            "{}\t{}:{}\t{}",
            project.name, project.host, project.remote_path, project.local_path
        );
    }
    Ok(())
}

fn project_names() -> Result<()> {
    let config = Config::load(&paths::config_path()?)?;
    for project in config.projects {
        println!("{}", project.name);
    }
    Ok(())
}

fn remove(name: &str) -> Result<()> {
    let config_path = paths::config_path()?;
    let mut config = Config::load(&config_path)?;
    if config.remove_project(name) {
        config.save(&config_path)?;
        println!("removed project '{name}' from config");
    } else {
        bail!("project '{name}' is not configured");
    }
    Ok(())
}

fn doctor() -> Result<()> {
    bincheck::require_binaries(&["ssh", "rsync", "mutagen"])?;
    println!("all required external binaries found");
    println!("config: {}", paths::config_path()?.display());
    println!("cache root: {}", paths::cache_root()?.display());
    Ok(())
}

fn with_project<F>(name: &str, f: F) -> Result<()>
where
    F: FnOnce(ProjectConfig) -> Result<()>,
{
    bincheck::require_binaries(&["mutagen"])?;
    f(load_project(name)?)
}

fn load_project(name: &str) -> Result<ProjectConfig> {
    Config::load(&paths::config_path()?)?.project(name)
}

fn ensure_local_cache(project: &ProjectConfig) -> Result<()> {
    fs::create_dir_all(&project.local_path)
        .with_context(|| format!("failed to create {}", project.local_path))
}

fn bootstrap_if_empty(project: &ProjectConfig) -> Result<()> {
    if fs::read_dir(&project.local_path)
        .with_context(|| format!("failed to read {}", project.local_path))?
        .next()
        .is_none()
    {
        rsync::bootstrap(project)?;
    }
    Ok(())
}

fn local_shell(project: &ProjectConfig) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let status = ProcessCommand::new(shell)
        .current_dir(&project.local_path)
        .status()
        .context("failed to start local shell")?;

    if !status.success() {
        bail!("local shell exited with status {status}");
    }

    Ok(())
}

fn confirm_reset(project: &ProjectConfig) -> Result<bool> {
    eprintln!(
        "This will delete all contents of the local cache for '{}' at {}.",
        project.name, project.local_path
    );
    eprint!("Type 'reset {}' to continue: ", project.name);
    io::stderr().flush().ok();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read confirmation")?;
    Ok(input.trim() == format!("reset {}", project.name))
}

fn clear_directory_contents(path: &str) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create {path}"))?;
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {path}"))? {
        let entry = entry.with_context(|| format!("failed to read entry in {path}"))?;
        let child = entry.path();
        if child.is_dir() {
            fs::remove_dir_all(&child)
                .with_context(|| format!("failed to remove {}", child.display()))?;
        } else {
            fs::remove_file(&child)
                .with_context(|| format!("failed to remove {}", child.display()))?;
        }
    }
    Ok(())
}

fn print_indented(text: &str) {
    for line in text.lines() {
        println!("  {line}");
    }
}
