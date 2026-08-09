use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub interval_seconds: u64,
    pub child_binary_name: String,
    pub log_file_path: String,
}

pub fn load_config() -> Result<Config, config::ConfigError> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));

    let primary_settings_path = exe_dir.join("settings.toml");
    let fallback_settings_path = exe_dir.parent().map(|parent| parent.join("settings.toml"));

    let settings_path = if primary_settings_path.exists() {
        primary_settings_path
    } else if let Some(fallback) = fallback_settings_path {
        fallback
    } else {
        exe_dir.join("settings.toml")
    };

    let mut config: Config = config::Config::builder()
        .add_source(config::File::from(settings_path))
        .add_source(config::Environment::with_prefix("AGENT").separator("__"))
        .build()?
        .try_deserialize()?;

    config.child_binary_name = resolve_path(&config.child_binary_name, &exe_dir);
    config.log_file_path = resolve_path(&config.log_file_path, &exe_dir);

    Ok(config)
}

fn resolve_path(raw_path: &str, exe_dir: &Path) -> String {
    let raw_candidate = Path::new(raw_path);
    let mut candidates = Vec::new();

    candidates.push(raw_candidate.to_path_buf());

    if raw_candidate.is_relative() {
        candidates.push(exe_dir.join(raw_candidate));

        if let Some(parent) = exe_dir.parent() {
            candidates.push(parent.join(raw_candidate));
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            return fs::canonicalize(&candidate)
                .unwrap_or(candidate)
                .to_string_lossy()
                .to_string();
        }

        #[cfg(not(windows))]
        {
            if candidate.extension().and_then(|ext| ext.to_str()) == Some("exe") {
                let fallback = candidate.with_extension("");
                if fallback.exists() {
                    return fs::canonicalize(&fallback)
                        .unwrap_or(fallback)
                        .to_string_lossy()
                        .to_string();
                }
            }
        }

        #[cfg(windows)]
        {
            if candidate.extension().is_none() {
                let fallback = candidate.with_extension("exe");
                if fallback.exists() {
                    return fs::canonicalize(&fallback)
                        .unwrap_or(fallback)
                        .to_string_lossy()
                        .to_string();
                }
            }
        }

    }

    raw_path.to_string()
}