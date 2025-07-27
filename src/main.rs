use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    organizations: Vec<String>,
    days: usize,
    include_private: bool,
    rate_limit_delay_ms: usize,
    ignored_users: Vec<String>,
    ignored_patterns: Vec<String>,
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
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, org-pulse");

    let cfg = confy::load::<AppConfig>("org-pulse", None)?;
    println!("{cfg:?}");

    Ok(())
}
