use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rdev")]
#[command(about = "Small remote-development helper around Mutagen and SSH")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init {
        name: String,
        remote: String,
    },
    Edit {
        name: String,
        #[arg(last = true)]
        command: Vec<String>,
    },
    Shell {
        name: String,
    },
    Run {
        name: String,
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    Status {
        name: String,
    },
    Flush {
        name: String,
    },
    Pause {
        name: String,
    },
    Resume {
        name: String,
    },
    Stop {
        name: String,
    },
    ResetFromRemote {
        name: String,
        #[arg(long)]
        yes: bool,
    },
    List,
    Remove {
        name: String,
    },
    Doctor,
}
