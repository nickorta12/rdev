# rdev

`rdev` is a small remote-development helper around Mutagen and SSH.

It is for a workflow where a desktop machine owns the real repository, while a laptop works in a local cached copy. Editors and local tools such as `nvim`, LSP servers, `rg`, and `fd` run against the laptop cache. Build, test, and shell commands run on the desktop in the real repository.

Mutagen handles sync. SSH handles remote command execution. `rdev` does not implement its own sync engine.

## Why `.git` is not synced

The desktop repository is the source of truth. Syncing `.git/` between machines is risky because Git metadata changes frequently and can be corrupted or conflicted by bidirectional file sync. `rdev` always excludes `.git/` during bootstrap and uses Mutagen's `--ignore-vcs` option for sync sessions.

The default ignored paths are:

- `.git/`
- `node_modules/`
- `target/`
- `.direnv/`
- `dist/`
- `build/`
- `.next/`
- `.cache/`

## Install with Nix

Enter the development shell:

```sh
nix develop
```

Build and test:

```sh
cargo build
cargo test
```

Build the flake package:

```sh
nix build
```

The dev shell includes Rust, Cargo, rustfmt, clippy, Mutagen, rsync, and OpenSSH.

## Usage

Configure a project:

```sh
rdev init foo desktop:/home/nick/src/foo
```

This writes `~/.config/rdev/config.toml` or `$XDG_CONFIG_HOME/rdev/config.toml`, creates a cache directory under `~/.cache/rdev/desktop/foo` or `$XDG_CACHE_HOME/rdev/desktop/foo`, and does not start sync yet.

Open a local cache shell:

```sh
rdev edit foo
```

If the cache is empty, `rdev edit` bootstraps it from the desktop with `rsync`, excluding `.git/` and heavy generated directories. It then starts or resumes the Mutagen session and opens a local shell in the cache.

Run a local editor in the cache:

```sh
rdev edit foo -- nvim .
```

Run a remote command in the real desktop repo:

```sh
rdev run foo -- cargo test
```

Open a remote interactive shell:

```sh
rdev shell foo
```

Show status:

```sh
rdev status foo
```

Manage sync:

```sh
rdev flush foo
rdev pause foo
rdev resume foo
rdev stop foo
```

## Recovery

If the laptop cache should be discarded and rebuilt from the desktop:

```sh
rdev reset-from-remote foo
```

This is destructive for the local cache, so it asks for confirmation. To skip the prompt:

```sh
rdev reset-from-remote foo --yes
```

The command terminates the Mutagen session if it exists, deletes the local cache contents, bootstraps from the remote repository with `rsync`, and starts the Mutagen session again.

## Known limitations

- A working SSH configuration is required for the configured host.
- `ssh`, `rsync`, and `mutagen` must be installed and available in `PATH`.
- Local Git status is intentionally unavailable or misleading because `.git/` is not synced.
- Remote command execution uses a small quoted shell script to `cd` into the repository before running the requested command.
- The first version is explicit. It does not daemonize and does not transparently remote arbitrary commands.
