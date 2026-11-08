use crate::config::Config;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Matcher, Utf32Str};
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;
use url::Url;

const EXECUTABLE: &str = "code";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Folder,
    Workspace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub kind: EntryType,
    pub name: String,
    /// Parent directory shown as the description (home shortened to `~`).
    pub path: String,
    /// Original VS Code URI (`file://` or `vscode-remote://`).
    pub uri: String,
    /// Local filesystem path, or `None` for remote URIs.
    pub local_path: Option<PathBuf>,
}

#[derive(Deserialize)]
struct WorkspaceJson {
    folder: Option<String>,
    workspace: Option<String>,
}

/// VS Code 1.120+ no longer keeps `history.recentlyOpenedPathsList` in the
/// global state DB, so recents are reconstructed from `workspaceStorage`:
/// each subfolder has a `workspace.json` naming the folder or workspace, and
/// the modification time of its `state.vscdb` tells when it was last used.
pub fn recently_opened(workspace_storage: &Path, home: &str, config: &Config) -> Vec<Entry> {
    let Ok(dirs) = fs::read_dir(workspace_storage) else {
        return Vec::new();
    };

    let mut found: Vec<(SystemTime, Entry)> = dirs
        .flatten()
        .filter_map(|dir| {
            let dir = dir.path();
            let text = fs::read_to_string(dir.join("workspace.json")).ok()?;
            let mtime = fs::metadata(dir.join("state.vscdb"))
                .ok()?
                .modified()
                .ok()?;
            let data: WorkspaceJson = serde_json::from_str(&text).ok()?;
            Some((mtime, to_entry(data, home)?))
        })
        .collect();

    found.sort_by_key(|(mtime, _)| Reverse(*mtime));

    let mut seen = HashSet::new();
    found
        .into_iter()
        .map(|(_, entry)| entry)
        .filter(|entry| seen.insert(entry.uri.clone()))
        .filter(
            |entry| match (&entry.local_path, config.hide_deleted_entries) {
                (Some(path), true) => path.exists(),
                _ => true,
            },
        )
        .collect()
}

fn to_entry(data: WorkspaceJson, home: &str) -> Option<Entry> {
    let (uri, kind) = match (data.workspace, data.folder) {
        (Some(uri), _) => (uri, EntryType::Workspace),
        (None, Some(uri)) => (uri, EntryType::Folder),
        (None, None) => return None,
    };

    let local_path = if uri.starts_with("file://") {
        Some(Url::parse(&uri).ok()?.to_file_path().ok()?)
    } else {
        None
    };
    let display = match &local_path {
        Some(path) => path.to_string_lossy().into_owned(),
        None => percent_decode_str(&uri).decode_utf8_lossy().into_owned(),
    };

    let (parent, base) = display.rsplit_once('/').unwrap_or(("", &display));
    let name = base
        .strip_suffix(".code-workspace")
        .unwrap_or(base)
        .to_string();

    Some(Entry {
        kind,
        name,
        path: shorten_home(parent, home),
        uri,
        local_path,
    })
}

fn shorten_home(path: &str, home: &str) -> String {
    if home.is_empty() || home == "/" {
        return path.to_string();
    }
    match path.strip_prefix(home) {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("~{rest}"),
        _ => path.to_string(),
    }
}

/// How literally the query appears in an entry; higher ranks first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MatchTier {
    /// Letters appear in order, but scattered.
    Fuzzy,
    /// Every query word appears unbroken somewhere in the parent path or name.
    ExactInPath,
    /// Every query word appears unbroken in the folder or workspace name.
    ExactInName,
}

/// Whether every whitespace-separated word of `query` occurs as an unbroken
/// substring of `text`. Case-insensitive unless the query has uppercase
/// letters, matching the fuzzy matcher's smart-case behaviour.
fn contains_all_words(text: &str, query: &str) -> bool {
    if query.chars().any(char::is_uppercase) {
        query.split_whitespace().all(|word| text.contains(word))
    } else {
        let text = text.to_lowercase();
        query
            .split_whitespace()
            .all(|word| text.contains(&word.to_lowercase()))
    }
}

/// Fuzzy-matches `query` against each entry's name and parent path and returns
/// the indices of the best matches, best first.
///
/// Entries whose name contains the query words unbroken come first, then those
/// whose path does, then scattered fuzzy matches. Within each group the fuzzy
/// score decides (word-start matches score highest, name matches count double),
/// and ties keep recently-opened order.
pub fn search(entries: &[Entry], query: &str, limit: usize) -> Vec<usize> {
    let pattern = Pattern::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT.match_paths());
    let mut buf = Vec::new();

    let mut scored: Vec<(MatchTier, u32, usize)> = entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let full = format!("{}/{}", entry.path, entry.name);
            let name_score = pattern.score(Utf32Str::new(&entry.name, &mut buf), &mut matcher);
            let full_score = pattern.score(Utf32Str::new(&full, &mut buf), &mut matcher);
            let score = name_score.map(|s| s * 2).max(full_score)?;
            let tier = if contains_all_words(&entry.name, query) {
                MatchTier::ExactInName
            } else if contains_all_words(&full, query) {
                MatchTier::ExactInPath
            } else {
                MatchTier::Fuzzy
            };
            Some((tier, score, index))
        })
        .collect();

    // Stable sort, so equal ranks stay in recently-opened order.
    scored.sort_by_key(|&(tier, score, _)| Reverse((tier, score)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, _, index)| index)
        .collect()
}

/// Launches VS Code on the entry, detached so it outlives the plugin.
pub fn open(entry: &Entry, config: &Config) -> std::io::Result<()> {
    let mut command = Command::new(EXECUTABLE);
    command.arg(if config.always_open_in_new_window {
        "--new-window"
    } else {
        "--reuse-window"
    });
    command.args(&config.vscode_cli_additional_args);

    match &entry.local_path {
        Some(path) => command.arg(path),
        None => command
            .arg(match entry.kind {
                EntryType::Workspace => "--file-uri",
                EntryType::Folder => "--folder-uri",
            })
            .arg(&entry.uri),
    };

    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;
    // Reap the child in the background so it does not linger as a zombie.
    std::thread::spawn(move || child.wait());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(folder: &str) -> Entry {
        to_entry(
            WorkspaceJson {
                folder: Some(folder.into()),
                workspace: None,
            },
            "/home/me",
        )
        .unwrap()
    }

    #[test]
    fn local_folder_with_space() {
        let e = entry("file:///home/me/projects/my%20app");
        assert_eq!(e.kind, EntryType::Folder);
        assert_eq!(e.name, "my app");
        assert_eq!(e.path, "~/projects");
        assert_eq!(
            e.local_path,
            Some(PathBuf::from("/home/me/projects/my app"))
        );
    }

    #[test]
    fn workspace_file_strips_extension() {
        let e = to_entry(
            WorkspaceJson {
                folder: None,
                workspace: Some("file:///home/me/ws/team.code-workspace".into()),
            },
            "/home/me",
        )
        .unwrap();
        assert_eq!(e.kind, EntryType::Workspace);
        assert_eq!(e.name, "team");
        assert_eq!(e.path, "~/ws");
    }

    #[test]
    fn remote_folder_is_decoded_and_has_no_local_path() {
        let e = entry("vscode-remote://wsl%2Bubuntu/home/me/remote-proj");
        assert_eq!(e.name, "remote-proj");
        assert_eq!(e.path, "vscode-remote://wsl+ubuntu/home/me");
        assert_eq!(e.local_path, None);
    }

    #[test]
    fn home_prefix_only_on_directory_boundary() {
        assert_eq!(shorten_home("/home/me", "/home/me"), "~");
        assert_eq!(shorten_home("/home/me/a", "/home/me"), "~/a");
        assert_eq!(shorten_home("/home/meow/a", "/home/me"), "/home/meow/a");
    }

    #[test]
    fn search_prefers_name_matches_and_respects_limit() {
        let entries = vec![
            entry("file:///home/me/api/frontend"),
            entry("file:///home/me/code/api"),
            entry("file:///home/me/code/other"),
        ];
        let results = search(&entries, "api", 8);
        assert_eq!(entries[results[0]].name, "api");
        assert_eq!(results.len(), 2);
        assert_eq!(search(&entries, "api", 1).len(), 1);
        assert!(search(&entries, "zzzqqq", 8).is_empty());
    }

    fn names(entries: &[Entry], query: &str) -> Vec<String> {
        search(entries, query, 8)
            .into_iter()
            .map(|i| entries[i].name.clone())
            .collect()
    }

    #[test]
    fn exact_substring_beats_scattered_letters() {
        // Listed in recently-opened order, scattered matches first.
        let entries: Vec<Entry> = ["a-p-i", "webapp-pinia", "myapiserver", "api-server"]
            .iter()
            .map(|n| entry(&format!("file:///home/me/p/{n}")))
            .collect();
        assert_eq!(
            names(&entries, "api"),
            ["api-server", "myapiserver", "a-p-i", "webapp-pinia"]
        );
    }

    #[test]
    fn exact_in_name_beats_exact_in_path() {
        let entries = vec![
            entry("file:///home/me/projects/my%20app"),
            entry("file:///home/me/code/remote-proj"),
        ];
        assert_eq!(names(&entries, "proj"), ["remote-proj", "my app"]);
    }

    #[test]
    fn multi_word_and_smart_case() {
        assert!(contains_all_words("my app server", "app my"));
        assert!(contains_all_words("MyApp", "myapp"));
        assert!(!contains_all_words("myapp", "MyApp"));
    }
}
