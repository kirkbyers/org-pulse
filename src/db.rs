use chrono::{DateTime, Utc};
use sqlx::{migrate::Migrator, pool::PoolConnection, query, query_as, sqlite::SqlitePoolOptions, Row, Sqlite, SqlitePool};
use anyhow::{Result};

const DB_URL: &str = "sqlite://org-pulse.db?mode=rwc";
static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn new_pool() -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await?;
    
    // Run migrations automatically
    MIGRATOR.run(&pool).await?;
    
    Ok(pool)
}

type PoolConn = PoolConnection<Sqlite>;

#[derive(Debug, Clone)]
pub struct Org {
    pub id: i64,
    pub name: String,
}

impl Org {
    pub async fn get(pool_con: &mut PoolConn, id: &i64) -> Result<Org> {
        let org_row: (i64, String) = query_as("
            SELECT id, name
            FROM orgs o
            WHERE o.id = $1 
            LIMIT 1;
        ").bind(id).fetch_one(pool_con.as_mut()).await?;

        Ok(Org {
            id: org_row.0,
            name: org_row.1
        })
    }

    pub async fn create(pool_con: &mut PoolConn, name: String) -> Result<Org> {
        let result = query("
            INSERT INTO orgs (name)
            VALUES ($1)
            ON CONFLICT(name) DO UPDATE SET name = name
            RETURNING id
        ")
            .bind(&name)
            .fetch_one(pool_con.as_mut())
            .await?;
        
        let id: i64 = result.get(0);
        
        Ok(Org { id, name })
    }

    pub async fn save(self: &Self, pool_con: &mut PoolConn) -> Result<()> {
        let _res = query("
            UPDATE orgs
            set name = $1
            where id = $2
        ")
            .bind(&self.name)
            .bind(self.id)
            .execute(pool_con.as_mut())
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Contributor {
    pub id: i64,
    pub username: String,
}

impl Contributor {
    pub async fn get(pool_con: &mut PoolConn, id: &i64) -> Result<Contributor> {
        let contributor_row: (i64, String) = query_as("
            SELECT id, username
            FROM contributors c
            WHERE c.id = $1 
            LIMIT 1;
        ").bind(id).fetch_one(pool_con.as_mut()).await?;

        Ok(Contributor {
            id: contributor_row.0,
            username: contributor_row.1
        })
    }

    pub async fn create(pool_con: &mut PoolConn, username: String) -> Result<Contributor> {
        let result = query("
            INSERT INTO contributors (username)
            VALUES ($1)
            ON CONFLICT(username) DO UPDATE SET username = username
            RETURNING id
        ")
            .bind(&username)
            .fetch_one(pool_con.as_mut())
            .await?;
        
        let id: i64 = result.get(0);
        
        Ok(Contributor { id, username })
    }

    pub async fn save(self: &Self, pool_con: &mut PoolConn) -> Result<()> {
        let _res = query("
            UPDATE contributors
            set username = $1
            where id = $2
        ")
            .bind(&self.username)
            .bind(self.id)
            .execute(pool_con.as_mut())
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub org: Org,
}

impl Repo {
    pub async fn get(pool_con: &mut PoolConn, id: &i64) -> Result<Repo> {
        let repo_row: (i64, String, i64) = query_as("
            SELECT r.id, r.name, r.org_id
            FROM repos r
            WHERE r.id = $1 
            LIMIT 1;
        ").bind(id).fetch_one(pool_con.as_mut()).await?;

        let org = Org::get(pool_con, &repo_row.2).await?;

        Ok(Repo {
            id: repo_row.0,
            name: repo_row.1,
            org
        })
    }

    pub async fn create(pool_con: &mut PoolConn, name: String, org: Org) -> Result<Repo> {
        let result = query("
            INSERT INTO repos (name, org_id)
            VALUES ($1, $2)
            ON CONFLICT(name, org_id) DO UPDATE SET name = name
            RETURNING id
        ")
            .bind(&name)
            .bind(org.id)
            .fetch_one(pool_con.as_mut())
            .await?;
        
        let id: i64 = result.get(0);
        
        Ok(Repo { id, name, org })
    }

    pub async fn save(self: &Self, pool_con: &mut PoolConn) -> Result<()> {
        let _res = query("
            UPDATE repos
            set name = $1, org_id = $2
            where id = $3
        ")
            .bind(&self.name)
            .bind(self.org.id)
            .bind(self.id)
            .execute(pool_con.as_mut())
            .await?;
        Ok(())
    }
}

pub struct Scrape {
    pub id: i64,
    pub start_dt: DateTime<Utc>,
    pub end_dt: DateTime<Utc>,
    pub repo_scrapes: Vec<RepoScrape>,
}

impl Scrape {
    pub async fn get(pool_con: &mut PoolConn, id: &i64) -> Result<Scrape> {
        let scrape_row: (i64, DateTime<Utc>, DateTime<Utc>) = query_as("
            SELECT id, start_dt, end_dt
            FROM scrapes s
            WHERE s.id = $1 
            LIMIT 1;
        ").bind(id).fetch_one(pool_con.as_mut()).await?;

        let repo_scrape_rows: Vec<(i64, i64, i64, i64, i64, i64, i64)> = query_as("
            SELECT id, scrape_id, org_id, repo_id, commits, prs, lines
            FROM repo_scrapes rs
            WHERE rs.scrape_id = $1;
        ").bind(id).fetch_all(pool_con.as_mut()).await?;

        let mut repo_scrapes = Vec::new();
        for row in repo_scrape_rows {
            let org = Org::get(pool_con, &row.2).await?;
            let repo = Repo::get(pool_con, &row.3).await?;
            
            let contributor_scrape_rows: Vec<(i64, i64, i64, i64, i64)> = query_as("
                SELECT id, repo_scrape_id, contributor_id, commits, lines
                FROM contributor_scrapes cs
                WHERE cs.repo_scrape_id = $1;
            ").bind(&row.0).fetch_all(pool_con.as_mut()).await?;

            let mut contributor_scrapes = Vec::new();
            for cs_row in contributor_scrape_rows {
                let contributor = Contributor::get(pool_con, &cs_row.2).await?;
                contributor_scrapes.push(ContributorScrapes {
                    id: cs_row.0,
                    contributor,
                    commits: cs_row.3,
                    lines: cs_row.4,
                });
            }

            repo_scrapes.push(RepoScrape {
                id: row.0,
                org,
                repo,
                commits: row.4,
                prs: row.5,
                lines: row.6,
                contributor_scrapes,
            });
        }

        Ok(Scrape {
            id: scrape_row.0,
            start_dt: scrape_row.1,
            end_dt: scrape_row.2,
            repo_scrapes,
        })
    }

    pub async fn create(pool_con: &mut PoolConn, start_dt: DateTime<Utc>, end_dt: DateTime<Utc>) -> Result<Scrape> {
        let result = query("
            INSERT INTO scrapes (start_dt, end_dt)
            VALUES ($1, $2)
            RETURNING id
        ")
            .bind(&start_dt)
            .bind(&end_dt)
            .fetch_one(pool_con.as_mut())
            .await?;
        
        let id: i64 = result.get(0);
        
        Ok(Scrape { 
            id, 
            start_dt, 
            end_dt, 
            repo_scrapes: Vec::new() 
        })
    }

    pub async fn save(self: &Self, pool_con: &mut PoolConn) -> Result<()> {
        let _res = query("
            UPDATE scrapes
            set start_dt = $1, end_dt = $2
            where id = $3
        ")
            .bind(&self.start_dt)
            .bind(&self.end_dt)
            .bind(self.id)
            .execute(pool_con.as_mut())
            .await?;
        
        for repo_scrape in &self.repo_scrapes {
            repo_scrape.save(pool_con).await?;
        }
        
        Ok(())
    }

    pub async fn list_all(pool_con: &mut PoolConn) -> Result<Vec<crate::stats::ScrapeInfo>> {
        let scrape_rows: Vec<(i64, DateTime<Utc>, DateTime<Utc>)> = query_as("
            SELECT id, start_dt, end_dt
            FROM scrapes
            ORDER BY start_dt DESC;
        ").fetch_all(pool_con.as_mut()).await?;

        let mut scrape_infos = Vec::new();
        for row in scrape_rows {
            // Count repos for this scrape
            let repo_count: (i64,) = query_as("
                SELECT COUNT(*) 
                FROM repo_scrapes 
                WHERE scrape_id = $1;
            ").bind(row.0).fetch_one(pool_con.as_mut()).await?;

            scrape_infos.push(crate::stats::ScrapeInfo {
                id: row.0,
                start_dt: row.1,
                end_dt: row.2,
                repo_count: repo_count.0,
            });
        }

        Ok(scrape_infos)
    }

    pub async fn get_latest(pool_con: &mut PoolConn) -> Result<Option<crate::stats::ScrapeInfo>> {
        let latest_scrape_result: Result<(i64, DateTime<Utc>, DateTime<Utc>), _> = query_as("
            SELECT id, start_dt, end_dt
            FROM scrapes
            ORDER BY start_dt DESC
            LIMIT 1;
        ").fetch_one(pool_con.as_mut()).await;

        match latest_scrape_result {
            Ok(row) => {
                // Count repos for this scrape
                let repo_count: (i64,) = query_as("
                    SELECT COUNT(*) 
                    FROM repo_scrapes 
                    WHERE scrape_id = $1;
                ").bind(row.0).fetch_one(pool_con.as_mut()).await?;

                Ok(Some(crate::stats::ScrapeInfo {
                    id: row.0,
                    start_dt: row.1,
                    end_dt: row.2,
                    repo_count: repo_count.0,
                }))
            }
            Err(_) => Ok(None),
        }
    }
}

pub struct RepoScrape {
    pub id: i64,
    pub org: Org,
    pub repo: Repo,
    pub commits: i64,
    pub prs: i64,
    pub lines: i64,
    pub contributor_scrapes: Vec<ContributorScrapes>,
}

impl RepoScrape {
    pub async fn get(pool_con: &mut PoolConn, id: &i64) -> Result<RepoScrape> {
        let repo_scrape_row: (i64, i64, i64, i64, i64, i64, i64) = query_as("
            SELECT id, scrape_id, org_id, repo_id, commits, prs, lines
            FROM repo_scrapes rs
            WHERE rs.id = $1 
            LIMIT 1;
        ").bind(id).fetch_one(pool_con.as_mut()).await?;

        let org = Org::get(pool_con, &repo_scrape_row.2).await?;
        let repo = Repo::get(pool_con, &repo_scrape_row.3).await?;

        let contributor_scrape_rows: Vec<(i64, i64, i64, i64, i64)> = query_as("
            SELECT id, repo_scrape_id, contributor_id, commits, lines
            FROM contributor_scrapes cs
            WHERE cs.repo_scrape_id = $1;
        ").bind(id).fetch_all(pool_con.as_mut()).await?;

        let mut contributor_scrapes = Vec::new();
        for cs_row in contributor_scrape_rows {
            let contributor = Contributor::get(pool_con, &cs_row.2).await?;
            contributor_scrapes.push(ContributorScrapes {
                id: cs_row.0,
                contributor,
                commits: cs_row.3,
                lines: cs_row.4,
            });
        }

        Ok(RepoScrape {
            id: repo_scrape_row.0,
            org,
            repo,
            commits: repo_scrape_row.4,
            prs: repo_scrape_row.5,
            lines: repo_scrape_row.6,
            contributor_scrapes,
        })
    }

    pub async fn create(pool_con: &mut PoolConn, scrape_id: i64, org: Org, repo: Repo, commits: i64, prs: i64, lines: i64) -> Result<RepoScrape> {
        let result = query("
            INSERT INTO repo_scrapes (scrape_id, org_id, repo_id, commits, prs, lines)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
        ")
            .bind(scrape_id)
            .bind(org.id)
            .bind(repo.id)
            .bind(commits)
            .bind(prs)
            .bind(lines)
            .fetch_one(pool_con.as_mut())
            .await?;
        
        let id: i64 = result.get(0);
        
        Ok(RepoScrape { 
            id, 
            org, 
            repo, 
            commits, 
            prs, 
            lines, 
            contributor_scrapes: Vec::new() 
        })
    }

    pub async fn save(self: &Self, pool_con: &mut PoolConn) -> Result<()> {
        let _res = query("
            UPDATE repo_scrapes
            set org_id = $1, repo_id = $2, commits = $3, prs = $4, lines = $5
            where id = $6
        ")
            .bind(self.org.id)
            .bind(self.repo.id)
            .bind(self.commits)
            .bind(self.prs)
            .bind(self.lines)
            .bind(self.id)
            .execute(pool_con.as_mut())
            .await?;
        
        for contributor_scrape in &self.contributor_scrapes {
            contributor_scrape.save(pool_con).await?;
        }
        
        Ok(())
    }
}

pub struct ContributorScrapes {
    pub id: i64,
    pub contributor: Contributor,
    pub commits: i64,
    pub lines: i64,
}

impl ContributorScrapes {
    pub async fn get(pool_con: &mut PoolConn, id: &i64) -> Result<ContributorScrapes> {
        let contributor_scrapes_row: (i64, i64, i64, i64, i64) = query_as("
            SELECT id, repo_scrape_id, contributor_id, commits, lines
            FROM contributor_scrapes cs
            WHERE cs.id = $1 
            LIMIT 1;
        ").bind(id).fetch_one(pool_con.as_mut()).await?;

        let contributor = Contributor::get(pool_con, &contributor_scrapes_row.2).await?;

        Ok(ContributorScrapes {
            id: contributor_scrapes_row.0,
            contributor,
            commits: contributor_scrapes_row.3,
            lines: contributor_scrapes_row.4,
        })
    }

    pub async fn create(pool_con: &mut PoolConn, repo_scrape_id: i64, contributor: Contributor, commits: i64, lines: i64) -> Result<ContributorScrapes> {
        let result = query("
            INSERT INTO contributor_scrapes (repo_scrape_id, contributor_id, commits, lines)
            VALUES ($1, $2, $3, $4)
            RETURNING id
        ")
            .bind(repo_scrape_id)
            .bind(contributor.id)
            .bind(commits)
            .bind(lines)
            .fetch_one(pool_con.as_mut())
            .await?;
        
        let id: i64 = result.get(0);
        
        Ok(ContributorScrapes { 
            id, 
            contributor, 
            commits, 
            lines 
        })
    }

    pub async fn save(self: &Self, pool_con: &mut PoolConn) -> Result<()> {
        let _res = query("
            UPDATE contributor_scrapes
            set contributor_id = $1, commits = $2, lines = $3
            where id = $4
        ")
            .bind(self.contributor.id)
            .bind(self.commits)
            .bind(self.lines)
            .bind(self.id)
            .execute(pool_con.as_mut())
            .await?;
        Ok(())
    }
}

// Statistics query functions for TUI

pub async fn get_org_stats(pool_con: &mut PoolConn, scrape_id: i64) -> Result<Vec<crate::stats::OrgStats>> {
    let org_stats_rows: Vec<(String, i64, i64, i64, i64)> = query_as("
        SELECT 
            o.name,
            SUM(rs.commits) as total_commits,
            SUM(rs.lines) as total_lines,
            COUNT(DISTINCT rs.repo_id) as repo_count,
            COUNT(DISTINCT cs.contributor_id) as contributor_count
        FROM orgs o
        JOIN repo_scrapes rs ON o.id = rs.org_id
        LEFT JOIN contributor_scrapes cs ON rs.id = cs.repo_scrape_id
        WHERE rs.scrape_id = $1
        GROUP BY o.id, o.name
        ORDER BY total_commits DESC;
    ").bind(scrape_id).fetch_all(pool_con.as_mut()).await?;

    let mut org_stats = Vec::new();
    for row in org_stats_rows {
        org_stats.push(crate::stats::OrgStats {
            name: row.0,
            total_commits: row.1,
            total_lines: row.2,
            repo_count: row.3,
            contributor_count: row.4,
        });
    }

    Ok(org_stats)
}

pub async fn get_repo_stats(pool_con: &mut PoolConn, scrape_id: i64) -> Result<Vec<crate::stats::RepoStats>> {
    let repo_stats_rows: Vec<(String, String, i64, i64, i64, i64)> = query_as("
        SELECT 
            o.name as org_name,
            r.name as repo_name,
            rs.commits,
            rs.lines,
            rs.prs,
            COUNT(DISTINCT cs.contributor_id) as contributor_count
        FROM repo_scrapes rs
        JOIN orgs o ON rs.org_id = o.id
        JOIN repos r ON rs.repo_id = r.id
        LEFT JOIN contributor_scrapes cs ON rs.id = cs.repo_scrape_id
        WHERE rs.scrape_id = $1
        GROUP BY rs.id, o.name, r.name, rs.commits, rs.lines, rs.prs
        ORDER BY rs.commits DESC;
    ").bind(scrape_id).fetch_all(pool_con.as_mut()).await?;

    let mut repo_stats = Vec::new();
    for row in repo_stats_rows {
        repo_stats.push(crate::stats::RepoStats {
            org_name: row.0,
            repo_name: row.1,
            commits: row.2,
            lines: row.3,
            prs: row.4,
            contributor_count: row.5,
        });
    }

    Ok(repo_stats)
}

pub async fn get_contributor_stats(pool_con: &mut PoolConn, scrape_id: i64) -> Result<Vec<crate::stats::ContributorStats>> {
    let contributor_stats_rows: Vec<(String, i64, i64, i64)> = query_as("
        SELECT 
            c.username,
            SUM(cs.commits) as total_commits,
            SUM(cs.lines) as total_lines,
            COUNT(DISTINCT rs.repo_id) as repo_count
        FROM contributors c
        JOIN contributor_scrapes cs ON c.id = cs.contributor_id
        JOIN repo_scrapes rs ON cs.repo_scrape_id = rs.id
        WHERE rs.scrape_id = $1
        GROUP BY c.id, c.username
        ORDER BY total_commits DESC;
    ").bind(scrape_id).fetch_all(pool_con.as_mut()).await?;

    let mut contributor_stats = Vec::new();
    for row in contributor_stats_rows {
        // Get the list of orgs this contributor worked in
        let org_names: Vec<(String,)> = query_as("
            SELECT DISTINCT o.name
            FROM contributors c
            JOIN contributor_scrapes cs ON c.id = cs.contributor_id
            JOIN repo_scrapes rs ON cs.repo_scrape_id = rs.id
            JOIN orgs o ON rs.org_id = o.id
            WHERE c.username = $1 AND rs.scrape_id = $2
            ORDER BY o.name;
        ").bind(&row.0).bind(scrape_id).fetch_all(pool_con.as_mut()).await?;

        let orgs: Vec<String> = org_names.into_iter().map(|org| org.0).collect();

        contributor_stats.push(crate::stats::ContributorStats {
            username: row.0,
            total_commits: row.1,
            total_lines: row.2,
            repo_count: row.3,
            orgs,
        });
    }

    Ok(contributor_stats)
}

// Detail view queries for drill-down functionality

pub async fn get_org_detail(pool_con: &mut PoolConn, scrape_id: i64, org_name: &str) -> Result<crate::stats::OrgDetail> {
    // Get all repos for this organization
    let repo_rows: Vec<(String, String, i64, i64, i64, i64)> = query_as("
        SELECT 
            o.name as org_name,
            r.name as repo_name,
            rs.commits,
            rs.lines,
            rs.prs,
            COUNT(DISTINCT cs.contributor_id) as contributor_count
        FROM orgs o
        JOIN repo_scrapes rs ON o.id = rs.org_id
        JOIN repos r ON rs.repo_id = r.id
        LEFT JOIN contributor_scrapes cs ON rs.id = cs.repo_scrape_id
        WHERE rs.scrape_id = $1 AND o.name = $2
        GROUP BY o.name, r.name, rs.commits, rs.lines, rs.prs
        ORDER BY rs.commits DESC;
    ").bind(scrape_id).bind(org_name).fetch_all(pool_con.as_mut()).await?;

    let repos = repo_rows.into_iter().map(|row| crate::stats::RepoStats {
        org_name: row.0,
        repo_name: row.1,
        commits: row.2,
        lines: row.3,
        prs: row.4,
        contributor_count: row.5,
    }).collect();

    Ok(crate::stats::OrgDetail {
        org_name: org_name.to_string(),
        repos,
    })
}

pub async fn get_repo_detail(pool_con: &mut PoolConn, scrape_id: i64, org_name: &str, repo_name: &str) -> Result<crate::stats::RepoDetail> {
    // Get all contributors for this specific repository
    let contributor_rows: Vec<(String, i64, i64)> = query_as("
        SELECT 
            c.username,
            cs.commits,
            cs.lines
        FROM contributors c
        JOIN contributor_scrapes cs ON c.id = cs.contributor_id
        JOIN repo_scrapes rs ON cs.repo_scrape_id = rs.id
        JOIN repos r ON rs.repo_id = r.id
        JOIN orgs o ON rs.org_id = o.id
        WHERE rs.scrape_id = $1 AND o.name = $2 AND r.name = $3
        ORDER BY cs.commits DESC;
    ").bind(scrape_id).bind(org_name).bind(repo_name).fetch_all(pool_con.as_mut()).await?;

    let contributors = contributor_rows.into_iter().map(|row| crate::stats::RepoContributor {
        username: row.0,
        commits: row.1,
        lines: row.2,
        prs: 0, // PRs not tracked at contributor level
    }).collect();

    Ok(crate::stats::RepoDetail {
        org_name: org_name.to_string(),
        repo_name: repo_name.to_string(),
        contributors,
    })
}

pub async fn get_contributor_detail(pool_con: &mut PoolConn, scrape_id: i64, username: &str) -> Result<crate::stats::ContributorDetail> {
    // Get all repositories this contributor worked on
    let contribution_rows: Vec<(String, String, i64, i64)> = query_as("
        SELECT 
            o.name as org_name,
            r.name as repo_name,
            cs.commits,
            cs.lines
        FROM contributors c
        JOIN contributor_scrapes cs ON c.id = cs.contributor_id
        JOIN repo_scrapes rs ON cs.repo_scrape_id = rs.id
        JOIN repos r ON rs.repo_id = r.id
        JOIN orgs o ON rs.org_id = o.id
        WHERE rs.scrape_id = $1 AND c.username = $2
        ORDER BY cs.commits DESC;
    ").bind(scrape_id).bind(username).fetch_all(pool_con.as_mut()).await?;

    let contributions = contribution_rows.into_iter().map(|row| crate::stats::ContributorRepo {
        org_name: row.0,
        repo_name: row.1,
        commits: row.2,
        lines: row.3,
        prs: 0, // PRs not tracked at contributor level
    }).collect();

    Ok(crate::stats::ContributorDetail {
        username: username.to_string(),
        contributions,
    })
}
