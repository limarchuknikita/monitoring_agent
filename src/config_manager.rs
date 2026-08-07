use serde::Deserialize;
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

    if Path::new(&config.child_binary_name).is_relative() {
        config.child_binary_name = exe_dir
            .join(&config.child_binary_name)
            .to_string_lossy()
            .to_string();
    }

    if Path::new(&config.log_file_path).is_relative() {
        config.log_file_path = exe_dir
            .join(&config.log_file_path)
            .to_string_lossy()
            .to_string();
    }

    Ok(config)
}