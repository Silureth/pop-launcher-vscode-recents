# pop-launcher-vscode-open-recent

A [pop-launcher](https://github.com/pop-os/launcher) plugin that lets you quickly open recent VS Code workspaces and folders directly from the launcher.

Trigger it by typing `vc` followed by your search query. The trigger prefix can be changed in `plugin.ron` by editing the `regex` and `help` fields under `query`.

## Requirements

- [pop-launcher](https://github.com/pop-os/launcher)
- VS Code installed with the `code` CLI available in `PATH`
- Node.js 18 or newer

## Setup

The prebuilt plugin (`dist/index.cjs`) is committed and has all its dependencies bundled in, so no `npm install` or build step is needed.

1. Clone this repository into your pop-launcher plugins folder:

   ```sh
   git clone https://github.com/Silureth/pop-launcher-vscode-recents.git \
     ~/.local/share/pop-launcher/plugins/vscode-open-recent
   cd ~/.local/share/pop-launcher/plugins/vscode-open-recent
   ```

2. Create `plugin.ron` from the example, filling in your username in the `icon` path:

   ```sh
   sed "s/<fill-you-username>/$USER/" plugin.example.ron > plugin.ron
   ```

3. Restart pop-launcher so it picks up the plugin:

   ```sh
   pkill pop-launcher
   ```

   It starts again the next time you open the launcher.

To update later, run `git pull` in the plugin folder.

## Configuration

Edit `config.json` to adjust behavior:

| Option | Default | Description |
|---|---|---|
| `always_open_in_new_window` | `true` | Open entries in a new VS Code window |
| `hide_deleted_entries` | `true` | Skip entries whose paths no longer exist |
| `vscode-cli-additional-args` | `[]` | Extra arguments passed to the `code` CLI |

## Development

Install the build tooling and rebuild `dist/index.cjs` after changing anything in `src/`:

```sh
npm install
npm run build
```

Commit the rebuilt `dist/index.cjs` along with your source changes, since that is what users run.

Watch for source changes and rebuild automatically:

```sh
npm run dev
```

Preview the plugin output directly in the terminal (useful for debugging):

```sh
npm run preview
```

## Changelog

- **2026-11-07** **build:** bundle dependencies into `dist/index.cjs` so no `npm install` is needed; upgrade dependencies
- **2026-11-06** **fix:** `vc` trigger, icon paths, paths with spaces, remote folders; `config.json` options now take effect
- **2026-05-18** **fix:** read from `workspaceStorage` for VS Code 1.120+ compatibility

## Credits

I don't remember where I originally found this — but thank you to whoever created it.
