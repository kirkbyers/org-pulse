use std::collections::HashMap;

use chrono::{DateTime, Utc};
use octocrab::models::{pulls::PullRequest, repos::RepoCommit};
use regex::Regex;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use sqlx::{pool::PoolConnection, query_as, Sqlite};

#[derive(Debug)]
pub struct Scrape {
    pub org: String,
    pub repo: String,
    pub ignored_user_patterns: String,
    pub contributors: HashMap<String, ScrapeContributor>,
    pub commits: u64,
    pub prs: u64,
    pub lines: u64,
    db_connection: PoolConnection<Sqlite>,
}

impl Scrape {
    pub async fn new(mut db_conn: PoolConnection<Sqlite>, org_name: &str, repo_name: &str, ignored_user_patterns: &str) -> Result<Self> {
        let (_db_org_id, _db_org_name) = create_org(&mut db_conn, org_name).await?;
        Ok(Scrape { 
            org: org_name.to_string(), 
            repo: repo_name.to_string(), 
            ignored_user_patterns: ignored_user_patterns.to_string(),
            contributors: HashMap::new(), 
            commits: 0,
            prs: 0,
            lines: 0,
            db_connection: db_conn
        })
    }

    pub fn process_commit(self: &mut Self, commit: &RepoCommit) -> Result<()> {
        // The only info we can get from this struct is author
        let commit_author = match &commit.author {
            Some(author) => author.login.clone(),
            None => "anonymous".to_string()
        };

        // Skip if username is in user ignore regex
        let user_ignore_regex = Regex::new(&format!(r"{}", &self.ignored_user_patterns))?;
        if let Some(_mat) = user_ignore_regex.find(&commit_author) {
            return Ok(());
        }

        self.commits += 1;
        
        let scrape_contributor = self.contributors
            .entry(commit_author.clone())
            .or_insert_with(|| ScrapeContributor::new(&commit_author));
        scrape_contributor.commits += 1;

        Ok(())
    }

    pub fn process_pr(self: &mut Self, pr: &PullRequest) -> Result<()> {
        let author = match pr.user.clone() {
            Some(auth) => auth.login,
            None => "anonymous".to_string()
        };
        // Skip if username is in user ignore regex
        let user_ignore_regex = Regex::new(&format!(r"{}", &self.ignored_user_patterns))?;
        if let Some(_mat) = user_ignore_regex.find(&author) {
            return Ok(());
        }

        let scrape_contributor = self.contributors
            .entry(author.clone())
            .or_insert_with(|| ScrapeContributor::new(&author));

        
        let line_count = pr.additions.unwrap_or_default() + pr.deletions.unwrap_or_default();
        scrape_contributor.lines += line_count;

        self.prs += 1;
        self.lines += line_count;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeContributor {
    pub username: String,
    pub commits: u64,
    pub lines: u64,
}

impl ScrapeContributor {
    pub fn new(user_name: &str) -> Self {
        ScrapeContributor { username: user_name.to_string(), commits: 0, lines: 0 }
    }
}

async fn create_org(pool_conn: &mut PoolConnection<Sqlite>, org_name: &str) -> Result<(u64, String)> {
    let org_rows: (u64, String) = query_as("
        INSERT OR IGNORE INTO orgs (name) VALUES ($1);
        SELECT id, name FROM orgs WHERE name = $1 LIMIT 1;
    ").bind(org_name).fetch_one(pool_conn.as_mut()).await?;

    Ok(org_rows)
}

async fn create_repo(pool_conn: &mut PoolConnection<Sqlite>, repo_name: &str, org_id: &i64) -> Result<(u64, String, u64)> {
    let repo_rows: (u64, String, u64) = query_as("
        INSERT OR IGNORE INTO repos (name, org_id) VALUES ($1, $2);
        SELECT id, name, org_id FROM repos WHERE name = $1 LIMIT 1;
    ").bind(repo_name).bind(org_id).fetch_one(pool_conn.as_mut()).await?;

    Ok(repo_rows)
}

async fn create_contributor(pool_conn: &mut PoolConnection<Sqlite>, contributor_username: &str) -> Result<(u64, String)> {
    let contributor_rows: (u64, String) = query_as("
        INSERT OR IGNORE INTO orgs (username) VALUES ($1);
        SELECT id, username FROM orgs WHERE username = $1 LIMIT 1;
    ").bind(contributor_username).fetch_one(pool_conn.as_mut()).await?;

    Ok(contributor_rows)
}

async fn create_scrape(
    pool_conn: &mut PoolConnection<Sqlite>,
    start_dt: &DateTime<Utc>,
    end_dt: &DateTime<Utc>,
    org_id: &i64,
    repo_id: &i64,
) -> Result<(u64, DateTime<Utc>,DateTime<Utc>, u64, u64, u64, u64, u64)> {
    let scrape_rows: (u64, DateTime<Utc>,DateTime<Utc>, u64, u64, u64, u64, u64) = query_as("
        INSERT OR IGNORE INTO scrapes (start_dt, end_dt, org_id, repo_id) VALUES ($1, $2, $3, $4)
        RETURNING id, start_dt, end_dt, org_id, repo_id, commits, prs, lines;
    ")
        .bind(start_dt)
        .bind(end_dt)
        .bind(org_id)
        .bind(repo_id)
        .fetch_one(pool_conn.as_mut()).await?;

    Ok(scrape_rows)
}

async fn create_scrape_contributor(
    pool_conn: &mut PoolConnection<Sqlite>,
    scrape_id: &i64,
    contributor_id: &i64,
) -> Result<(u64, u64, u64, u64, u64)> {
    let scrape_contributor_row: (u64, u64, u64, u64, u64) = query_as("
        INSERT OR IGNORE INTO contributor_scrapes (scrape_id, contributor_id) VALUES ($1, $2)
        RETURNING id, scrape_id, contributor_id, commits, lines
    ")
        .bind(scrape_id)
        .bind(contributor_id)
        .fetch_one(pool_conn.as_mut()).await?;
    Ok(scrape_contributor_row)
}
