//! Configuration file support for codex-skills.

use std::path::PathBuf;

use serde::Deserialize;

/// Configuration options for codex-skills.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Default number of results to show in pick command
    pub default_top: usize,
    /// Default clip length for summaries
    pub clip_length: usize,
    /// Default skills directory
    pub skills_dir: Option<PathBuf>,
}

impl Config {
    /// Load configuration from the default config file location.
    /// Returns default config if file doesn't exist.
    pub fn load() -> Self {
        Self::load_from_paths(&[
            // Current directory
            PathBuf::from(".codex-skills.toml"),
            PathBuf::from("codex-skills.toml"),
            // Home directory
            dirs_config_path(),
        ])
    }

    /// Load configuration from a list of paths, using the first one that exists.
    fn load_from_paths(paths: &[PathBuf]) -> Self {
        for path in paths {
            if path.exists() {
                if let Ok(contents) = std::fs::read_to_string(path) {
                    if let Ok(config) = toml::from_str(&contents) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    /// Get the default top value (3 if not configured).
    pub fn get_default_top(&self) -> usize {
        if self.default_top > 0 {
            self.default_top
        } else {
            3
        }
    }

    /// Get the clip length (80 if not configured).
    pub fn get_clip_length(&self) -> usize {
        if self.clip_length > 0 {
            self.clip_length
        } else {
            80
        }
    }
}

/// Get the config path in the user's home directory.
fn dirs_config_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".config").join("codex-skills").join("config.toml")
    } else {
        PathBuf::from(".codex-skills.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = Config::default();
        assert_eq!(config.get_default_top(), 3);
        assert_eq!(config.get_clip_length(), 80);
    }

    #[test]
    fn test_config_with_custom_values() {
        let config = Config {
            default_top: 5,
            clip_length: 100,
            skills_dir: Some(PathBuf::from("/custom/path")),
        };
        assert_eq!(config.get_default_top(), 5);
        assert_eq!(config.get_clip_length(), 100);
    }
}
