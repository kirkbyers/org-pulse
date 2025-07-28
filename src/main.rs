use std::env;
use chrono::{Duration, Utc};
use octocrab::{params::{repos::Sort, Direction}, Octocrab};
use regex::Regex;
use serde::{Deserialize, Serialize};

const CONFIG_APTH: &str = "./config.toml";
const COMMITS_PER_PAGE: u8 = 200;

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
            let repos = octocrab.orgs(&org.organization.login)
                .list_repos()
                .sort(Sort::Updated)
                .direction(Direction::Descending)
                .per_page(results_count)
                .page(page)
                .send()
                .await?;
            results_count = 0;
            for repo in repos {
                results_count += 1;
                let repo_details = octocrab.repos(&org.organization.login, &repo.name);
                let commits_this_week = match repo_details
                    .list_commits()
                    .since(seven_days_ago.clone())
                    .branch("main")
                    .per_page(COMMITS_PER_PAGE)
                    .send()
                    .await {
                        Ok(val) => val,
                        Err(octocrab::Error::GitHub { source, .. }) if source.status_code.as_str() == "404" => {
                                // If master isn't 
                                repo_details
                                    .list_commits()
                                    .since(seven_days_ago.clone())
                                    .branch("master")
                                    .per_page(COMMITS_PER_PAGE)
                                    .send()
                                    .await?
                        },
                        Err(e) => {
                            println!("Unknown error for {:?}: {:?}", &repo.full_name.unwrap_or_default(), &e);
                            continue;
                        },
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
