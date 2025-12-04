// cloudscanner/src/config/mod.rs

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProviderConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub provider_type: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "provider")]
    pub providers: Vec<ProviderConfig>,
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
