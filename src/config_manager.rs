use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub interval_seconds: u64,
    pub child_binary_name: String,
}

pub fn load_config() -> Result<Config, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("settings.toml"))
        .add_source(config::Environment::with_prefix("AGENT").separator("__"))
        .build()?
        .try_deserialize()
}