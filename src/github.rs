//! GitHub GraphQL API client.
//!
//! Fetches user stats (repos, stars, commits, LOC) with retry/backoff handling.

use anyhow::{Context, Result};
use futures::future::join_all;
use reqwest::Client;
use reqwest::header::RETRY_AFTER;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

const MAX_RETRIES: usize = 4;
const INITIAL_BACKOFF_MS: u64 = 250;
const DEFAULT_RETRY_AFTER_SECS: u64 = 2;
const REQUEST_TIMEOUT_SECS: u64 = 30;
const REPOS_PAGE_SIZE: usize = 100;

#[derive(Deserialize)]
struct CountObj {
    #[serde(rename = "totalCount")]
    total_count: u64,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
}

#[derive(Deserialize)]
struct UserData<T> {
    user: Option<T>,
}

#[derive(Deserialize)]
struct RepositoryData<T> {
    repository: Option<T>,
}

#[derive(Clone)]
pub struct GithubClient {
    token: Arc<String>,
    user_agent: Arc<String>,
    http: Arc<Client>,
}

#[derive(Debug, Default)]
pub struct LocStats {
    pub additions: u64,
    pub deletions: u64,
    pub commits: u64,
}

pub struct Stats {
    pub repos: u32,
    pub stars: u32,
    pub followers: u32,
    pub commits_total: u32,
    pub contributed_repos: u32,
    pub loc_add: i64,
    pub loc_del: i64,
    pub loc_total: i64,
}

impl Stats {
    pub async fn fetch(client: &GithubClient, username: &str) -> Result<Self> {
        let loc = client.total_loc(username).await?;
        Ok(Self {
            repos: client.owned_repo_count(username).await?,
            stars: client.star_count(username).await?,
            followers: client.follower_count(username).await?,
            commits_total: client.commit_count(username).await?,
            contributed_repos: client.contributed_repos(username).await?,
            loc_add: loc.additions as i64,
            loc_del: loc.deletions as i64,
            loc_total: (loc.additions as i64) - (loc.deletions as i64),
        })
    }
}

impl GithubClient {
    pub fn new(user_agent: &str) -> Result<Self> {
        let token =
            std::env::var("ACCESS_TOKEN").context("ACCESS_TOKEN environment variable not set")?;
        let http = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self {
            token: Arc::new(token),
            user_agent: Arc::new(user_agent.to_string()),
            http: Arc::new(http),
        })
    }

    async fn graphql(&self, query: &str) -> Result<Value> {
        let mut attempt = 0usize;

        loop {
            attempt += 1;

            let req = self
                .http
                .post("https://api.github.com/graphql")
                .bearer_auth(&*self.token)
                .header("User-Agent", &*self.user_agent)
                .json(&serde_json::json!({ "query": query }));

            let resp = req
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Network error sending GraphQL request: {e}"))?;

            let status = resp.status();
            let headers = resp.headers().clone();

            let json: Value = resp
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse JSON from GitHub: {e}"))?;

            if let Some(errors) = json.get("errors") {
                return Err(anyhow::anyhow!("GraphQL reported errors: {errors:#}"));
            }

            if status.is_success() {
                return Ok(json);
            }

            if status.as_u16() == 429 {
                if attempt >= MAX_RETRIES {
                    return Err(anyhow::anyhow!(
                        "GitHub API returned 429 (rate-limited) and retries exhausted"
                    ));
                }
                let wait_secs = headers
                    .get(RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_RETRY_AFTER_SECS);
                sleep(Duration::from_secs(wait_secs)).await;
                continue;
            }

            if status.is_server_error() && attempt < MAX_RETRIES {
                let backoff =
                    Duration::from_millis(INITIAL_BACKOFF_MS.saturating_mul(1 << (attempt - 1)));
                sleep(backoff).await;
                continue;
            }

            return Err(anyhow::anyhow!(
                "GitHub API returned HTTP {}: {json:#}",
                status.as_u16()
            ));
        }
    }

    pub async fn owned_repo_count(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"{{
                user(login: "{username}") {{
                    repositories(ownerAffiliations: OWNER) {{ totalCount }}
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct Repositories {
            repositories: CountObj,
        }

        let json = self.graphql(&query).await?;
        let parsed: GraphQLResponse<UserData<Repositories>> = serde_json::from_value(json)
            .context("Failed to deserialize owned_repo_count response")?;

        let count = parsed
            .data
            .and_then(|d| d.user)
            .map(|r| r.repositories.total_count)
            .unwrap_or(0);

        Ok(count as u32)
    }

    pub async fn list_owned_repos(&self, username: &str) -> Result<Vec<String>> {
        let query = format!(
            r#"{{
                user(login: "{username}") {{
                    repositories(ownerAffiliations: OWNER, first: {REPOS_PAGE_SIZE}) {{
                        nodes {{ name }}
                    }}
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct Repositories {
            repositories: RepoNodes,
        }

        #[derive(Deserialize)]
        struct RepoNodes {
            nodes: Option<Vec<RepoNameNode>>,
        }

        #[derive(Deserialize)]
        struct RepoNameNode {
            name: String,
        }

        let json = self.graphql(&query).await?;
        let parsed: GraphQLResponse<UserData<Repositories>> = serde_json::from_value(json)
            .context("Failed to deserialize list_owned_repos response")?;

        let repos = parsed
            .data
            .and_then(|d| d.user)
            .and_then(|u| u.repositories.nodes)
            .map(|nodes| nodes.into_iter().map(|n| n.name).collect())
            .unwrap_or_default();

        Ok(repos)
    }

    pub async fn follower_count(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"{{
                user(login: "{username}") {{
                    followers {{ totalCount }}
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct Followers {
            followers: CountObj,
        }

        let json = self.graphql(&query).await?;
        let parsed: GraphQLResponse<UserData<Followers>> = serde_json::from_value(json)
            .context("Failed to deserialize follower_count response")?;

        let count = parsed
            .data
            .and_then(|d| d.user)
            .map(|u| u.followers.total_count)
            .unwrap_or(0);

        Ok(count as u32)
    }

    pub async fn contributed_repos(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"{{
                user(login: "{username}") {{
                    repositories(
                        first: 1,
                        ownerAffiliations: [OWNER, COLLABORATOR, ORGANIZATION_MEMBER]
                    ) {{ totalCount }}
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct ContribRepos {
            repositories: CountObj,
        }

        let json = self.graphql(&query).await?;
        let parsed: GraphQLResponse<UserData<ContribRepos>> = serde_json::from_value(json)
            .context("Failed to deserialize contributed_repos response")?;

        let total = parsed
            .data
            .and_then(|d| d.user)
            .map(|u| u.repositories.total_count)
            .unwrap_or(0);

        Ok(total as u32)
    }

    pub async fn commit_count(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"{{
                user(login: "{username}") {{
                    contributionsCollection {{
                        totalCommitContributions
                    }}
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct Contributions {
            #[serde(rename = "contributionsCollection")]
            contributions_collection: ContribCollection,
        }

        #[derive(Deserialize)]
        struct ContribCollection {
            #[serde(rename = "totalCommitContributions")]
            total_commit_contributions: u64,
        }

        let json = self.graphql(&query).await?;
        let parsed: GraphQLResponse<UserData<Contributions>> =
            serde_json::from_value(json).context("Failed to deserialize commit_count response")?;

        let commits = parsed
            .data
            .and_then(|d| d.user)
            .map(|c| c.contributions_collection.total_commit_contributions)
            .unwrap_or(0);

        Ok(commits as u32)
    }

    pub async fn star_count(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"{{
                user(login: "{username}") {{
                    repositories(ownerAffiliations: OWNER, first: {REPOS_PAGE_SIZE}) {{
                        nodes {{ stargazers {{ totalCount }} }}
                    }}
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct StarNodes {
            repositories: RepoNodes,
        }

        #[derive(Deserialize)]
        struct RepoNodes {
            nodes: Option<Vec<StarNode>>,
        }

        #[derive(Deserialize)]
        struct StarNode {
            stargazers: CountObj,
        }

        let json = self.graphql(&query).await?;
        let parsed: GraphQLResponse<UserData<StarNodes>> =
            serde_json::from_value(json).context("Failed to deserialize star_count response")?;

        let mut total = 0u64;
        if let Some(data) = parsed.data
            && let Some(user) = data.user
            && let Some(nodes) = user.repositories.nodes
        {
            for n in nodes {
                total += n.stargazers.total_count;
            }
        }

        Ok(total as u32)
    }

    pub async fn repo_loc(&self, username: &str, repo: &str) -> Result<LocStats> {
        #[derive(Deserialize)]
        struct RepoWrapper {
            #[serde(rename = "defaultBranchRef")]
            default_branch_ref: Option<DefaultBranchRef>,
        }

        #[derive(Deserialize)]
        struct DefaultBranchRef {
            target: Option<TargetCommit>,
        }

        #[derive(Deserialize)]
        struct TargetCommit {
            history: Option<CommitHistoryPage>,
        }

        #[derive(Deserialize)]
        struct CommitHistoryPage {
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
            nodes: Option<Vec<HistoryNode>>,
        }

        #[derive(Deserialize)]
        struct PageInfo {
            #[serde(rename = "hasNextPage")]
            has_next_page: bool,
            #[serde(rename = "endCursor")]
            end_cursor: Option<String>,
        }

        #[derive(Deserialize)]
        struct HistoryNode {
            additions: Option<u64>,
            deletions: Option<u64>,
            author: Option<CommitAuthor>,
        }

        #[derive(Deserialize)]
        struct CommitAuthor {
            user: Option<UserLogin>,
        }

        #[derive(Deserialize)]
        struct UserLogin {
            login: Option<String>,
        }

        let mut stats = LocStats::default();
        let mut cursor: Option<String> = None;

        loop {
            let after = cursor
                .as_ref()
                .map(|c| format!("\"{c}\""))
                .unwrap_or_else(|| "null".to_string());

            let query = format!(
                r#"{{
                    repository(name: "{repo}", owner: "{username}") {{
                        defaultBranchRef {{
                            target {{
                                ... on Commit {{
                                    history(first: 100, after: {after}) {{
                                        pageInfo {{ hasNextPage endCursor }}
                                        nodes {{
                                            additions
                                            deletions
                                            author {{ user {{ login }} }}
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}"#,
                after = after
            );

            let json = self.graphql(&query).await?;
            let parsed: GraphQLResponse<RepositoryData<RepoWrapper>> = serde_json::from_value(json)
                .context("Failed to deserialize repo_loc (history) response")?;

            let history = parsed
                .data
                .and_then(|d| d.repository)
                .and_then(|r| r.default_branch_ref)
                .and_then(|d| d.target)
                .and_then(|t| t.history);

            let history = match history {
                Some(h) => h,
                None => return Ok(stats),
            };

            if let Some(nodes) = history.nodes {
                for node in nodes {
                    let author_login = node
                        .author
                        .and_then(|a| a.user)
                        .and_then(|u| u.login)
                        .unwrap_or_default();

                    if author_login == username {
                        stats.commits = stats.commits.saturating_add(1);
                        stats.additions =
                            stats.additions.saturating_add(node.additions.unwrap_or(0));
                        stats.deletions =
                            stats.deletions.saturating_add(node.deletions.unwrap_or(0));
                    }
                }
            }

            if !history.page_info.has_next_page {
                break;
            }

            cursor = history.page_info.end_cursor;
        }

        Ok(stats)
    }

    pub async fn total_loc(&self, username: &str) -> Result<LocStats> {
        let repos = self.list_owned_repos(username).await?;
        let futures: Vec<_> = repos
            .iter()
            .map(|repo| self.repo_loc(username, repo))
            .collect();

        let results = join_all(futures).await;

        let mut total = LocStats::default();
        for (repo, result) in repos.iter().zip(results) {
            match result {
                Ok(loc) => {
                    total.additions = total.additions.saturating_add(loc.additions);
                    total.deletions = total.deletions.saturating_add(loc.deletions);
                    total.commits = total.commits.saturating_add(loc.commits);
                }
                Err(e) => {
                    eprintln!("Warning: failed to get LOC for repo {}: {e:#}", repo);
                }
            }
        }

        Ok(total)
    }
}
