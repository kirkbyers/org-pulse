use crate::stats::{ViewData, ScrapeInfo};
use crate::db::{new_pool, Scrape, get_org_stats, get_repo_stats, get_contributor_stats};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct App {
    pub current_view: View,
    pub current_scrape: Option<i64>,
    pub scrapes: Vec<ScrapeInfo>,
    pub data: ViewData,
    pub sort_order: SortOrder,
    pub sort_field: SortField,
    pub selected_index: usize,
    pub should_quit: bool,
    pub is_scraping: bool,
    pub pending_view_switch: Option<View>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Org,
    Repo,
    Contributors,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortField {
    Name,
    Commits,
    Lines,
    Repos,
    Prs,
}

#[derive(Debug, Clone, PartialEq)]
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
            should_quit: false,
            is_scraping: false,
            pending_view_switch: None,
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
    }

    pub fn set_sort_field(&mut self, field: SortField) {
        if self.sort_field != field {
            self.sort_field = field;
            self.sort_order = SortOrder::Descending;
        } else {
            self.toggle_sort_order();
        }
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

    pub fn get_item_count(&self) -> usize {
        match &self.data {
            ViewData::Orgs(orgs) => orgs.len(),
            ViewData::Repos(repos) => repos.len(),
            ViewData::Contributors(contributors) => contributors.len(),
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
            }
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
            self.switch_view_with_data(view).await?;
        }
        Ok(())
    }
}