use std::env;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    organizations: Vec<String>,
    days: usize,
    include_private: bool,
    rate_limit_delay_ms: usize,
    ignored_users: Vec<String>,
    ignored_patterns: Vec<String>,
    github_token: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            organizations: vec![],
            days: 7,
            include_private: true,
            rate_limit_delay_ms: 500,
            ignored_users: vec![],
            ignored_patterns: vec![],
            github_token: String::new(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, org-pulse");

    let cfg = confy::load::<AppConfig>("org-pulse", None)?;

    let github_token:String = env::var("GITHUB_TOKEN")?;

    println!("{github_token:?}");

    Ok(())
}
