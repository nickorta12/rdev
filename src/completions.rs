use std::io::{self, Write};

use clap::CommandFactory;
use clap_complete::Shell;

use crate::cli::Cli;

pub fn generate(shell: Shell) -> io::Result<()> {
    let script = match shell {
        Shell::Bash => Some(BASH),
        Shell::Fish => Some(FISH),
        Shell::Zsh => Some(ZSH),
        _ => None,
    };

    if let Some(script) = script {
        io::stdout().write_all(script.as_bytes())
    } else {
        let mut command = Cli::command();
        clap_complete::generate(shell, &mut command, "rdev", &mut io::stdout());
        Ok(())
    }
}

const BASH: &str = r#"_rdev()
{
    local cur prev words cword
    _init_completion || return

    local commands="init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help"

    if [[ $cword -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
        return
    fi

    local cmd="${words[1]}"
    case "$cmd" in
        init)
            return
            ;;
        completions)
            COMPREPLY=( $(compgen -W "bash fish zsh" -- "$cur") )
            return
            ;;
        reset-from-remote)
            if [[ $cur == -* ]]; then
                COMPREPLY=( $(compgen -W "--yes --help" -- "$cur") )
            elif [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$(rdev __project-names 2>/dev/null)" -- "$cur") )
            fi
            return
            ;;
        run)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$(rdev __project-names 2>/dev/null)" -- "$cur") )
            elif [[ $cword -eq 3 ]]; then
                COMPREPLY=( $(compgen -c -- "$cur") )
            else
                COMPREPLY=( $(compgen -f -- "$cur") )
            fi
            return
            ;;
        edit|shell|status|flush|pause|resume|stop|remove)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$(rdev __project-names 2>/dev/null)" -- "$cur") )
            fi
            return
            ;;
        help)
            COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
            return
            ;;
        *)
            return
            ;;
    esac
}

complete -F _rdev rdev
"#;

const FISH: &str = r#"function __fish_rdev_project_names
    rdev __project-names 2>/dev/null
end

function __fish_rdev_run_project_position
    set -l tokens (commandline -opc)
    test (count $tokens) -eq 2
end

function __fish_rdev_run_command_position
    set -l tokens (commandline -opc)
    test (count $tokens) -eq 3
end

complete -c rdev -f
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a init -d 'Configure a remote project'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a edit -d 'Open the local cache'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a shell -d 'Open a remote shell'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a run -d 'Run a remote command'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a status -d 'Show project, sync, and remote Git status'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a flush -d 'Flush pending Mutagen sync changes'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a pause -d 'Pause the Mutagen sync session'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a resume -d 'Resume or create the Mutagen sync session'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a stop -d 'Terminate the Mutagen sync session'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a reset-from-remote -d 'Discard and rebuild the local cache from remote'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a list -d 'List configured projects'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a remove -d 'Remove a project from config'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a doctor -d 'Check local rdev prerequisites'
complete -c rdev -n "not __fish_seen_subcommand_from init edit shell run status flush pause resume stop reset-from-remote list remove doctor completions help" -a completions -d 'Generate shell completions'

complete -c rdev -n "__fish_seen_subcommand_from edit shell status flush pause resume stop reset-from-remote remove" -a "(__fish_rdev_project_names)"
complete -c rdev -n "__fish_seen_subcommand_from run; and __fish_rdev_run_project_position" -a "(__fish_rdev_project_names)"
complete -c rdev -n "__fish_seen_subcommand_from run; and __fish_rdev_run_command_position" -a "(__fish_complete_command)"
complete -c rdev -n "__fish_seen_subcommand_from reset-from-remote" -l yes -d 'Skip the interactive confirmation prompt'
complete -c rdev -n "__fish_seen_subcommand_from completions" -a "bash fish zsh"
"#;

const ZSH: &str = r#"#compdef rdev

_rdev_project_names() {
  local -a projects
  projects=(${(f)"$(rdev __project-names 2>/dev/null)"})
  compadd -a projects
}

_rdev_impl() {
  local -a rdev_commands
  local -a tokens
  rdev_commands=(
    'init:Configure a remote project'
    'edit:Open the local cache'
    'shell:Open a remote shell'
    'run:Run a remote command'
    'status:Show project, sync, and remote Git status'
    'flush:Flush pending Mutagen sync changes'
    'pause:Pause the Mutagen sync session'
    'resume:Resume or create the Mutagen sync session'
    'stop:Terminate the Mutagen sync session'
    'reset-from-remote:Discard and rebuild the local cache from remote'
    'list:List configured projects'
    'remove:Remove a project from config'
    'doctor:Check local rdev prerequisites'
    'completions:Generate shell completions'
    'help:Print help'
  )

  if (( CURRENT == 2 )); then
    _describe 'commands' rdev_commands
    return
  fi

  case $words[2] in
    init)
      _files
      ;;
    edit|shell|status|flush|pause|resume|stop|remove)
      if (( CURRENT == 3 )); then
        _rdev_project_names
      fi
      ;;
    run)
      tokens=(${(z)LBUFFER})
      if (( ${#tokens} == 2 )); then
        _rdev_project_names
      elif (( ${#tokens} == 3 )) && [[ $LBUFFER == *' ' ]]; then
        _path_commands
      elif (( ${#tokens} == 4 )) && [[ $tokens[4] != -* && $LBUFFER != *' ' ]]; then
        _path_commands
      elif (( ${#tokens} > 4 )) && [[ $words[CURRENT] != -* ]]; then
        _files
      else
        return 0
      fi
      ;;
    reset-from-remote)
      if [[ $CURRENT == 3 && $words[CURRENT] == -* ]]; then
        compadd -- --yes --help
      elif (( CURRENT == 3 )); then
        _rdev_project_names
      else
        compadd -- --yes
      fi
      ;;
    completions)
      compadd -- bash fish zsh
      ;;
    help)
      _describe 'commands' rdev_commands
      ;;
  esac
}

if [[ -n ${ZSH_EVAL_CONTEXT[(r)file]} ]]; then
  compdef _rdev_impl rdev
else
  _rdev_impl "$@"
fi
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_completion_scripts_call_project_names_command() {
        assert!(BASH.contains("rdev __project-names"));
        assert!(FISH.contains("rdev __project-names"));
        assert!(ZSH.contains("rdev __project-names"));
    }
}
