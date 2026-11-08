mod config;
mod vscode;

use config::Config;
use pop_launcher::{IconSource, PluginResponse, PluginSearchResult, Request};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use vscode::{Entry, EntryType};

const MAX_RESULTS: usize = 8;

struct Plugin {
    entries: Vec<Entry>,
    /// Indices into `entries` for the rows currently shown.
    shown: Vec<usize>,
    config: Config,
    out: io::StdoutLock<'static>,
}

impl Plugin {
    fn send(&mut self, response: PluginResponse) {
        // If the launcher has gone away there is nobody to report to.
        if serde_json::to_writer(&mut self.out, &response).is_ok() {
            let _ = self.out.write_all(b"\n");
            let _ = self.out.flush();
        }
    }

    fn search(&mut self, query: &str) {
        let query = strip_trigger(query);
        self.shown = if query.is_empty() {
            (0..self.entries.len().min(MAX_RESULTS)).collect()
        } else {
            vscode::search(&self.entries, query, MAX_RESULTS)
        };

        self.send(PluginResponse::Clear);

        if self.shown.is_empty() {
            let (name, description) = if query.is_empty() {
                ("No recent VS Code workspaces.", "")
            } else {
                ("No results found.", "Try a different query.")
            };
            self.send(PluginResponse::Append(PluginSearchResult {
                id: 0,
                name: name.into(),
                description: description.into(),
                icon: Some(IconSource::Name("dialog-information".into())),
                ..Default::default()
            }));
        }

        for id in 0..self.shown.len() {
            let entry = &self.entries[self.shown[id]];
            let result = PluginSearchResult {
                id: id as u32,
                name: entry.name.clone(),
                description: entry.path.clone(),
                icon: Some(icon(entry)),
                ..Default::default()
            };
            self.send(PluginResponse::Append(result));
        }

        self.send(PluginResponse::Finished);
    }

    fn activate(&mut self, id: u32) {
        if let Some(&index) = self.shown.get(id as usize) {
            // Nothing useful to show the user if `code` fails to start.
            let _ = vscode::open(&self.entries[index], &self.config);
        }
        self.send(PluginResponse::Close);
    }
}

/// Icon from the user's desktop icon theme, so it matches the rest of the system.
fn icon(entry: &Entry) -> IconSource {
    match (entry.kind, &entry.local_path) {
        // No standard icon name exists for workspaces; the theme picks one for
        // the file type VS Code registers, or falls back to a generic document.
        (EntryType::Workspace, _) => IconSource::Mime("application/x-code-workspace".into()),
        (EntryType::Folder, Some(_)) => IconSource::Name("folder".into()),
        (EntryType::Folder, None) => IconSource::Name("folder-remote".into()),
    }
}

/// Drops the trigger word the launcher routed on (whatever `regex` in
/// plugin.ron matches, `vc` by default), leaving just the search text.
fn strip_trigger(query: &str) -> &str {
    match query.trim().split_once(char::is_whitespace) {
        Some((_, rest)) => rest.trim(),
        None => "",
    }
}

fn main() {
    // The binary sits in the plugin folder, next to plugin.ron and config.json.
    let plugin_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
        .unwrap_or_default();
    let home = std::env::var("HOME").unwrap_or_default();

    let config = Config::load(&plugin_dir.join("config.json"));
    let storage = PathBuf::from(&home).join(".config/Code/User/workspaceStorage");
    let entries = vscode::recently_opened(&storage, &home, &config);

    let mut plugin = Plugin {
        entries,
        shown: Vec::new(),
        config,
        out: io::stdout().lock(),
    };

    for line in io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        match serde_json::from_str::<Request>(&line) {
            Ok(Request::Search(query)) => plugin.search(&query),
            Ok(Request::Activate(id)) => plugin.activate(id),
            Ok(Request::Exit) => break,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_trigger_drops_first_word() {
        assert_eq!(strip_trigger("vc"), "");
        assert_eq!(strip_trigger("  vc  my app "), "my app");
        assert_eq!(strip_trigger("code api"), "api");
    }
}
