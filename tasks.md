# TUI Implementation Tasks

## Phase 1: Basic TUI Structure ✅

### Task 1.1: Setup Dependencies ✅
- [x] Add ratatui and crossterm dependencies
- [x] Fix compilation issues

### Task 1.2: Create Project Structure ✅
- [x] Create `src/app/` module with state, ui, events
- [x] Update lib.rs imports

### Task 1.3: Move Scraper Logic ✅
- [x] Extract scraping to `src/scraper.rs`
- [x] Convert main.rs to TUI entry point

### Task 1.4: Basic TUI Application ✅
- [x] Implement event loop and basic UI
- [x] Add quit functionality

## Phase 2: Database Integration ✅

### Task 2.1: Fix Database Architecture ✅
- [x] Consolidate database operations to use db.rs patterns

### Task 2.2: Add Statistics Data Structures ✅
- [x] Create `src/stats.rs` with OrgStats, RepoStats, ContributorStats

### Task 2.3: Extend Database with TUI Queries ✅
- [x] Add Scrape::list_all() and get_latest() methods
- [x] Add get_org_stats, get_repo_stats, get_contributor_stats functions

### Task 2.4: Data Loading ✅
- [x] Implement App::new_with_data() async constructor
- [x] Add data refresh methods

## Phase 3: Core TUI Functionality ✅

### Task 3.1: View Management ✅
- [x] Implement view switching (o/r/u keys)
- [x] Add pending view switch for async data loading

### Task 3.2: Table Rendering ✅
- [x] Implement all three table views with proper headers
- [x] Add number formatting and responsive layout

### Task 3.3: Navigation ✅
- [x] Add selection highlighting and arrow/j/k navigation
- [x] Implement wrap-around selection

### Task 3.4: Header and Footer ✅
- [x] Add comprehensive header with scrape info
- [x] Add multi-line footer with all keyboard shortcuts

## Phase 4: Sorting and Polish

### Task 4.1: Sorting Implementation ✅
- [x] Multi-field sorting with keyboard shortcuts
- [x] Dynamic sort order toggling

### Task 4.2: Scrape Selection ✅
- [x] Scrape browsing interface with 't' key
- [x] Seamless scrape switching

### Task 4.3: Background Scraping
- [ ] Add "Start Scrape" functionality ('S' key)
- [ ] Show "Scraping..." indicator during operation
- [ ] Block TUI during scraping (foreground operation)
- [ ] Refresh data after scrape completes
- [ ] Handle scrape errors gracefully

### Task 4.4: Detail Views (Drill-down)
- [ ] Add detail view state to App
- [ ] Implement Enter key handling for drill-down:
  - [ ] Org → List of repos in that org
  - [ ] Repo → List of contributors for that repo
  - [ ] Contributor → List of repo contributions
- [ ] Add back navigation (Escape key)
- [ ] Update UI for detail views

### Task 4.5: Error Handling and Polish
- [ ] Add error display in UI
- [ ] Handle database connection errors
- [ ] Handle empty data gracefully
- [ ] Add loading indicators where needed
- [ ] Improve table formatting and alignment

## Phase 5: Final Testing and Documentation

### Task 5.1: Integration Testing
- [ ] Test complete workflow: scrape → browse → drill-down
- [ ] Test all keyboard shortcuts work correctly
- [ ] Test with different data scenarios (empty, large datasets)

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

## Current Status
- ✅ Phases 1-3: Complete TUI infrastructure, database integration, and core functionality
- ✅ Tasks 4.1-4.2: Complete sorting and scrape selection
- 🔄 Next: Task 4.3 Background Scraping

## Key Implementation Files
- **Main TUI**: `src/main.rs`
- **App State**: `src/app/state.rs`
- **Events**: `src/app/events.rs` 
- **UI Rendering**: `src/app/ui.rs`
- **Database**: `src/db.rs`
- **Statistics**: `src/stats.rs`

## Next Steps
**Task 4.3: Background Scraping** - 'S' key implementation currently stubbed in events.rs:94-96, needs integration with scraper::run_scrape() function.