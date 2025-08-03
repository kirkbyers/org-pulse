use std::collections::HashMap;

use octocrab::{models::{pulls::PullRequest, repos::RepoCommit}};
use regex::Regex;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scrape {
    pub org: String,
    pub repo: String,
    pub ignored_user_patterns: String,
    pub contributors: HashMap<String, ScrapeContributor>,
    pub commits: u64,
    pub prs: u64,
    pub lines: u64,
}

impl Scrape {
    pub fn new(org_name: &str, repo_name: &str, ignored_user_patterns: &str) -> Self {
        Scrape { 
            org: org_name.to_string(), 
            repo: repo_name.to_string(), 
            ignored_user_patterns: ignored_user_patterns.to_string(),
            contributors: HashMap::new(), 
            commits: 0,
            prs: 0,
            lines: 0
        }
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