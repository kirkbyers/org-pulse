# TUI Implementation Tasks

## Phase 1: Basic TUI Structure

### Task 1.1: Setup Dependencies ✅
- [x] Add `ratatui = "0.26"` to Cargo.toml
- [x] Add `crossterm = "0.27"` to Cargo.toml  
- [x] Test basic compilation
- [x] Fixed compilation error by adding `Row` import to db.rs

### Task 1.2: Create Project Structure ✅
- [x] Create `src/app/` directory
- [x] Create `src/app/mod.rs` with basic module structure
- [x] Create `src/app/state.rs` with App struct and enums
- [x] Create `src/app/ui.rs` stub
- [x] Create `src/app/events.rs` stub
- [x] Update `src/lib.rs` to include new modules
- [x] Fixed ratatui Frame API usage for 0.26 compatibility
- [x] Removed unused imports to clean up warnings

### Task 1.3: Move Scraper Logic ✅
- [x] Create `src/scraper.rs`
- [x] Move current `main.rs` logic to `scraper::run_scrape()` function
- [x] Keep main.rs as TUI entry point
- [x] Test that scraper module compiles and works
- [x] Updated lib.rs to include scraper module
- [x] Replaced main.rs with complete TUI setup and event loop

### Task 1.4: Basic TUI Application ✅
- [x] Implement basic event loop in `main.rs`
- [x] Create minimal UI with placeholder table
- [x] Add quit functionality ('q' key)
- [x] Test basic TUI runs and can be quit
- [x] All components working together - full TUI infrastructure complete

## Phase 2: Database Integration

### Task 2.1: Fix Database Architecture Issues ✅
- [x] Review scrape.rs vs db.rs patterns
- [x] Move database operations from scrape.rs to use db.rs create/get methods
- [x] Ensure scrape.rs calls db.rs for all database operations
- [x] Test that existing scraping still works with db.rs methods
- [x] Refactored scraper.rs to use proper db.rs create/get methods
- [x] Added Clone derives to db structs for easier usage
- [x] Deprecated old scrape.rs patterns in favor of db.rs

### Task 2.2: Add Statistics Data Structures ✅
- [x] Add `src/stats.rs`
- [x] Define `OrgStats`, `RepoStats`, `ContributorStats` structs
- [x] Define `ScrapeInfo` struct
- [x] Add `ViewData` enum to hold different stat types
- [x] Reorganized statistics structures from app/state.rs to dedicated stats.rs module
- [x] Updated imports and module exports to use new stats module
- [x] Maintained clean separation between app state and statistics data structures

### Task 2.3: Extend Database with TUI Queries ✅
- [x] Add `Scrape::list_all()` method to get all scrapes
- [x] Add `Scrape::get_latest()` method to get most recent scrape
- [x] Add `get_org_stats(pool, scrape_id)` function
- [x] Add `get_repo_stats(pool, scrape_id)` function  
- [x] Add `get_contributor_stats(pool, scrape_id)` function
- [x] Test all new database functions
- [x] Implemented comprehensive SQL queries with proper aggregation
- [x] Added repo counting and contributor organization mapping
- [x] All functions return properly structured statistics data

### Task 2.4: Data Loading ✅
- [x] Implement data loading in App initialization
- [x] Load scrape list on startup
- [x] Default to latest scrape
- [x] Load initial org stats view
- [x] Test data loads correctly in TUI
- [x] Added async App::new_with_data() constructor
- [x] Integrated database queries with app initialization
- [x] Added data refresh methods for view switching
- [x] Proper error handling when no scrape data exists

## Phase 3: Core TUI Functionality ✅

### Task 3.1: View Management ✅
- [x] Implement view switching logic in app state
- [x] Add keyboard handlers for 'o', 'r', 'u' keys
- [x] Update UI to show current view
- [x] Refresh data when switching views
- [x] Test view switching works
- [x] Added pending view switch mechanism for async data loading
- [x] Integrated view switching with database queries
- [x] Updated main TUI loop to handle async view switches

### Task 3.2: Table Rendering ✅
- [x] Implement org stats table in UI
- [x] Implement repo stats table in UI
- [x] Implement contributor stats table in UI
- [x] Add table headers and formatting
- [x] Test all three views display correctly

### Task 3.3: Navigation ✅
- [x] Add selection index to app state
- [x] Implement up/down arrow and j/k navigation
- [x] Highlight selected row in tables
- [x] Handle wrap-around at list boundaries
- [x] Test navigation works in all views

### Task 3.4: Header and Footer ✅
- [x] Add header with current scrape info and view name
- [x] Add footer with keyboard shortcuts
- [x] Show current sort field and order
- [x] Test header/footer display correctly

## Phase 4: Sorting and Polish

### Task 4.1: Sorting Implementation ✅
- [x] Add sort logic to app state
- [x] Implement keyboard handlers for sort keys (s, n, c, l, p, R)
- [x] Sort data when sort field/order changes
- [x] Update UI to show current sort
- [x] Test sorting works for all fields in all views

### Task 4.2: Scrape Selection ✅
- [x] Add scrape selection mode/view
- [x] Allow browsing available scrapes
- [x] Switch between scrapes and reload data
- [x] Add keyboard shortcut to enter scrape selection
- [x] Test scrape selection works

### Task 4.3: Background Scraping
- [ ] Add "Start Scrape" functionality ('S' key)
- [ ] Show "Scraping..." indicator during operation
- [ ] Block TUI during scraping (foreground operation)
- [ ] Refresh data after scrape completes
- [ ] Handle scrape errors gracefully
- [ ] Test scraping from TUI works

### Task 4.4: Detail Views (Drill-down)
- [ ] Add detail view state to App
- [ ] Implement Enter key handling for drill-down:
  - [ ] Org → List of repos in that org
  - [ ] Repo → List of contributors for that repo
  - [ ] Contributor → List of repo contributions
- [ ] Add back navigation (Escape key)
- [ ] Update UI for detail views
- [ ] Test drill-down navigation works

### Task 4.5: Error Handling and Polish
- [ ] Add error display in UI
- [ ] Handle database connection errors
- [ ] Handle empty data gracefully
- [ ] Add loading indicators where needed
- [ ] Improve table formatting and alignment
- [ ] Test error scenarios

## Phase 5: Final Testing and Documentation

### Task 5.1: Integration Testing
- [ ] Test complete workflow: scrape → browse → drill-down
- [ ] Test all keyboard shortcuts work correctly
- [ ] Test with different data scenarios (empty, large datasets)
- [ ] Test error recovery

### Task 5.2: Documentation
- [ ] Update README with TUI instructions
- [ ] Document keyboard shortcuts
- [ ] Add usage examples
- [ ] Update help text in TUI

### Task 5.3: Code Cleanup
- [ ] Remove unused code from migration
- [ ] Add proper error handling throughout
- [ ] Add code documentation
- [ ] Run clippy and fix warnings

## Implementation Notes

- Each task should be completable independently and result in working code
- Test each task before moving to the next
- Focus on getting basic functionality working before adding polish
- Commit after completing each major task group
- Some tasks may need to be broken down further during implementation

## Current Status
- [x] Phase 1: Basic TUI Structure
- [x] Phase 2: Database Integration  
- [x] Phase 3: Core TUI Functionality
- [ ] Phase 4: Sorting and Polish
- [ ] Phase 5: Final Testing and Documentation

## Resume Notes

### Current State (Last Updated)
- **Completed**: Phases 1, 2 & 3, Tasks 4.1 & 4.2
- **Next Task**: Task 4.3: Background Scraping
- **Key Achievement**: Complete scrape selection system with browsing and switching functionality

### Resume Notes for Next Session
**Current Progress:**
- ✅ **Phases 1-3 Complete**: Full TUI infrastructure, database integration, and core functionality
- ✅ **Task 4.1 Complete**: Multi-field sorting system with keyboard shortcuts
- ✅ **Task 4.2 Complete**: Scrape selection and browsing interface

**Next Priority - Task 4.3: Background Scraping:**
1. **'S' Key Implementation**: Currently stubbed in events.rs line 94-96, needs actual scraper integration
2. **UI Indicators**: Add "Scraping..." status display and progress indication
3. **Async Integration**: Integrate with existing `scraper::run_scrape()` function
4. **Error Handling**: Graceful handling of scraping failures
5. **Data Refresh**: Auto-refresh scrape list and data after completion

**Key Implementation Areas:**
- **events.rs**: Replace stub 'S' key handler with async scraping call
- **state.rs**: Add scraping status tracking and progress state
- **ui.rs**: Add scraping indicators to header/footer
- **main.rs**: May need async scraping task management

**Current Keyboard Shortcuts:**
- Navigation: ↑↓/j/k, Views: o/r/u, Sort: s/n/c/l/p/R, Scrapes: t, Quit: q
- Scrape Selection: t (toggle), Enter (select), Esc (back)

**Architecture Status:**
- All core systems working: tables, navigation, sorting, scrape selection
- Database auto-migration functional
- Clean separation between sync UI and async operations
- Ready for final polish phase tasks

### Recent Progress Summary
1. ✅ **Phase 1**: Complete TUI infrastructure (dependencies, structure, scraper separation, basic app)
2. ✅ **Phase 2**: Complete database integration (architecture fixes, stats structures, TUI queries, data loading)  
3. ✅ **Phase 3**: Complete core TUI functionality (view management, table rendering, navigation, header/footer)
  - ✅ **Task 3.1**: Complete view management (o/r/u key switching with async data refresh)
  - ✅ **Task 3.2**: Complete table rendering (proper ratatui tables with headers, formatting, and responsive layout)
  - ✅ **Task 3.3**: Complete navigation system (up/down/j/k keys, row highlighting, wrap-around selection)
  - ✅ **Task 3.4**: Enhanced header and footer with comprehensive UI information
4. ✅ **Task 4.1**: Complete sorting implementation with multi-field support and keyboard shortcuts
5. ✅ **Task 4.2**: Complete scrape selection system with browsing interface and data switching

### Key Technical Implementation Details
- **App Structure**: `src/app/` with state.rs, events.rs, ui.rs modules
- **Database**: `src/db.rs` with comprehensive TUI query functions (`get_org_stats`, `get_repo_stats`, `get_contributor_stats`)
- **Statistics**: `src/stats.rs` with all data structures (`OrgStats`, `RepoStats`, `ContributorStats`, `ViewData`)
- **Async Integration**: Pending view switch mechanism bridges sync events with async data loading
- **Main Loop**: Handles UI rendering, events, and async view switch processing

### Database Query Functions Available
```rust
// In src/db.rs - ready to use
Scrape::list_all(pool) -> Vec<ScrapeInfo>
Scrape::get_latest(pool) -> Option<ScrapeInfo>
get_org_stats(pool, scrape_id) -> Vec<OrgStats>
get_repo_stats(pool, scrape_id) -> Vec<RepoStats>
get_contributor_stats(pool, scrape_id) -> Vec<ContributorStats>
```

### Task 3.2 Implementation Notes ✅
**Completed Features:**
- ✅ **Table Implementation**: All three table views (Organizations, Repositories, Contributors) now use proper ratatui Table widgets
- ✅ **Column Headers**: Added styled headers with bold formatting and blue background for all views  
- ✅ **Data Formatting**: Implemented `format_number()` helper for readable numeric display (1.5K, 2.3M format)
- ✅ **Responsive Layout**: Column widths optimized for each view type with appropriate percentages
- ✅ **Empty State Handling**: Graceful display when no data is available for any view
- ✅ **Organization Display**: Added smart truncation for contributor organizations (shows first 2 + count)

**Technical Implementation:**
- Fixed ratatui 0.26 API compatibility (Table::new requires widths parameter)
- Added proper error handling for data loading states
- All table rendering functions now extract data from ViewData enum correctly

### Task 3.3 Implementation Notes ✅
**Completed Features:**
- ✅ **Selection State**: `selected_index` field already existed in App struct with proper initialization
- ✅ **Keyboard Navigation**: Up/Down arrow keys and j/k Vim-style navigation implemented in events.rs
- ✅ **Visual Feedback**: Selected rows highlighted with dark gray background and white text
- ✅ **Wrap-around**: Navigation wraps from bottom to top and top to bottom seamlessly
- ✅ **Multi-view Support**: Navigation works consistently across Organizations, Repositories, and Contributors views
- ✅ **Footer Update**: Added navigation hints (↑↓/j/k: Navigate) to footer for user guidance

**Technical Implementation:**
- Row highlighting uses `enumerate()` to track row index and applies styling conditionally
- Selection state resets to 0 when switching views for consistent user experience
- Wrap-around handled by modulo arithmetic in `move_selection_up()` and `move_selection_down()`
- All three table rendering functions updated with identical selection highlighting logic

### Task 3.4 Implementation Notes ✅
**Completed Features:**
- ✅ **Enhanced Header**: Added item count and current selection (e.g., "3/25 items") to provide user context
- ✅ **Comprehensive Footer**: Split into two lines for better organization with all keyboard shortcuts
- ✅ **Navigation Info**: Clear display of available navigation keys (↑↓/j/k)
- ✅ **View Switching**: Organized view controls (o/r/u) for easy reference
- ✅ **Sort Information**: Real-time display of current sort field and order with directional arrows
- ✅ **Layout Optimization**: Increased footer height to 3 lines to accommodate comprehensive information

**Technical Implementation:**
- Made `get_item_count()` method public in App struct for header access
- Enhanced header with selection position display (current/total format)
- Reorganized footer into logical groups: Navigation | Views | Sort | Status
- Improved visual layout with proper spacing and categorization

### Task 4.1 Implementation Notes ✅
**Completed Features:**
- ✅ **Multi-field Sorting**: Complete sorting support for Name, Commits, Lines, Repos, PRs across all views
- ✅ **Keyboard Shortcuts**: All sort keys working (s: toggle order, n: name, c: commits, l: lines, p: PRs, R: repos)
- ✅ **Dynamic Sorting**: Data automatically sorted when sort field or order changes
- ✅ **View-specific Logic**: Smart handling of inapplicable fields (e.g., PRs field in org view falls back to commits)
- ✅ **Selection Reset**: Selection automatically resets to top after sorting for consistent UX
- ✅ **Real-time Updates**: Footer shows current sort field and order with directional arrows

**Technical Implementation:**
- Added `Copy` trait to `SortField` and `SortOrder` enums for efficient copying
- Implemented static sort methods (`sort_orgs_static`, `sort_repos_static`, `sort_contributors_static`) to avoid borrow checker conflicts
- Added `apply_sort()` method that automatically applies sorting to current data
- Integrated sorting into data refresh cycle so initial data loads are properly sorted
- Fallback sorting logic handles inapplicable fields gracefully (PRs → commits for orgs/contributors)

### Task 4.2 Implementation Notes ✅
**Completed Features:**
- ✅ **Scrape Selection Mode**: Added `ScrapeSelection` view to browse available scrapes
- ✅ **Interactive Table**: Professional scrape list with ID, dates, and repository count
- ✅ **Keyboard Navigation**: Up/down navigation with j/k support for scrape browsing
- ✅ **Easy Access**: 't' key toggles between scrape selection and data views
- ✅ **Seamless Switching**: Enter key selects scrape and automatically loads data
- ✅ **Smart UI**: Context-sensitive footer shows appropriate shortcuts for each mode
- ✅ **Data Integration**: Automatic data refresh when switching between scrapes

**Technical Implementation:**
- Extended `View` enum with `ScrapeSelection` variant
- Added `scrape_selected_index` field for independent navigation within scrapes
- Implemented context-sensitive event handling (different behavior in scrape selection mode)
- Created dedicated `draw_scrape_selection_table()` function with formatted date display
- Modified header to show scrape count and selection position
- Enhanced `handle_pending_view_switch()` to handle scrape selection logic

### File Locations
- Main TUI: `src/main.rs` 
- App State: `src/app/state.rs`
- Events: `src/app/events.rs`
- UI Rendering: `src/app/ui.rs` (contains table stubs to implement)
- Database: `src/db.rs`
- Statistics: `src/stats.rs`

### Testing Notes
- Project compiles successfully with `cargo check` and `cargo build`
- TUI loads with real database data (if available)
- View switching works with 'o', 'r', 'u' keys
- Quit works with 'q' key
- Error handling for missing scrape data

### Architecture Notes
- Clean separation between sync UI events and async database operations
- All data structures properly organized in dedicated modules
- Database queries return structured data ready for table display
- App state includes pending view switch mechanism for responsive UI