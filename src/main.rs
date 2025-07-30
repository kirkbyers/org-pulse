use std::env;
use chrono::{Duration, Utc};
use octocrab::{params::{repos::Sort, Direction}, Octocrab};
use org_pulse::github::{self, get_org_repos_by_page, get_repo_commits};
use regex::Regex;
use serde::{Deserialize, Serialize};

const CONFIG_APTH: &str = "./config.toml";

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    organizations: Vec<String>,
    days: usize,
    include_private: bool,
    rate_limit_delay_ms: usize,
    ignored_org_pattern: String,
    ignored_user_patterns: String,
    ignored_repo_patterns: String,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg: AppConfig = confy::load_path(CONFIG_APTH)?;

    let github_token:String = env::var("GITHUB_TOKEN")?;
    let octocrab = Octocrab::builder().personal_token(github_token).build()?;

    let orgs = octocrab.current()
        .list_org_memberships_for_authenticated_user()
        .send()
        .await?;
    let org_ignore_regex = Regex::new(&format!(r"{}", &cfg.ignored_org_pattern))?;
    for org in orgs {
        if let Some(_mat) = org_ignore_regex.find(&org.organization.login) {
            println!("Skipping {}", &org.organization.login);
            continue;
        }

        let mut results_count = 50;
        let mut page: u32 = 1;
        let seven_days_ago = Utc::now() - Duration::days(7);
        while results_count == 50 {
            let repos = get_org_repos_by_page(&octocrab, &org.organization.login, &results_count, &page).await?;
            results_count = 0;
            for repo in repos {
                results_count += 1;
                let commits_this_week = match get_repo_commits(&octocrab, &org.organization.login, &repo.name, seven_days_ago).await {
                    Ok(val) => val,
                    Err(_e) => continue
                };
                let mut commit_counter = 0;
                for _commit in commits_this_week {
                    commit_counter += 1;
                }
                if commit_counter == 0 {
                    break;
                }
                println!("{} commits in {} this week", &commit_counter, &repo.full_name.unwrap_or_default());

            }
            page += 1;
        }
    }
    Ok(())
}
