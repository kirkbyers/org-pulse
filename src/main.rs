use std::env;
use chrono::{Duration, Utc};
use org_pulse::{github::Github, scrape::Scrape};
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
    let gh = Github::new(&github_token);

    let orgs = gh.get_orgs().await?;
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
            let repos = gh.get_org_repos_by_page(&org.organization.login, &results_count, &page).await?;
            results_count = 0;
            for repo in repos {
                let mut repo_scrape = Scrape::new(&org.organization.login, &repo.name);
                results_count += 1;

                let commits_this_week = match gh.get_repo_commits(&org.organization.login, &repo.name, seven_days_ago).await {
                    Ok(val) => val,
                    Err(_e) => continue
                };

                // Process each commit for the week in the repo
                let mut commit_counter = 0;
                for commit in commits_this_week {
                    commit_counter += 1;
                    repo_scrape.process_commit(&commit);
                }
                if commit_counter == 0 {
                    break;
                }
                println!("{:#?}", repo_scrape);

            }
            page += 1;
        }
    }
    Ok(())
}
