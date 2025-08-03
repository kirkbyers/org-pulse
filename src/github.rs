use chrono::{DateTime, Utc};
use octocrab::{models::{orgs::MembershipInvitation, pulls::PullRequest, repos::RepoCommit, Repository}, FromResponse, Octocrab, Page};
use anyhow::Result;

const COMMITS_PER_PAGE: u8 = 200;

pub struct Github {
    client: Octocrab
}

impl Github {
    pub fn new(auth_token: &str) -> Self {
        Github {
            client: Octocrab::builder().personal_token(auth_token).build().expect("Failed to build Octocrab")
        }
    }

    pub async fn get_repo_commits(self: &Self, org: &str, repo: &str, since: DateTime<Utc>) -> Result<Page<RepoCommit>> {
        let repo_details = self.client.repos(org, repo);
        return match repo_details
            .list_commits()
            .since(since)
            .branch("main")
            .per_page(COMMITS_PER_PAGE)
            .send()
            .await {
                Ok(val) => Ok(val),
                Err(octocrab::Error::GitHub { source, .. }) if source.status_code.as_str() == "404" => {
                        // If main doesn't exist
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

    pub async fn get_repo_prs (self: &Self, org: &str, repo: &str, since: DateTime<Utc>) -> Result<Vec<PullRequest>> {
        let repo_details = self.client.pulls(org, repo)
            .list()
            .state(octocrab::params::State::All)
            .sort(octocrab::params::pulls::Sort::Updated)
            .direction(octocrab::params::Direction::Descending)
            .base("main")
            .per_page(COMMITS_PER_PAGE)
            .send()
            .await?;
        let mut res = vec![];

        for detail in repo_details {
            match detail.merged_at {
                Some(dt) => {
                    if dt < since {
                        return Ok(res)
                    }
                    let full_res: PullRequest = FromResponse::from_response(self.client._get(&detail.url).await?).await?;
                    res.push(full_res);
                },
                None => continue
            }
        }

        Ok(res)
    }

    pub async fn get_org_repos_by_page(self: &Self, org: &str, per_page: &u8, page: &u32) -> Result<Page<Repository>> {
        return self.client.orgs(org)
                    .list_repos()
                    .sort(octocrab::params::repos::Sort::Updated)
                    .direction(octocrab::params::Direction::Descending)
                    .per_page(*per_page)
                    .page(*page)
                    .send().await.map_err(|e| e.into())
    }

    pub async fn get_orgs(self: &Self) -> Result<Page<MembershipInvitation>> {
        return self.client.current()
            .list_org_memberships_for_authenticated_user()
            .send()
            .await.map_err(|e| e.into())
    }
}
