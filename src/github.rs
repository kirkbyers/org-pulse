use chrono::{DateTime, Utc};
use octocrab::{models::{repos::RepoCommit, Repository}, params::{repos::Sort, Direction}, repos::RepoHandler, Octocrab, Page};
use anyhow::Result;

const COMMITS_PER_PAGE: u8 = 200;

pub async fn get_repo_commits(oc: &Octocrab, org: &str, repo: &str, since: DateTime<Utc>) -> Result<Page<RepoCommit>> {
    let repo_details = oc.repos(org, repo);
    return match repo_details
        .list_commits()
        .since(since)
        .branch("main")
        .per_page(COMMITS_PER_PAGE)
        .send()
        .await {
            Ok(val) => Ok(val),
            Err(octocrab::Error::GitHub { source, .. }) if source.status_code.as_str() == "404" => {
                    // If master isn't 
                    repo_details
                        .list_commits()
                        .since(since)
                        .branch("master")
                        .per_page(COMMITS_PER_PAGE)
                        .send()
                        .await
                        .map_err(|e| e.into())
            },
            Err(e) => {
                println!("Unknown error for {:?}: {:?}", repo, &e);
                Err(e.into())
            },
        };
}

pub async fn get_org_repos_by_page(oc: &Octocrab, org: &str, per_page: &u8, page: &u32) -> Result<Page<Repository>> {
    return oc.orgs(org)
                .list_repos()
                .sort(Sort::Updated)
                .direction(Direction::Descending)
                .per_page(*per_page)
                .page(*page)
                .send().await.map_err(|e| e.into())
}