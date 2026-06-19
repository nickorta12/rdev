use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(name = "rdev")]
#[command(about = "Small remote-development helper around Mutagen and SSH")]
#[command(
    long_about = "rdev keeps a local laptop cache of a remote desktop repository. Editors and local tools run against the cache; build, test, and shell commands run remotely over SSH in the real repository. Mutagen handles sync and .git is never synced.",
    after_help = "Examples:
  rdev init foo desktop:/home/nick/src/foo
  rdev edit foo
  rdev edit foo -- nvim .
  rdev run foo -- cargo test
  rdev shell foo
  rdev status foo
  rdev completions zsh > _rdev"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Configure a remote project",
        long_about = "Create or update a project entry in rdev's TOML config. The local cache directory is created, but sync is not started yet. For Git repositories, ignore patterns are read from the remote root .gitignore and .git/ is always added.",
        after_help = "Examples:
  rdev init foo desktop:/home/nick/src/foo
  rdev init work macstudio:/Users/nick/src/work"
    )]
    Init {
        #[arg(help = "Project name used by later rdev commands")]
        name: String,
        #[arg(help = "Remote repository as <host>:<remote_path>")]
        remote: String,
    },
    #[command(
        about = "Open or use the local cache",
        long_about = "Ensure the local cache exists, bootstrap it from the remote repo if empty, start or resume Mutagen sync, then open a local shell in the cache. With a command after --, run that local command in the cache instead.",
        after_help = "Examples:
  rdev edit foo
  rdev edit foo -- nvim .
  rdev edit foo -- rg TODO"
    )]
    Edit {
        #[arg(help = "Configured project name")]
        name: String,
        #[arg(help = "Optional local command to run inside the cache", last = true)]
        command: Vec<String>,
    },
    #[command(
        about = "Open a remote shell",
        long_about = "Flush Mutagen first, then open an interactive SSH shell on the desktop in the remote project directory.",
        after_help = "Examples:
  rdev shell foo"
    )]
    Shell {
        #[arg(help = "Configured project name")]
        name: String,
    },
    #[command(
        about = "Run a remote command",
        long_about = "Flush Mutagen first, then run the command on the desktop via SSH in the real repository directory. Put the command after -- so its arguments are passed through safely.",
        after_help = "Examples:
  rdev run foo -- cargo test
  rdev run foo -- nix flake check
  rdev run foo -- just build"
    )]
    Run {
        #[arg(help = "Configured project name")]
        name: String,
        #[arg(help = "Remote command and arguments", last = true, required = true)]
        command: Vec<String>,
    },
    #[command(
        about = "Show project, sync, and remote Git status",
        after_help = "Examples:
  rdev status foo"
    )]
    Status {
        #[arg(help = "Configured project name")]
        name: String,
    },
    #[command(
        about = "Flush pending Mutagen sync changes",
        after_help = "Examples:
  rdev flush foo"
    )]
    Flush {
        #[arg(help = "Configured project name")]
        name: String,
    },
    #[command(
        about = "Pause the Mutagen sync session",
        after_help = "Examples:
  rdev pause foo"
    )]
    Pause {
        #[arg(help = "Configured project name")]
        name: String,
    },
    #[command(
        about = "Resume the Mutagen sync session",
        after_help = "Examples:
  rdev resume foo"
    )]
    Resume {
        #[arg(help = "Configured project name")]
        name: String,
    },
    #[command(
        about = "Terminate the Mutagen sync session",
        long_about = "Terminate the Mutagen sync session for a project. The local cache directory is left in place.",
        after_help = "Examples:
  rdev stop foo"
    )]
    Stop {
        #[arg(help = "Configured project name")]
        name: String,
    },
    #[command(
        about = "Discard and rebuild the local cache from remote",
        long_about = "Dangerous operation. Terminates the Mutagen session if it exists, deletes the local cache contents, bootstraps from the remote repository with rsync, and starts the sync session again.",
        after_help = "Examples:
  rdev reset-from-remote foo
  rdev reset-from-remote foo --yes"
    )]
    ResetFromRemote {
        #[arg(help = "Configured project name")]
        name: String,
        #[arg(long, help = "Skip the interactive confirmation prompt")]
        yes: bool,
    },
    #[command(
        about = "List configured projects",
        after_help = "Examples:
  rdev list"
    )]
    List,
    #[command(
        about = "Remove a project from config",
        long_about = "Remove a project entry from rdev's config. This does not terminate Mutagen and does not delete the local cache.",
        after_help = "Examples:
  rdev remove foo"
    )]
    Remove {
        #[arg(help = "Configured project name")]
        name: String,
    },
    #[command(
        about = "Check local rdev prerequisites",
        after_help = "Examples:
  rdev doctor"
    )]
    Doctor,
    #[command(
        about = "Generate shell completions",
        long_about = "Generate command-line completions for the requested shell. Nix installs bash, fish, and zsh completions automatically for the packaged binary.",
        after_help = "Examples:
  rdev completions bash
  rdev completions fish
  rdev completions zsh > _rdev"
    )]
    Completions {
        #[arg(help = "Shell to generate completions for")]
        shell: Shell,
    },
}
