# Org Pulse

This is a github org statistics app for small to medium size engineering 
orgs that provides weekly insights on who contributed what where.

- Get weekly top contributors (by commit and LoC) and reviewers per repo 
  in a given github org

## Config

### Settings

a `config.toml` is generated in the directory the application is ran in.

### Github Token

Use `gh` to set github token to use

```bash
$ export GITHUB_TOKEN=$(gh auth token)
```

## Who is this for

Members of a growing engineering org or startup

- The ICs that are suffering from imposter syndrom
  - Take a look at you contributions in the repos you work in relative 
    to the rest of the org
- The managers that can't keep up with various projects
  - Look back on the week and see exactly what progress was made where
- The leaders trying to wrangle focus and direction
  - Where are efforts going

## Why this application

Developer productivity isn't messurable by a single statistic. However, 
by looking at repository statistics you can begin to see patterns and
at the very least outliers in large swings in performance.

Org Pulse provides a light solution to give you answers and insights 
without having to commit to an entirely new methodology or process.

Org Pulse is designed for you to outgrow eventually, but until you reach
organizational maturity, we're here to help you identify strugling projects,
high velocity inatives, and celebrate efforts.

Its simple. A weekly summary containing

- top 3 contributors by commit on main
- top 3 contributors by LoC
- top 3 PR reviewers by number of reviews

per repo you subscribe to.

## Pricing

Its simple. As many repos as you want per org with an All-repository read access token.

If there's a user already paying for the org you want to subscribe to,
then you're good to go.