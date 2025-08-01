use std::collections::HashMap;

use octocrab::models::repos::RepoCommit;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scrape {
    org: String,
    repo: String,
    contributors: HashMap<String, ScrapeContributor>,
    commits: u64,
    prs: u64,
    lines: u64,
}

impl Scrape {
    pub fn new(org_name: &str, repo_name: &str) -> Self {
        Scrape { org: org_name.to_string(), repo: repo_name.to_string(), contributors: HashMap::new(), commits: 0, prs: 0, lines: 0 }
    }

    pub fn process_commit(self: &mut Self, commit: &RepoCommit) {
        // The only info we can get from this struct is author
        let commit_author = match &commit.author {
            Some(author) => author.login.clone(),
            None => "anonymous".to_string()
        };
        // TODO: Skip if username is in user ignore regex

        self.commits += 1;
        
        let scrape_contributor = self.contributors
            .entry(commit_author.clone())
            .or_insert_with(|| ScrapeContributor::new(&commit_author));
        scrape_contributor.commits += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeContributor {
    username: String,
    commits: u64,
    lines: u64,
    reviews: u64,
}

impl ScrapeContributor {
    pub fn new(user_name: &str) -> Self {
        ScrapeContributor { username: user_name.to_string(), commits: 0, lines: 0, reviews: 0 }
    }
}