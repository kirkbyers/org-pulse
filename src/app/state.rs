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

#[derive(Debug, Clone)]
pub enum ViewData {
    Orgs(Vec<OrgStats>),
    Repos(Vec<RepoStats>),
    Contributors(Vec<ContributorStats>),
    Loading,
    Error(String),
}

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
    pub start_dt: chrono::DateTime<chrono::Utc>,
    pub end_dt: chrono::DateTime<chrono::Utc>,
    pub repo_count: i64,
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
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
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

    fn get_item_count(&self) -> usize {
        match &self.data {
            ViewData::Orgs(orgs) => orgs.len(),
            ViewData::Repos(repos) => repos.len(),
            ViewData::Contributors(contributors) => contributors.len(),
            ViewData::Loading | ViewData::Error(_) => 0,
        }
    }
}