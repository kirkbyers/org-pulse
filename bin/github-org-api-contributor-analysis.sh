#!/bin/bash

# Script to analyze git contributors across all repositories in a GitHub organization
# Uses GitHub API exclusively - no cloning required
# Finds the largest contributor by commits, lines changed, and PR reviews

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Global variables
TEMP_DIR=$(mktemp -d)
COMMITS_FILE="$TEMP_DIR/commits.txt"
LINES_FILE="$TEMP_DIR/lines.txt"
REVIEWS_FILE="$TEMP_DIR/reviews.txt"
REPO_COMMITS_FILE="$TEMP_DIR/repo_commits.txt"
REPO_LINES_FILE="$TEMP_DIR/repo_lines.txt"
REPO_REVIEWS_FILE="$TEMP_DIR/repo_reviews.txt"

# Initialize temp files
touch "$COMMITS_FILE" "$LINES_FILE" "$REVIEWS_FILE" "$REPO_COMMITS_FILE" "$REPO_LINES_FILE" "$REPO_REVIEWS_FILE"

# Configuration variables
ORGANIZATION=""
GITHUB_TOKEN=""
DAYS_BACK=7
REPO_FILTER=""
INCLUDE_PRIVATE=false
RATE_LIMIT_DELAY=1.0
MAX_REPOS=1000

# Statistics
repos_analyzed=0
repos_with_activity=0
api_calls_made=0
rate_limit_remaining=5000

# Cleanup function
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Usage function
usage() {
    echo "Usage: $0 --org ORGANIZATION [OPTIONS]"
    echo ""
    echo "Required:"
    echo "  --org ORGANIZATION        GitHub organization name to analyze"
    echo ""
    echo "Options:"
    echo "  --token TOKEN            GitHub personal access token (default: from 'gh auth token')"
    echo "  --since DAYS             Number of days to look back (default: 7)"
    echo "  --repos PATTERN          Filter repositories by name pattern (default: all repos)"
    echo "  --include-private        Include private repositories (default: public only)"
    echo "  --rate-limit-delay SEC   Delay between API calls in seconds (default: 0.5)"
    echo "  --max-repos NUM          Maximum number of repositories to analyze (default: 1000)"
    echo "  -h, --help               Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --org facebook                              # Analyze all public Facebook repos"
    echo "  $0 --org mycompany --include-private           # Include private repos (requires token)"
    echo "  $0 --org microsoft --repos react --since 14   # Analyze React repos from last 2 weeks"
    echo "  $0 --org google --max-repos 50                 # Limit to first 50 repositories"
    echo ""
    echo "Authentication:"
    echo "  - For public repositories: No authentication required"
    echo "  - For private repositories: Requires GitHub token with 'repo' scope"
    echo "  - Token can be provided via --token or 'gh auth token' command"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --org)
            ORGANIZATION="$2"
            shift 2
            ;;
        --token)
            GITHUB_TOKEN="$2"
            shift 2
            ;;
        --since)
            DAYS_BACK="$2"
            shift 2
            ;;
        --repos)
            REPO_FILTER="$2"
            shift 2
            ;;
        --include-private)
            INCLUDE_PRIVATE=true
            shift
            ;;
        --rate-limit-delay)
            RATE_LIMIT_DELAY="$2"
            shift 2
            ;;
        --max-repos)
            MAX_REPOS="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Validate required parameters
if [ -z "$ORGANIZATION" ]; then
    echo -e "${RED}Error: Organization name is required${NC}"
    usage
    exit 1
fi

# Check if gh CLI is available
if ! command -v gh >/dev/null 2>&1; then
    echo -e "${RED}Error: GitHub CLI (gh) is required but not installed${NC}"
    echo "Please install GitHub CLI: https://cli.github.com/"
    exit 1
fi

# Check if jq is available
if ! command -v jq >/dev/null 2>&1; then
    echo -e "${RED}Error: jq is required but not installed${NC}"
    echo "Please install jq: https://stedolan.github.io/jq/"
    exit 1
fi

# Get GitHub token if not provided
if [ -z "$GITHUB_TOKEN" ]; then
    if command -v gh >/dev/null 2>&1; then
        GITHUB_TOKEN=$(gh auth token 2>/dev/null || echo "")
    fi
fi

# Set authentication for gh commands
if [ -n "$GITHUB_TOKEN" ]; then
    export GH_TOKEN="$GITHUB_TOKEN"
    echo -e "${GREEN}✅ Using GitHub authentication${NC}"
else
    echo -e "${YELLOW}⚠️  No GitHub token found - only public repositories will be accessible${NC}"
fi

# Calculate since date
SINCE_DATE=$(date -d "$DAYS_BACK days ago" -Iseconds 2>/dev/null || date -v-${DAYS_BACK}d -Iseconds 2>/dev/null)
if [ -z "$SINCE_DATE" ]; then
    echo -e "${RED}Error: Unable to calculate since date${NC}"
    exit 1
fi

echo -e "${BLUE}🔍 Analyzing GitHub organization: $ORGANIZATION${NC}"
echo -e "${BLUE}📅 Looking back $DAYS_BACK days (since: $SINCE_DATE)${NC}"
echo ""

# Rate limiting helper function
check_rate_limit() {
    if [ -n "$GITHUB_TOKEN" ]; then
        local rate_info
        rate_info=$(gh api rate_limit --jq '.rate.remaining' 2>/dev/null || echo "unknown")
        if [ "$rate_info" != "unknown" ] && [ "$rate_info" -lt 100 ]; then
            echo -e "${YELLOW}⚠️  Rate limit low ($rate_info remaining), adding delay...${NC}"
            sleep 2
        fi
    fi
    sleep "$RATE_LIMIT_DELAY"
}

# Helper functions for data management
add_commits_for_author() {
    local author="$1"
    local count="$2"
    local existing_count=$(grep "^$author:" "$COMMITS_FILE" 2>/dev/null | cut -d: -f2 || echo "0")
    local new_count=$((existing_count + count))
    
    grep -v "^$author:" "$COMMITS_FILE" > "$COMMITS_FILE.tmp" 2>/dev/null || true
    echo "$author:$new_count" >> "$COMMITS_FILE.tmp"
    mv "$COMMITS_FILE.tmp" "$COMMITS_FILE"
}

add_lines_for_author() {
    local author="$1"
    local count="$2"
    local existing_count=$(grep "^$author:" "$LINES_FILE" 2>/dev/null | cut -d: -f2 || echo "0")
    local new_count=$((existing_count + count))
    
    grep -v "^$author:" "$LINES_FILE" > "$LINES_FILE.tmp" 2>/dev/null || true
    echo "$author:$new_count" >> "$LINES_FILE.tmp"
    mv "$LINES_FILE.tmp" "$LINES_FILE"
}

add_reviews_for_author() {
    local author="$1"
    local count="$2"
    local existing_count=$(grep "^$author:" "$REVIEWS_FILE" 2>/dev/null | cut -d: -f2 || echo "0")
    local new_count=$((existing_count + count))
    
    grep -v "^$author:" "$REVIEWS_FILE" > "$REVIEWS_FILE.tmp" 2>/dev/null || true
    echo "$author:$new_count" >> "$REVIEWS_FILE.tmp"
    mv "$REVIEWS_FILE.tmp" "$REVIEWS_FILE"
}

# Function to get all repositories from the organization
get_organization_repos() {
    echo -e "${CYAN}📡 Fetching repositories from organization: $ORGANIZATION${NC}"
    
    local repos=""
    if [ "$INCLUDE_PRIVATE" = true ]; then
        # Get both public and private repositories
        local public_repos=$(gh repo list "$ORGANIZATION" --visibility public --limit "$MAX_REPOS" --json name --jq '.[] | .name' 2>/dev/null || echo "")
        local private_repos=$(gh repo list "$ORGANIZATION" --visibility private --limit "$MAX_REPOS" --json name --jq '.[] | .name' 2>/dev/null || echo "")
        repos=$(printf "%s\n%s" "$public_repos" "$private_repos" | grep -v '^$' | head -n "$MAX_REPOS")
    else
        repos=$(gh repo list "$ORGANIZATION" --visibility public --limit "$MAX_REPOS" --json name --jq '.[] | .name' 2>/dev/null || echo "")
    fi
    
    if [ -z "$repos" ]; then
        echo -e "${RED}Error: No repositories found for organization '$ORGANIZATION'${NC}"
        echo -e "${YELLOW}Please check:${NC}"
        echo -e "  - Organization name is correct"
        echo -e "  - You have access to the organization"
        echo -e "  - Organization has public repositories (or use --include-private with token)"
        exit 1
    fi
    
    # Apply repository filter if specified
    if [ -n "$REPO_FILTER" ]; then
        repos=$(echo "$repos" | grep -i "$REPO_FILTER" || echo "")
        if [ -z "$repos" ]; then
            echo -e "${RED}Error: No repositories match filter '$REPO_FILTER'${NC}"
            exit 1
        fi
    fi
    
    local repo_count=$(echo "$repos" | wc -l | tr -d ' ')
    echo -e "${GREEN}✅ Found $repo_count repositories to analyze${NC}"
    echo "$repos"
}

# Function to analyze commits for a repository using GitHub API
analyze_repo_commits() {
    local repo_name="$1"
    local local_commits_file="$2"
    local local_lines_file="$3"
    
    echo -e "  ${CYAN}📊 Analyzing commits...${NC}"
    
    # Get the default branch first
    local default_branch
    default_branch=$(gh api "repos/$ORGANIZATION/$repo_name" --jq '.default_branch' 2>/dev/null || echo "main")
    
    # Get commits since the specified date (URL encode the since parameter)
    local encoded_since=$(printf '%s' "$SINCE_DATE" | sed 's/+/%2B/g' | sed 's/:/%3A/g')
    local commits_data
    commits_data=$(gh api "repos/$ORGANIZATION/$repo_name/commits?sha=$default_branch&since=$encoded_since&per_page=100" \
        --jq '.[] | {author: .author.login, commit_author: .commit.author.name, sha: .sha}' 2>/dev/null || echo "")
    
    if [ -z "$commits_data" ]; then
        echo -e "  ${YELLOW}⚠️  No commits found in the last $DAYS_BACK days${NC}"
        return 1
    fi
    
    # Count commits by author
    local commit_count=0
    while IFS= read -r commit_json; do
        if [ -n "$commit_json" ]; then
            local author=$(echo "$commit_json" | jq -r '.author // .commit_author // "unknown"' 2>/dev/null)
            
            # Skip bot accounts and unknown authors
            if [ "$author" = "null" ] || [ "$author" = "unknown" ] || [[ "$author" =~ \[bot\]$ ]] || [ "$author" = "semantic-release-bot" ]; then
                continue
            fi
            
            # Add to local counts
            local existing_count=$(grep "^$author:" "$local_commits_file" 2>/dev/null | cut -d: -f2 || echo "0")
            local new_count=$((existing_count + 1))
            grep -v "^$author:" "$local_commits_file" > "$local_commits_file.tmp" 2>/dev/null || true
            echo "$author:$new_count" >> "$local_commits_file.tmp"
            mv "$local_commits_file.tmp" "$local_commits_file"
            
            # Add to global counts
            add_commits_for_author "$author" 1
            commit_count=$((commit_count + 1))
        fi
    done <<< "$commits_data"
    
    echo -e "  ${GREEN}✅ Found $commit_count commits${NC}"
    return 0
}

# Function to analyze line changes from recent commits
analyze_commit_line_changes() {
    local repo_name="$1"
    local local_lines_file="$2"
    
    echo -e "  ${CYAN}📈 Analyzing line changes from commits...${NC}"
    
    # Get the default branch first
    local default_branch
    default_branch=$(gh api "repos/$ORGANIZATION/$repo_name" --jq '.default_branch' 2>/dev/null || echo "main")
    
    # Get recent commits with their SHAs
    local encoded_since=$(printf '%s' "$SINCE_DATE" | sed 's/+/%2B/g' | sed 's/:/%3A/g')
    local commit_shas
    commit_shas=$(gh api "repos/$ORGANIZATION/$repo_name/commits?sha=$default_branch&since=$encoded_since&per_page=50" \
        --jq '.[] | {author: (.author.login // .commit.author.name), sha: .sha}' 2>/dev/null || echo "")
    
    if [ -z "$commit_shas" ]; then
        echo -e "  ${YELLOW}⚠️  No recent commits to analyze${NC}"
        return 1
    fi
    
    local commits_analyzed=0
    while IFS= read -r commit_json; do
        if [ -n "$commit_json" ]; then
            local author=$(echo "$commit_json" | jq -r '.author // "unknown"' 2>/dev/null)
            local sha=$(echo "$commit_json" | jq -r '.sha // ""' 2>/dev/null)
            
            # Skip bot accounts and unknown authors
            if [ "$author" = "null" ] || [ "$author" = "unknown" ] || [[ "$author" =~ \[bot\]$ ]] || [ "$author" = "semantic-release-bot" ]; then
                continue
            fi
            
            if [ -n "$sha" ]; then
                check_rate_limit
                # Get commit details to analyze line changes
                local commit_stats
                commit_stats=$(gh api "repos/$ORGANIZATION/$repo_name/commits/$sha" \
                    --jq '.stats | {additions: .additions, deletions: .deletions}' 2>/dev/null || echo "")
                
                if [ -n "$commit_stats" ]; then
                    local additions=$(echo "$commit_stats" | jq -r '.additions // 0' 2>/dev/null)
                    local deletions=$(echo "$commit_stats" | jq -r '.deletions // 0' 2>/dev/null)
                    local total_lines=$((additions + deletions))
                    
                    if [ "$total_lines" -gt 0 ]; then
                        # Add to local counts
                        local existing_lines=$(grep "^$author:" "$local_lines_file" 2>/dev/null | cut -d: -f2 || echo "0")
                        local new_lines=$((existing_lines + total_lines))
                        grep -v "^$author:" "$local_lines_file" > "$local_lines_file.tmp" 2>/dev/null || true
                        echo "$author:$new_lines" >> "$local_lines_file.tmp"
                        mv "$local_lines_file.tmp" "$local_lines_file"
                        
                        # Add to global counts
                        add_lines_for_author "$author" "$total_lines"
                    fi
                fi
                
                commits_analyzed=$((commits_analyzed + 1))
                # Limit to prevent too many API calls for large repos
                if [ "$commits_analyzed" -ge 15 ]; then
                    break
                fi
            fi
        fi
    done <<< "$commit_shas"
    
    echo -e "  ${GREEN}✅ Analyzed line changes from $commits_analyzed commits${NC}"
    return 0
}

# Function to analyze PR reviews using GitHub API
analyze_repo_reviews() {
    local repo_name="$1"
    local local_reviews_file="$2"
    
    echo -e "  ${CYAN}🔍 Analyzing PR reviews...${NC}"
    
    # Get merged/closed PRs from the specified time period (limit to recent PRs)
    local prs_data
    prs_data=$(gh api "repos/$ORGANIZATION/$repo_name/pulls?state=closed&per_page=20&sort=updated&direction=desc" \
        --jq '.[] | select(.merged_at != null) | .number' 2>/dev/null || echo "")
    
    if [ -z "$prs_data" ]; then
        echo -e "  ${YELLOW}⚠️  No merged PRs found in the last $DAYS_BACK days${NC}"
        return 1
    fi
    
    local reviews_found=0
    local since_epoch
    since_epoch=$(date -d "$SINCE_DATE" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%S" "${SINCE_DATE%+*}" +%s 2>/dev/null)
    
    # Process each PR to get reviews
    while IFS= read -r pr_number; do
        if [ -n "$pr_number" ]; then
            check_rate_limit
            
            # Get reviews for this PR
            local reviews_data
            reviews_data=$(gh api "repos/$ORGANIZATION/$repo_name/pulls/$pr_number/reviews" \
                --jq '.[] | select(.submitted_at != null) | {reviewer: .user.login, submitted_at: .submitted_at, user_type: .user.type}' 2>/dev/null || echo "")
            
            if [ -n "$reviews_data" ]; then
                while IFS= read -r review_json; do
                    if [ -n "$review_json" ]; then
                        local reviewer=$(echo "$review_json" | jq -r '.reviewer // "unknown"' 2>/dev/null)
                        local submitted_at=$(echo "$review_json" | jq -r '.submitted_at // ""' 2>/dev/null)
                        local user_type=$(echo "$review_json" | jq -r '.user_type // ""' 2>/dev/null)
                        
                        # Skip if essential data is missing or it's a bot
                        if [ -z "$reviewer" ] || [ -z "$submitted_at" ] || [ "$reviewer" = "null" ] || [ "$reviewer" = "unknown" ]; then
                            continue
                        fi
                        
                        # Skip bot accounts
                        if [ "$user_type" = "Bot" ] || [[ "$reviewer" =~ \[bot\]$ ]] || [ "$reviewer" = "semantic-release-bot" ]; then
                            continue
                        fi
                        
                        # Check if review is from the specified time period
                        local review_epoch
                        review_epoch=$(date -d "${submitted_at%Z}" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%S" "${submitted_at%Z}" +%s 2>/dev/null || echo "0")
                        
                        if [ "$review_epoch" -gt "$since_epoch" ]; then
                            # Add to local counts
                            local existing_count=$(grep "^$reviewer:" "$local_reviews_file" 2>/dev/null | cut -d: -f2 || echo "0")
                            local new_count=$((existing_count + 1))
                            grep -v "^$reviewer:" "$local_reviews_file" > "$local_reviews_file.tmp" 2>/dev/null || true
                            echo "$reviewer:$new_count" >> "$local_reviews_file.tmp"
                            mv "$local_reviews_file.tmp" "$local_reviews_file"
                            
                            # Add to global counts
                            add_reviews_for_author "$reviewer" 1
                            reviews_found=$((reviews_found + 1))
                        fi
                    fi
                done <<< "$reviews_data"
            fi
        fi
    done <<< "$prs_data"
    
    echo -e "  ${GREEN}✅ Found $reviews_found reviews${NC}"
    return 0
}

# Main function to analyze a single repository
analyze_single_repository() {
    local repo_name="$1"
    
    echo -e "${YELLOW}Analyzing repository: $ORGANIZATION/$repo_name${NC}"
    
    # Create local tracking files for this repository
    local local_commits_file="$TEMP_DIR/local_commits_$repo_name.txt"
    local local_lines_file="$TEMP_DIR/local_lines_$repo_name.txt"
    local local_reviews_file="$TEMP_DIR/local_reviews_$repo_name.txt"
    touch "$local_commits_file" "$local_lines_file" "$local_reviews_file"
    
    repos_analyzed=$((repos_analyzed + 1))
    
    # Track if any activity was found
    local has_activity=false
    
    # Analyze commits
    check_rate_limit
    if analyze_repo_commits "$repo_name" "$local_commits_file" "$local_lines_file"; then
        has_activity=true
    fi
    
    # Analyze line changes from commits
    check_rate_limit
    if analyze_commit_line_changes "$repo_name" "$local_lines_file"; then
        has_activity=true
    fi
    
    # Analyze PR reviews
    check_rate_limit
    if analyze_repo_reviews "$repo_name" "$local_reviews_file"; then
        has_activity=true
    fi
    
    if [ "$has_activity" = true ]; then
        repos_with_activity=$((repos_with_activity + 1))
        
        # Find top contributors for this repository
        local top_commits_author=""
        local top_commits_count=0
        if [ -s "$local_commits_file" ]; then
            while IFS=: read -r author count; do
                if [ "$count" -gt "$top_commits_count" ]; then
                    top_commits_count="$count"
                    top_commits_author="$author"
                fi
            done < "$local_commits_file"
        fi
        
        local top_lines_author=""
        local top_lines_count=0
        if [ -s "$local_lines_file" ]; then
            while IFS=: read -r author count; do
                if [ "$count" -gt "$top_lines_count" ]; then
                    top_lines_count="$count"
                    top_lines_author="$author"
                fi
            done < "$local_lines_file"
        fi
        
        local top_reviews_author=""
        local top_reviews_count=0
        if [ -s "$local_reviews_file" ]; then
            while IFS=: read -r author count; do
                if [ "$count" -gt "$top_reviews_count" ]; then
                    top_reviews_count="$count"
                    top_reviews_author="$author"
                fi
            done < "$local_reviews_file"
        fi
        
        # Store repository results
        if [ -n "$top_commits_author" ]; then
            echo "$repo_name:$top_commits_author ($top_commits_count commits)" >> "$REPO_COMMITS_FILE"
        fi
        if [ -n "$top_lines_author" ]; then
            echo "$repo_name:$top_lines_author ($top_lines_count lines)" >> "$REPO_LINES_FILE"
        fi
        if [ -n "$top_reviews_author" ]; then
            echo "$repo_name:$top_reviews_author ($top_reviews_count reviews)" >> "$REPO_REVIEWS_FILE"
        fi
        
        # Display repository summary
        echo -e "  ${GREEN}📊 Top by commits: ${top_commits_author:-"N/A"} (${top_commits_count} commits)${NC}"
        echo -e "  ${GREEN}📊 Top by lines: ${top_lines_author:-"N/A"} (${top_lines_count} lines changed)${NC}"
        echo -e "  ${GREEN}📊 Top by reviews: ${top_reviews_author:-"N/A"} (${top_reviews_count} PR reviews)${NC}"
    else
        echo -e "  ${GREEN}✅ No activity in the last $DAYS_BACK days${NC}"
    fi
    
    echo ""
}

# Main execution starts here
echo -e "${BLUE}🚀 Starting GitHub Organization Contributor Analysis${NC}"
echo ""

# Get all repositories from the organization
repositories=$(get_organization_repos)

if [ -z "$repositories" ]; then
    echo -e "${RED}No repositories found to analyze${NC}"
    exit 1
fi

# Analyze each repository
echo -e "${BLUE}📊 Beginning repository analysis...${NC}"
echo ""

while IFS= read -r repo_name; do
    if [ -n "$repo_name" ]; then
        analyze_single_repository "$repo_name"
    fi
done <<< "$repositories"

# Generate final report
echo ""
echo -e "${BLUE}📈 GITHUB ORGANIZATION CONTRIBUTOR ANALYSIS REPORT${NC}"
echo -e "${BLUE}===================================================${NC}"
echo ""

if [ "$repos_with_activity" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  No repositories with activity found in the last $DAYS_BACK days${NC}"
    echo -e "${BLUE}📊 SUMMARY:${NC}"
    echo -e "  ${YELLOW}Organization: $ORGANIZATION${NC}"
    echo -e "  ${YELLOW}Total repositories analyzed: $repos_analyzed${NC}"
    echo -e "  ${YELLOW}Repositories with activity: $repos_with_activity${NC}"
    echo -e "  ${YELLOW}Time period: Last $DAYS_BACK days${NC}"
    exit 0
fi

# Find overall top contributors
overall_top_commits_author=""
overall_top_commits_count=0
if [ -s "$COMMITS_FILE" ]; then
    while IFS=: read -r author count; do
        if [ "$count" -gt "$overall_top_commits_count" ]; then
            overall_top_commits_count="$count"
            overall_top_commits_author="$author"
        fi
    done < "$COMMITS_FILE"
fi

overall_top_lines_author=""
overall_top_lines_count=0
if [ -s "$LINES_FILE" ]; then
    while IFS=: read -r author count; do
        if [ "$count" -gt "$overall_top_lines_count" ]; then
            overall_top_lines_count="$count"
            overall_top_lines_author="$author"
        fi
    done < "$LINES_FILE"
fi

overall_top_reviews_author=""
overall_top_reviews_count=0
if [ -s "$REVIEWS_FILE" ]; then
    while IFS=: read -r author count; do
        if [ "$count" -gt "$overall_top_reviews_count" ]; then
            overall_top_reviews_count="$count"
            overall_top_reviews_author="$author"
        fi
    done < "$REVIEWS_FILE"
fi

# Display overall winners
echo -e "${PURPLE}🏆 OVERALL WINNERS (Organization: $ORGANIZATION):${NC}"
if [ -n "$overall_top_commits_author" ]; then
    echo -e "  ${RED}👑 Most commits: $overall_top_commits_author ($overall_top_commits_count commits)${NC}"
fi
if [ -n "$overall_top_lines_author" ]; then
    echo -e "  ${RED}👑 Most lines changed: $overall_top_lines_author ($overall_top_lines_count lines)${NC}"
fi
if [ -n "$overall_top_reviews_author" ]; then
    echo -e "  ${RED}👑 Most PR reviews: $overall_top_reviews_author ($overall_top_reviews_count reviews)${NC}"
fi
echo ""

# Display per-repository results
echo -e "${PURPLE}📋 PER-REPOSITORY RESULTS:${NC}"
echo ""

if [ -s "$REPO_COMMITS_FILE" ]; then
    echo -e "${CYAN}Top Contributors by Commits:${NC}"
    while IFS=: read -r repo contributor_info; do
        echo -e "  ${GREEN}$repo: $contributor_info${NC}"
    done < "$REPO_COMMITS_FILE"
    echo ""
fi

if [ -s "$REPO_LINES_FILE" ]; then
    echo -e "${CYAN}Top Contributors by Lines Changed:${NC}"
    while IFS=: read -r repo contributor_info; do
        echo -e "  ${GREEN}$repo: $contributor_info${NC}"
    done < "$REPO_LINES_FILE"
    echo ""
fi

if [ -s "$REPO_REVIEWS_FILE" ]; then
    echo -e "${CYAN}Top Contributors by PR Reviews:${NC}"
    while IFS=: read -r repo contributor_info; do
        echo -e "  ${GREEN}$repo: $contributor_info${NC}"
    done < "$REPO_REVIEWS_FILE"
    echo ""
fi

# Count unique contributors
unique_contributors=0
if [ -s "$COMMITS_FILE" ]; then
    unique_contributors=$(cut -d: -f1 "$COMMITS_FILE" | sort -u | wc -l | tr -d ' ')
fi

# Display summary
echo -e "${BLUE}📊 SUMMARY:${NC}"
echo -e "  ${YELLOW}Organization: $ORGANIZATION${NC}"
echo -e "  ${YELLOW}Time period: Last $DAYS_BACK days${NC}"
echo -e "  ${YELLOW}Total repositories analyzed: $repos_analyzed${NC}"
echo -e "  ${YELLOW}Repositories with activity: $repos_with_activity${NC}"
echo -e "  ${YELLOW}Total unique contributors: $unique_contributors${NC}"
if [ -n "$REPO_FILTER" ]; then
    echo -e "  ${YELLOW}Repository filter applied: $REPO_FILTER${NC}"
fi
echo -e "  ${YELLOW}Include private repositories: $INCLUDE_PRIVATE${NC}"

echo ""
echo -e "${GREEN}✅ Analysis complete!${NC}"