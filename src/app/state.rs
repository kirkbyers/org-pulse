use crate::stats::{ViewData, ScrapeInfo, OrgStats, RepoStats, ContributorStats};
use crate::db::{new_pool, Scrape, get_org_stats, get_repo_stats, get_contributor_stats, get_org_detail, get_repo_detail, get_contributor_detail};
use crate::scraper;
use anyhow::Result;

#[derive(Debug, Clone)]
enum DrillType {
    Org(String),
    Repo(String, String), // org_name, repo_name
    Contributor(String),
}

#[derive(Debug, Clone)]
pub struct App {
    pub current_view: View,
    pub current_scrape: Option<i64>,
    pub scrapes: Vec<ScrapeInfo>,
    pub data: ViewData,
    pub sort_order: SortOrder,
    pub sort_field: SortField,
    pub selected_index: usize,
    pub scrape_selected_index: usize,
    pub should_quit: bool,
    pub is_scraping: bool,
    pub scraping_error: Option<String>,
    pub pending_view_switch: Option<View>,
    pub start_scraping_requested: bool,
    pub drill_down_requested: bool,
    pub navigate_back_requested: bool,
    pub view_history: Vec<(View, String)>, // (view, context) for back navigation
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Org,
    Repo,
    Contributors,
    ScrapeSelection,
    OrgDetail,
    RepoDetail,
    ContributorDetail,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Name,
    Commits,
    Lines,
    Repos,
    Prs,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_view: View::Org,
            current_scrape: None,
            scrapes: Vec::new(),
            data: ViewData::Loading,
            sort_order: SortOrder::Descending,
            sort_field: SortField::Commits,
            selected_index: 0,
            scrape_selected_index: 0,
            should_quit: false,
            is_scraping: false,
            scraping_error: None,
            pending_view_switch: None,
            start_scraping_requested: false,
            drill_down_requested: false,
            navigate_back_requested: false,
            view_history: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn new_with_data() -> Result<Self> {
        let mut app = Self::default();
        
        // Initialize database connection and load data
        let db_pool = new_pool().await?;
        let mut db_conn = db_pool.acquire().await?;

        // Load all available scrapes
        app.scrapes = Scrape::list_all(&mut db_conn).await?;

        // Default to latest scrape if available
        if let Some(latest_scrape) = Scrape::get_latest(&mut db_conn).await? {
            app.current_scrape = Some(latest_scrape.id);
            // Set scrape selection index to the latest scrape
            if let Some(index) = app.scrapes.iter().position(|s| s.id == latest_scrape.id) {
                app.scrape_selected_index = index;
            }

            // Load initial org stats for the latest scrape
            let org_stats = get_org_stats(&mut db_conn, latest_scrape.id).await?;
            app.data = ViewData::Orgs(org_stats);
        } else {
            // No scrapes available
            app.data = ViewData::Error("No scrape data available. Run a scrape first.".to_string());
        }

        Ok(app)
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn switch_view(&mut self, view: View) {
        if self.current_view != view {
            self.current_view = view;
            self.selected_index = 0;
            self.data = ViewData::Loading;
        }
    }

    pub fn request_view_switch(&mut self, view: View) {
        if self.current_view != view {
            self.pending_view_switch = Some(view);
        }
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
        self.apply_sort();
    }

    pub fn set_sort_field(&mut self, field: SortField) {
        if self.sort_field != field {
            self.sort_field = field;
            self.sort_order = SortOrder::Descending;
        } else {
            self.toggle_sort_order();
        }
        self.apply_sort();
    }

    pub fn apply_sort(&mut self) {
        let sort_field = self.sort_field;
        let sort_order = self.sort_order;
        
        match &mut self.data {
            ViewData::Orgs(orgs) => {
                Self::sort_orgs_static(orgs, sort_field, sort_order);
            }
            ViewData::Repos(repos) => {
                Self::sort_repos_static(repos, sort_field, sort_order);
            }
            ViewData::Contributors(contributors) => {
                Self::sort_contributors_static(contributors, sort_field, sort_order);
            }
            ViewData::OrgDetail(detail) => {
                Self::sort_repos_static(&mut detail.repos, sort_field, sort_order);
            }
            ViewData::RepoDetail(detail) => {
                Self::sort_repo_contributors_static(&mut detail.contributors, sort_field, sort_order);
            }
            ViewData::ContributorDetail(detail) => {
                Self::sort_contributor_repos_static(&mut detail.contributions, sort_field, sort_order);
            }
            ViewData::Loading | ViewData::Error(_) => {}
        }
        // Reset selection to top after sorting
        self.selected_index = 0;
    }

    pub fn move_selection_up(&mut self) {
        let item_count = self.get_item_count();
        if item_count > 0 {
            self.selected_index = if self.selected_index == 0 {
                item_count - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn move_selection_down(&mut self) {
        let item_count = self.get_item_count();
        if item_count > 0 {
            self.selected_index = (self.selected_index + 1) % item_count;
        }
    }

    pub fn move_scrape_selection_up(&mut self) {
        if !self.scrapes.is_empty() {
            self.scrape_selected_index = if self.scrape_selected_index == 0 {
                self.scrapes.len() - 1
            } else {
                self.scrape_selected_index - 1
            };
        }
    }

    pub fn move_scrape_selection_down(&mut self) {
        if !self.scrapes.is_empty() {
            self.scrape_selected_index = (self.scrape_selected_index + 1) % self.scrapes.len();
        }
    }

    pub fn get_item_count(&self) -> usize {
        match &self.data {
            ViewData::Orgs(orgs) => orgs.len(),
            ViewData::Repos(repos) => repos.len(),
            ViewData::Contributors(contributors) => contributors.len(),
            ViewData::OrgDetail(detail) => detail.repos.len(),
            ViewData::RepoDetail(detail) => detail.contributors.len(),
            ViewData::ContributorDetail(detail) => detail.contributions.len(),
            ViewData::Loading | ViewData::Error(_) => 0,
        }
    }

    pub async fn refresh_current_view_data(&mut self) -> Result<()> {
        if let Some(scrape_id) = self.current_scrape {
            let db_pool = new_pool().await?;
            let mut db_conn = db_pool.acquire().await?;

            match self.current_view {
                View::Org => {
                    let org_stats = get_org_stats(&mut db_conn, scrape_id).await?;
                    self.data = ViewData::Orgs(org_stats);
                }
                View::Repo => {
                    let repo_stats = get_repo_stats(&mut db_conn, scrape_id).await?;
                    self.data = ViewData::Repos(repo_stats);
                }
                View::Contributors => {
                    let contributor_stats = get_contributor_stats(&mut db_conn, scrape_id).await?;
                    self.data = ViewData::Contributors(contributor_stats);
                }
                View::ScrapeSelection => {
                    // No data loading needed for scrape selection view
                }
                View::OrgDetail | View::RepoDetail | View::ContributorDetail => {
                    // Detail views are loaded through drill-down, not refresh_current_view_data
                    // This shouldn't be called for detail views, but we handle it gracefully
                }
            }
            // Apply current sort after loading data
            self.apply_sort();
        } else {
            self.data = ViewData::Error("No scrape selected".to_string());
        }
        Ok(())
    }

    pub async fn switch_view_with_data(&mut self, view: View) -> Result<()> {
        if self.current_view != view {
            self.current_view = view;
            self.selected_index = 0;
            self.data = ViewData::Loading;
            self.refresh_current_view_data().await?;
        }
        Ok(())
    }

    pub async fn handle_pending_view_switch(&mut self) -> Result<()> {
        if let Some(view) = self.pending_view_switch.take() {
            // Special handling for selecting a scrape when in scrape selection mode
            if self.current_view == View::ScrapeSelection && view != View::ScrapeSelection {
                self.select_current_scrape().await?;
            } else {
                self.switch_view_with_data(view).await?;
            }
        }
        Ok(())
    }

    pub async fn select_current_scrape(&mut self) -> Result<()> {
        if let Some(scrape) = self.scrapes.get(self.scrape_selected_index) {
            self.current_scrape = Some(scrape.id);
            self.current_view = View::Org; // Return to org view after selecting scrape
            self.selected_index = 0; // Reset selection
            self.refresh_current_view_data().await?;
        }
        Ok(())
    }

    fn sort_orgs_static(orgs: &mut Vec<OrgStats>, sort_field: SortField, sort_order: SortOrder) {
        match sort_field {
            SortField::Name => {
                orgs.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.name.cmp(&b.name),
                    SortOrder::Descending => b.name.cmp(&a.name),
                });
            }
            SortField::Commits => {
                orgs.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.total_commits.cmp(&b.total_commits),
                    SortOrder::Descending => b.total_commits.cmp(&a.total_commits),
                });
            }
            SortField::Lines => {
                orgs.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.total_lines.cmp(&b.total_lines),
                    SortOrder::Descending => b.total_lines.cmp(&a.total_lines),
                });
            }
            SortField::Repos => {
                orgs.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.repo_count.cmp(&b.repo_count),
                    SortOrder::Descending => b.repo_count.cmp(&a.repo_count),
                });
            }
            SortField::Prs => {
                // PRs not applicable to orgs, fallback to commits
                orgs.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.total_commits.cmp(&b.total_commits),
                    SortOrder::Descending => b.total_commits.cmp(&a.total_commits),
                });
            }
        }
    }

    fn sort_repos_static(repos: &mut Vec<RepoStats>, sort_field: SortField, sort_order: SortOrder) {
        match sort_field {
            SortField::Name => {
                repos.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.repo_name.cmp(&b.repo_name),
                    SortOrder::Descending => b.repo_name.cmp(&a.repo_name),
                });
            }
            SortField::Commits => {
                repos.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.commits.cmp(&b.commits),
                    SortOrder::Descending => b.commits.cmp(&a.commits),
                });
            }
            SortField::Lines => {
                repos.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.lines.cmp(&b.lines),
                    SortOrder::Descending => b.lines.cmp(&a.lines),
                });
            }
            SortField::Prs => {
                repos.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.prs.cmp(&b.prs),
                    SortOrder::Descending => b.prs.cmp(&a.prs),
                });
            }
            SortField::Repos => {
                // Repos field not applicable to repo view, fallback to commits
                repos.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.commits.cmp(&b.commits),
                    SortOrder::Descending => b.commits.cmp(&a.commits),
                });
            }
        }
    }

    fn sort_contributors_static(contributors: &mut Vec<ContributorStats>, sort_field: SortField, sort_order: SortOrder) {
        match sort_field {
            SortField::Name => {
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.username.cmp(&b.username),
                    SortOrder::Descending => b.username.cmp(&a.username),
                });
            }
            SortField::Commits => {
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.total_commits.cmp(&b.total_commits),
                    SortOrder::Descending => b.total_commits.cmp(&a.total_commits),
                });
            }
            SortField::Lines => {
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.total_lines.cmp(&b.total_lines),
                    SortOrder::Descending => b.total_lines.cmp(&a.total_lines),
                });
            }
            SortField::Repos => {
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.repo_count.cmp(&b.repo_count),
                    SortOrder::Descending => b.repo_count.cmp(&a.repo_count),
                });
            }
            SortField::Prs => {
                // PRs not applicable to contributors, fallback to commits
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.total_commits.cmp(&b.total_commits),
                    SortOrder::Descending => b.total_commits.cmp(&a.total_commits),
                });
            }
        }
    }

    fn sort_repo_contributors_static(contributors: &mut Vec<crate::stats::RepoContributor>, sort_field: SortField, sort_order: SortOrder) {
        match sort_field {
            SortField::Name => {
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.username.cmp(&b.username),
                    SortOrder::Descending => b.username.cmp(&a.username),
                });
            }
            SortField::Commits => {
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.commits.cmp(&b.commits),
                    SortOrder::Descending => b.commits.cmp(&a.commits),
                });
            }
            SortField::Lines => {
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.lines.cmp(&b.lines),
                    SortOrder::Descending => b.lines.cmp(&a.lines),
                });
            }
            SortField::Prs => {
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.prs.cmp(&b.prs),
                    SortOrder::Descending => b.prs.cmp(&a.prs),
                });
            }
            SortField::Repos => {
                // Repos not applicable to repo contributors, fallback to commits
                contributors.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.commits.cmp(&b.commits),
                    SortOrder::Descending => b.commits.cmp(&a.commits),
                });
            }
        }
    }

    fn sort_contributor_repos_static(contributions: &mut Vec<crate::stats::ContributorRepo>, sort_field: SortField, sort_order: SortOrder) {
        match sort_field {
            SortField::Name => {
                contributions.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.repo_name.cmp(&b.repo_name),
                    SortOrder::Descending => b.repo_name.cmp(&a.repo_name),
                });
            }
            SortField::Commits => {
                contributions.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.commits.cmp(&b.commits),
                    SortOrder::Descending => b.commits.cmp(&a.commits),
                });
            }
            SortField::Lines => {
                contributions.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.lines.cmp(&b.lines),
                    SortOrder::Descending => b.lines.cmp(&a.lines),
                });
            }
            SortField::Prs => {
                contributions.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.prs.cmp(&b.prs),
                    SortOrder::Descending => b.prs.cmp(&a.prs),
                });
            }
            SortField::Repos => {
                // Sort by repo name when repos field selected
                contributions.sort_by(|a, b| match sort_order {
                    SortOrder::Ascending => a.repo_name.cmp(&b.repo_name),
                    SortOrder::Descending => b.repo_name.cmp(&a.repo_name),
                });
            }
        }
    }

    pub fn request_scraping(&mut self) {
        self.start_scraping_requested = true;
    }

    pub fn request_drill_down(&mut self) {
        self.drill_down_requested = true;
    }

    pub fn request_navigate_back(&mut self) {
        self.navigate_back_requested = true;
    }

    pub fn start_scraping(&mut self) {
        self.is_scraping = true;
        self.scraping_error = None;
        self.start_scraping_requested = false;
    }

    pub fn finish_scraping_success(&mut self) {
        self.is_scraping = false;
        self.scraping_error = None;
    }

    pub fn finish_scraping_error(&mut self, error: String) {
        self.is_scraping = false;
        self.scraping_error = Some(error);
    }

    pub async fn refresh_after_scrape(&mut self) -> Result<()> {
        // Reload scrape list to include new scrape
        let pool = new_pool().await?;
        let mut db_conn = pool.acquire().await?;
        self.scrapes = Scrape::list_all(&mut db_conn).await?;
        
        // Switch to the latest scrape (which should be the new one)
        if let Some(latest) = self.scrapes.first() {
            self.current_scrape = Some(latest.id);
            self.refresh_current_view_data().await?;
        }
        
        Ok(())
    }

    pub async fn handle_scraping_request(&mut self) -> Result<()> {
        if self.start_scraping_requested {
            self.start_scraping();
            
            // Run the scrape (this will block the TUI as intended per plan)
            match scraper::run_scrape().await {
                Ok(()) => {
                    self.finish_scraping_success();
                    // Refresh data after successful scrape
                    self.refresh_after_scrape().await?;
                }
                Err(e) => {
                    self.finish_scraping_error(format!("Scrape failed: {}", e));
                }
            }
        }
        Ok(())
    }

    pub async fn handle_navigation_requests(&mut self) -> Result<()> {
        if self.drill_down_requested {
            self.drill_down_requested = false;
            self.drill_down().await?;
        }
        if self.navigate_back_requested {
            self.navigate_back_requested = false;
            self.navigate_back().await?;
        }
        Ok(())
    }

    // Drill-down navigation methods
    async fn drill_down(&mut self) -> Result<()> {
        if self.is_scraping {
            return Ok(()); // Don't allow drill-down during scraping
        }

        if let Some(scrape_id) = self.current_scrape {
            // Extract data to avoid borrow checker issues
            let drill_info = match &self.data {
                ViewData::Orgs(orgs) => {
                    orgs.get(self.selected_index).map(|org| {
                        (self.current_view.clone(), org.name.clone(), DrillType::Org(org.name.clone()))
                    })
                }
                ViewData::Repos(repos) => {
                    repos.get(self.selected_index).map(|repo| {
                        let context = format!("{}/{}", repo.org_name, repo.repo_name);
                        (self.current_view.clone(), context, DrillType::Repo(repo.org_name.clone(), repo.repo_name.clone()))
                    })
                }
                ViewData::Contributors(contributors) => {
                    contributors.get(self.selected_index).map(|contributor| {
                        (self.current_view.clone(), contributor.username.clone(), DrillType::Contributor(contributor.username.clone()))
                    })
                }
                ViewData::OrgDetail(detail) => {
                    detail.repos.get(self.selected_index).map(|repo| {
                        let context = format!("{}/{}", detail.org_name, repo.repo_name);
                        (self.current_view.clone(), context, DrillType::Repo(repo.org_name.clone(), repo.repo_name.clone()))
                    })
                }
                ViewData::RepoDetail(detail) => {
                    detail.contributors.get(self.selected_index).map(|contributor| {
                        (self.current_view.clone(), contributor.username.clone(), DrillType::Contributor(contributor.username.clone()))
                    })
                }
                _ => None,
            };

            if let Some((view, context, drill_type)) = drill_info {
                self.view_history.push((view, context));
                match drill_type {
                    DrillType::Org(org_name) => {
                        self.drill_into_org(scrape_id, &org_name).await?;
                    }
                    DrillType::Repo(org_name, repo_name) => {
                        self.drill_into_repo(scrape_id, &org_name, &repo_name).await?;
                    }
                    DrillType::Contributor(username) => {
                        self.drill_into_contributor(scrape_id, &username).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn navigate_back(&mut self) -> Result<()> {
        if let Some((previous_view, context)) = self.view_history.pop() {
            self.current_view = previous_view;
            self.selected_index = 0;
            self.data = ViewData::Loading;
            
            // Reload the previous view data
            match self.current_view {
                View::Org => self.refresh_current_view_data().await?,
                View::Repo => self.refresh_current_view_data().await?,
                View::Contributors => self.refresh_current_view_data().await?,
                View::OrgDetail => {
                    if let Some(scrape_id) = self.current_scrape {
                        let detail = get_org_detail(&mut self.get_db_connection().await?, scrape_id, &context).await?;
                        self.data = ViewData::OrgDetail(detail);
                    }
                }
                View::RepoDetail => {
                    if let Some(scrape_id) = self.current_scrape {
                        let parts: Vec<&str> = context.split('/').collect();
                        if parts.len() == 2 {
                            let detail = get_repo_detail(&mut self.get_db_connection().await?, scrape_id, parts[0], parts[1]).await?;
                            self.data = ViewData::RepoDetail(detail);
                        }
                    }
                }
                _ => {}
            }
            self.apply_sort();
        }
        Ok(())
    }

    async fn drill_into_org(&mut self, scrape_id: i64, org_name: &str) -> Result<()> {
        let mut db_conn = self.get_db_connection().await?;
        let detail = get_org_detail(&mut db_conn, scrape_id, org_name).await?;
        self.data = ViewData::OrgDetail(detail);
        self.current_view = View::OrgDetail;
        self.selected_index = 0;
        self.apply_sort();
        Ok(())
    }

    async fn drill_into_repo(&mut self, scrape_id: i64, org_name: &str, repo_name: &str) -> Result<()> {
        let mut db_conn = self.get_db_connection().await?;
        let detail = get_repo_detail(&mut db_conn, scrape_id, org_name, repo_name).await?;
        self.data = ViewData::RepoDetail(detail);
        self.current_view = View::RepoDetail;
        self.selected_index = 0;
        self.apply_sort();
        Ok(())
    }

    async fn drill_into_contributor(&mut self, scrape_id: i64, username: &str) -> Result<()> {
        let mut db_conn = self.get_db_connection().await?;
        let detail = get_contributor_detail(&mut db_conn, scrape_id, username).await?;
        self.data = ViewData::ContributorDetail(detail);
        self.current_view = View::ContributorDetail;
        self.selected_index = 0;
        self.apply_sort();
        Ok(())
    }

    async fn get_db_connection(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Sqlite>> {
        let pool = new_pool().await?;
        Ok(pool.acquire().await?)
    }
}