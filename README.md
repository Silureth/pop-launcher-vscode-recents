# pop-launcher-vscode-open-recent

A [pop-launcher](https://github.com/pop-os/launcher) plugin that lets you quickly open recent VS Code workspaces and folders directly from the launcher, including remote ones (WSL, SSH, containers).

Trigger it by typing `vc` followed by your search query. The trigger prefix can be changed in `plugin.ron` by editing the `regex` and `help` fields under `query`.

## Requirements

- [pop-launcher](https://github.com/pop-os/launcher)
- VS Code installed with the `code` CLI available in `PATH`

The plugin is a single static binary for x86_64 or ARM64 Linux, so nothing else (no Node.js, no Rust) needs to be installed.

## Install

1. Download the latest release into your pop-launcher plugins folder:

   ```sh
   mkdir -p ~/.local/share/pop-launcher/plugins
   curl -fsSL "https://github.com/Silureth/pop-launcher-vscode-recents/releases/latest/download/vscode-open-recent-$(uname -m).tar.gz" \
     | tar -xz -C ~/.local/share/pop-launcher/plugins
   cd ~/.local/share/pop-launcher/plugins/vscode-open-recent
   ```

2. Create `plugin.ron` from the example, filling in the plugin folder in the `icon` path:

   ```sh
   sed "s|<plugin-folder>|$PWD|" plugin.example.ron > plugin.ron
   ```

3. Restart pop-launcher so it picks up the plugin:

   ```sh
   pkill pop-launcher
   ```

   It starts again the next time you open the launcher.

**To update**, re-run the `curl` command from step 1, then `pkill pop-launcher`. Your `plugin.ron` and `config.json` are kept, since the release only contains the example files.

### Upgrading from the Node.js version

Earlier versions were a Node.js script installed with `git clone`. Move the old folder aside, follow the install steps above, then copy over your settings if you had changed them:

```sh
mv ~/.local/share/pop-launcher/plugins/vscode-open-recent ~/vscode-open-recent-old
# ...install steps 1-3...
cp ~/vscode-open-recent-old/config.json ~/.local/share/pop-launcher/plugins/vscode-open-recent/
```

## Configuration

The defaults below apply out of the box. To change them, copy `config.example.json` to `config.json` in the plugin folder and edit it:

| Option | Default | Description |
|---|---|---|
| `always_open_in_new_window` | `true` | Open entries in a new VS Code window |
| `hide_deleted_entries` | `true` | Skip local folders that no longer exist |
| `vscode-cli-additional-args` | `[]` | Extra arguments passed to the `code` CLI, e.g. `["--profile", "Work"]` |

## Development

Requires a [Rust toolchain](https://rustup.rs).

```sh
cargo test
cargo build --release
```

To run your local build in the launcher, clone the repository as the plugin folder (`~/.local/share/pop-launcher/plugins/vscode-open-recent`), create `plugin.ron` as in the install steps, then copy the binary next to it after each build:

```sh
cp target/release/vscode-open-recent .
pkill pop-launcher
```

The plugin speaks JSON over stdin/stdout, so you can also try it directly in the terminal:

```sh
printf '{"Search":"vc my project"}\n"Exit"\n' | cargo run
```

### Releasing

Bump `version` in `Cargo.toml`, commit, then push a matching tag:

```sh
git tag v1.0.0
git push origin v1.0.0
```

The release workflow builds static x86_64 and ARM64 binaries and attaches them to a GitHub release.

## Changelog

- **2026-11-08** **rewrite:** rewritten in Rust and distributed as a static binary through GitHub releases, so Node.js is no longer needed. `config.json` is now optional (see `config.example.json`)
- **2026-11-07** **build:** bundle dependencies into `dist/index.cjs` so no `npm install` is needed; upgrade dependencies
- **2026-11-06** **fix:** `vc` trigger, icon paths, paths with spaces, remote folders; `config.json` options now take effect
- **2026-05-18** **fix:** read from `workspaceStorage` for VS Code 1.120+ compatibility

## Credits

I don't remember where I originally found this — but thank you to whoever created it.
