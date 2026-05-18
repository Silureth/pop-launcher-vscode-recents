# pop-launcher-vscode-open-recent

A [pop-launcher](https://github.com/pop-os/launcher) plugin that lets you quickly open recent VS Code workspaces and folders directly from the launcher.

Trigger it by typing `vc` followed by your search query. The trigger prefix can be changed in `plugin.ron` by editing the `regex` and `help` fields under `query`.

## Requirements

- [pop-launcher](https://github.com/pop-os/launcher)
- VS Code installed with the `code` CLI available in `PATH`
- Node.js

## Setup

1. Clone or copy this directory into your pop-launcher plugins folder:

   ```
   ~/.local/share/pop-launcher/plugins/vscode-open-recent/
   ```

2. Copy and rename the example plugin config, then fill in your username:

   ```sh
   cp plugin.example.ron plugin.ron
   ```

   Open `plugin.ron` and replace `<fill-you-username>` in the `icon` path with your actual username.

3. Install dependencies:

   ```sh
   npm install
   ```

3. Build the plugin:

   ```sh
   npm run build
   ```

   The compiled output is written to `dist/index.cjs`.

## Configuration

Edit `config.json` to adjust behavior:

| Option | Default | Description |
|---|---|---|
| `always_open_in_new_window` | `true` | Open entries in a new VS Code window |
| `hide_deleted_entries` | `true` | Skip entries whose paths no longer exist |
| `vscode-cli-additional-args` | `[]` | Extra arguments passed to the `code` CLI |

## Development

Watch for source changes and rebuild automatically:

```sh
npm run dev
```

Preview the plugin output directly in the terminal (useful for debugging):

```sh
npm run preview
```

## Changelog

- **2026-05-18** **fix:** read from `workspaceStorage` for VS Code 1.120+ compatibility

## Credits

I don't remember where I originally found this — but thank you to whoever created it.
