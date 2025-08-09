use std::env;
use chrono::{Duration, Utc};
use anyhow::Result;
use regex::Regex;
use sqlx::migrate;
use std::collections::HashMap;

use crate::{config::get_config, db::{new_pool, Org, Repo, Scrape, RepoScrape, Contributor, ContributorScrapes}, github::Github};

// Temporary data structure to collect scrape data before saving to DB
#[derive(Debug)]
struct TempRepoScrape {
    contributors: HashMap<String, TempContributorData>,
    total_commits: i64,
    total_prs: i64,
    total_lines: i64,
}

#[derive(Debug)]
struct TempContributorData {
    username: String,
    commits: i64,
    lines: i64,
}

impl TempRepoScrape {
    fn new(_org_name: &str, _repo_name: &str) -> Self {
        Self {
            contributors: HashMap::new(),
            total_commits: 0,
            total_prs: 0,
            total_lines: 0,
        }
    }

    fn process_commit(&mut self, commit: &octocrab::models::repos::RepoCommit, user_ignore_regex: &Regex) -> Result<()> {
        let commit_author = match &commit.author {
            Some(author) => author.login.clone(),
            None => "anonymous".to_string()
        };

        if user_ignore_regex.find(&commit_author).is_some() {
            return Ok(());
        }

        self.total_commits += 1;
        
        let contributor = self.contributors
            .entry(commit_author.clone())
            .or_insert_with(|| TempContributorData {
                username: commit_author,
                commits: 0,
                lines: 0,
            });
        contributor.commits += 1;

        Ok(())
    }

    fn process_pr(&mut self, pr: &octocrab::models::pulls::PullRequest, user_ignore_regex: &Regex) -> Result<()> {
        let author = match pr.user.clone() {
            Some(auth) => auth.login,
            None => "anonymous".to_string()
        };

        if user_ignore_regex.find(&author).is_some() {
            return Ok(());
        }

        let contributor = self.contributors
            .entry(author.clone())
            .or_insert_with(|| TempContributorData {
                username: author,
                commits: 0,
                lines: 0,
            });

        let line_count = pr.additions.unwrap_or_default() + pr.deletions.unwrap_or_default();
        contributor.lines += line_count as i64;

        self.total_prs += 1;
        self.total_lines += line_count as i64;
        Ok(())
    }
}

/// Runs a complete scrape of GitHub organizations and repositories.
/// This function runs silently to avoid interfering with TUI display.
pub async fn run_scrape() -> Result<()> {
    let db_pool = new_pool().await?;
    migrate!().run(&db_pool).await?;
    let cfg = get_config().unwrap();
    let github_token: String = env::var("GITHUB_TOKEN")?;
    let gh = Github::new(&github_token);

    let orgs = gh.get_orgs().await?;
    let org_ignore_regex = Regex::new(&cfg.ignored_org_pattern.to_string())?;
    let user_ignore_regex = Regex::new(&cfg.ignored_user_patterns.to_string())?;
    
    // Create a new scrape session
    let start_time = Utc::now() - Duration::days(7);
    let end_time = Utc::now();
    
    let mut db_conn = db_pool.acquire().await?;
    let scrape = Scrape::create(&mut db_conn, start_time, end_time).await?;
    
    for org in orgs {
        if org_ignore_regex.find(&org.organization.login).is_some() {
            continue;
        }

        // Create or get org from database
        let db_org = Org::create(&mut db_conn, org.organization.login.clone()).await?;

        let mut results_count = 50;
        let mut page: u32 = 1;
        
        while results_count == 50 {
            let repos = gh.get_org_repos_by_page(&org.organization.login, &results_count, &page).await?;
            results_count = 0;
            
            for repo in repos {
                results_count += 1;
                
                // Create or get repo from database
                let db_repo = Repo::create(&mut db_conn, repo.name.clone(), db_org.clone()).await?;
                let mut temp_repo_scrape = TempRepoScrape::new(&org.organization.login, &repo.name);

                let commits_this_week = match gh.get_repo_commits(&org.organization.login, &repo.name, start_time).await {
                    Ok(val) => val,
                    Err(_e) => continue
                };

                // Process each commit for the week in the repo
                let mut commit_counter = 0;
                for commit in commits_this_week {
                    commit_counter += 1;
                    temp_repo_scrape.process_commit(&commit, &user_ignore_regex)?;
                }
                if commit_counter == 0 {
                    continue;
                }

                let repo_prs = gh.get_repo_prs(&org.organization.login, &repo.name, start_time).await?;
                for pr in repo_prs {
                    temp_repo_scrape.process_pr(&pr, &user_ignore_regex)?;
                }

                // Save repo scrape to database
                let repo_scrape = RepoScrape::create(
                    &mut db_conn,
                    scrape.id,
                    db_org.clone(),
                    db_repo,
                    temp_repo_scrape.total_commits,
                    temp_repo_scrape.total_prs,
                    temp_repo_scrape.total_lines,
                ).await?;

                // Save contributor data
                for (_username, temp_contributor) in temp_repo_scrape.contributors {
                    let db_contributor = Contributor::create(&mut db_conn, temp_contributor.username).await?;
                    let _contributor_scrape = ContributorScrapes::create(
                        &mut db_conn,
                        repo_scrape.id,
                        db_contributor,
                        temp_contributor.commits,
                        temp_contributor.lines,
                    ).await?;
                }

                // Repo processing completed silently to avoid TUI interference
            }
            page += 1;
        }
    }
    
    // Scrape completed silently to avoid TUI interference
    Ok(())
}