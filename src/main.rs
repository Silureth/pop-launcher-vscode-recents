mod config;
mod vscode;

use config::Config;
use pop_launcher::{IconSource, PluginResponse, PluginSearchResult, Request};
use std::borrow::Cow;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use vscode::{Entry, EntryType};

/// Must match the `regex` in plugin.ron.
const TRIGGER: &str = "vc";
const MAX_RESULTS: usize = 8;

struct Plugin {
    entries: Vec<Entry>,
    /// Indices into `entries` for the rows currently shown.
    shown: Vec<usize>,
    config: Config,
    assets: PathBuf,
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

    fn icon(&self, name: &str) -> Option<IconSource> {
        let path = self.assets.join(format!("{name}.svg"));
        Some(IconSource::Name(Cow::Owned(
            path.to_string_lossy().into_owned(),
        )))
    }

    fn search(&mut self, query: &str) {
        let query = clean_query(query, TRIGGER);
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
            let icon = self.icon("info");
            self.send(PluginResponse::Append(PluginSearchResult {
                id: 0,
                name: name.into(),
                description: description.into(),
                icon,
                ..Default::default()
            }));
        }

        for id in 0..self.shown.len() {
            let entry = &self.entries[self.shown[id]];
            let result = PluginSearchResult {
                id: id as u32,
                name: entry.name.clone(),
                description: entry.path.clone(),
                icon: self.icon(match entry.kind {
                    EntryType::Folder => "folder",
                    EntryType::Workspace => "workspace",
                }),
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

/// Strips the leading trigger word (e.g. "vc") and surrounding whitespace.
fn clean_query<'a>(query: &'a str, trigger: &str) -> &'a str {
    let query = query.trim();
    query.strip_prefix(trigger).unwrap_or(query).trim()
}

fn main() {
    // The binary sits in the plugin folder, next to assets/ and config.json.
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
        assets: plugin_dir.join("assets"),
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
    fn clean_query_strips_trigger() {
        assert_eq!(clean_query("vc", "vc"), "");
        assert_eq!(clean_query("  vc  my app ", "vc"), "my app");
        assert_eq!(clean_query("vcfoo", "vc"), "foo");
        assert_eq!(clean_query("api vs", "vc"), "api vs");
    }
}
