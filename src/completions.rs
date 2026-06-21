use std::io::{self, Write};

pub fn generate() -> io::Result<()> {
    io::stdout().write_all(ZSH.as_bytes())
}

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
