use serde::{Deserialize, Serialize};
use anyhow::Result;

const CONFIG_APTH: &str = "./config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub organizations: Vec<String>,
    pub days: usize,
    pub include_private: bool,
    pub rate_limit_delay_ms: usize,
    pub ignored_org_pattern: String,
    pub ignored_user_patterns: String,
    pub ignored_repo_patterns: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            organizations: vec![],
            days: 7,
            include_private: true,
            rate_limit_delay_ms: 500,
            ignored_user_patterns: "".to_string(),
            ignored_repo_patterns: "".to_string(),
            ignored_org_pattern: "".to_string(),
        }
    }
}

pub fn get_config() -> Result<AppConfig> {
    let cfg: AppConfig = confy::load_path(CONFIG_APTH)?;
    Ok(cfg)
}
