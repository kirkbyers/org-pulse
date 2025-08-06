use chrono::{DateTime, Utc};
use sqlx::{pool::PoolConnection, query, query_as, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use anyhow::{Result};

const DB_URL: &str = "sqlite://org-pulse.db?mode=rwc";

pub async fn new_pool () -> Result<SqlitePool> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await.map_err(|e| e.into())
    
}

type PoolConn = PoolConnection<Sqlite>;

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

pub struct Contributor {
    pub id: u64,
    pub name: String,
}

pub struct Repo {
    pub id: u64,
    pub name: String,
    pub org: Org,
}

pub struct Scrape {
    pub id: u64,
    pub start_dt: DateTime<Utc>,
    pub end_dt: DateTime<Utc>,
}

pub struct RepoScrape {
    pub id: u64,
    pub scrape: Scrape,
    pub org: Org,
    pub repo: Repo,
    pub commits: u64,
    pub prs: u64,
    pub lines: u64,
}

pub struct ContributorScrapes {
    pub id: u64,
    pub repo_scrape: RepoScrape,
    pub contributor: Contributor,
    pub commits: u64,
    pub lines: u64,
}
