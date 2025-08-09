use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct OrgStats {
    pub name: String,
    pub total_commits: i64,
    pub total_lines: i64,
    pub repo_count: i64,
    pub contributor_count: i64,
}

#[derive(Debug, Clone)]
pub struct RepoStats {
    pub org_name: String,
    pub repo_name: String,
    pub commits: i64,
    pub lines: i64,
    pub prs: i64,
    pub contributor_count: i64,
}

#[derive(Debug, Clone)]
pub struct ContributorStats {
    pub username: String,
    pub total_commits: i64,
    pub total_lines: i64,
    pub repo_count: i64,
    pub orgs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScrapeInfo {
    pub id: i64,
    pub start_dt: DateTime<Utc>,
    pub end_dt: DateTime<Utc>,
    pub repo_count: i64,
}

// Detail view data structures for drill-down
#[derive(Debug, Clone)]
pub struct OrgDetail {
    pub org_name: String,
    pub repos: Vec<RepoStats>,
}

#[derive(Debug, Clone)]
pub struct RepoDetail {
    pub org_name: String,
    pub repo_name: String,
    pub contributors: Vec<RepoContributor>,
}

#[derive(Debug, Clone)]
pub struct RepoContributor {
    pub username: String,
    pub commits: i64,
    pub lines: i64,
    pub prs: i64,
}

#[derive(Debug, Clone)]
pub struct ContributorDetail {
    pub username: String,
    pub contributions: Vec<ContributorRepo>,
}

#[derive(Debug, Clone)]
pub struct ContributorRepo {
    pub org_name: String,
    pub repo_name: String,
    pub commits: i64,
    pub lines: i64,
    pub prs: i64,
}

#[derive(Debug, Clone)]
pub enum ViewData {
    Orgs(Vec<OrgStats>),
    Repos(Vec<RepoStats>),
    Contributors(Vec<ContributorStats>),
    OrgDetail(OrgDetail),
    RepoDetail(RepoDetail),
    ContributorDetail(ContributorDetail),
    Loading,
    Error(String),
}