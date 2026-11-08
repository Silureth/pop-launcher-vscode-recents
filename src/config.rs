use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub always_open_in_new_window: bool,
    pub hide_deleted_entries: bool,
    #[serde(rename = "vscode-cli-additional-args")]
    pub vscode_cli_additional_args: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            always_open_in_new_window: true,
            hide_deleted_entries: true,
            vscode_cli_additional_args: Vec::new(),
        }
    }
}

impl Config {
    /// Reads `config.json`, falling back to defaults if it is missing or invalid.
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_config_keeps_defaults() {
        let config: Config = serde_json::from_str(r#"{"hide_deleted_entries": false}"#).unwrap();
        assert!(config.always_open_in_new_window);
        assert!(!config.hide_deleted_entries);
        assert!(config.vscode_cli_additional_args.is_empty());
    }

    #[test]
    fn reads_additional_args() {
        let config: Config =
            serde_json::from_str(r#"{"vscode-cli-additional-args": ["--profile", "Work"]}"#)
                .unwrap();
        assert_eq!(config.vscode_cli_additional_args, ["--profile", "Work"]);
    }
}
