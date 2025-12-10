use anyhow::{Context, Result};
use reqwest::Client;
use reqwest::header::RETRY_AFTER;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize)]
struct CountObj {
    #[serde(rename = "totalCount")]
    total_count: u64,
}

#[derive(Clone)]
pub struct GithubClient {
    token: Arc<String>,
    http: Arc<Client>,
}

#[derive(Debug, Default)]
pub struct LocStats {
    pub additions: u64,
    pub deletions: u64,
    pub commits: u64,
}

impl GithubClient {
    /// Create a GitHub GraphQL client using ACCESS_TOKEN env variable.
    pub fn new() -> Result<Self> {
        let token =
            std::env::var("ACCESS_TOKEN").context("ACCESS_TOKEN environment variable not set")?;
        Ok(Self {
            token: Arc::new(token),
            http: Arc::new(Client::new()),
        })
    }

    /// Low-level GraphQL request with basic retry/backoff and `errors` checking.
    async fn graphql(&self, query: &str) -> Result<Value> {
        // Simple retry/backoff policy
        const MAX_RETRIES: usize = 4;
        let mut attempt = 0usize;

        loop {
            attempt += 1;

            let req = self
                .http
                .post("https://api.github.com/graphql")
                .bearer_auth(&*self.token)
                .header("User-Agent", "halfguru-stats")
                .json(&serde_json::json!({ "query": query }));

            let resp = req
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Network error sending GraphQL request: {e}"))?;

            let status = resp.status();
            let headers = resp.headers().clone();

            // Parse JSON (even for non-2xx to capture error payloads)
            let json: Value = resp
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse JSON from GitHub: {e}"))?;

            // If GraphQL returned an `errors` field, treat it as an error.
            if let Some(errors) = json.get("errors") {
                return Err(anyhow::anyhow!("GraphQL reported errors: {errors:#}"));
            }

            // Retry on rate-limit / server errors. If status is success, return.
            if status.is_success() {
                return Ok(json);
            }

            // If rate limited, honor Retry-After header when present
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
                    .unwrap_or(2);
                sleep(Duration::from_secs(wait_secs)).await;
                continue;
            }

            // Retry on 5xx server errors
            if status.is_server_error() && attempt < MAX_RETRIES {
                let backoff = Duration::from_millis(250u64.saturating_mul(1 << (attempt - 1)));
                sleep(backoff).await;
                continue;
            }

            return Err(anyhow::anyhow!(
                "GitHub API returned HTTP {}: {json:#}",
                status.as_u16()
            ));
        }
    }

    /// Fetch number of repositories owned by `username`.
    pub async fn owned_repo_count(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"
            {{
                user(login: "{username}") {{
                    repositories(ownerAffiliations: OWNER) {{
                        totalCount
                    }}
                }}
            }}
        "#
        );

        #[derive(Deserialize)]
        struct ReposWrapper {
            data: Option<UserWrapper>,
        }
        #[derive(Deserialize)]
        struct UserWrapper {
            user: Option<RepositoriesCount>,
        }
        #[derive(Deserialize)]
        struct RepositoriesCount {
            repositories: CountObj,
        }

        let json = self.graphql(&query).await?;
        let parsed: ReposWrapper = serde_json::from_value(json)
            .context("Failed to deserialize owned_repo_count response")?;

        let count = parsed
            .data
            .and_then(|d| d.user)
            .map(|r| r.repositories.total_count)
            .unwrap_or(0);

        Ok(count as u32)
    }

    /// List owned repositories (first page; we keep first: 100 to match original behavior).
    pub async fn list_owned_repos(&self, username: &str) -> Result<Vec<String>> {
        let query = format!(
            r#"
        {{
            user(login: "{username}") {{
                repositories(ownerAffiliations: OWNER, first: 100) {{
                    nodes {{
                        name
                    }}
                }}
            }}
        }}
        "#
        );

        #[derive(Deserialize)]
        struct RepoListResponse {
            data: Option<RepoListData>,
        }
        #[derive(Deserialize)]
        struct RepoListData {
            user: Option<RepoListUser>,
        }
        #[derive(Deserialize)]
        struct RepoListUser {
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
        let parsed: RepoListResponse = serde_json::from_value(json)
            .context("Failed to deserialize list_owned_repos response")?;

        let mut out = Vec::new();
        if let Some(data) = parsed.data {
            if let Some(user) = data.user {
                if let Some(nodes) = user.repositories.nodes {
                    for n in nodes {
                        out.push(n.name);
                    }
                }
            }
        }

        Ok(out)
    }

    /// Follower count
    pub async fn follower_count(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"
            {{
                user(login: "{username}") {{
                    followers {{
                        totalCount
                    }}
                }}
            }}
        "#
        );

        #[derive(Deserialize)]
        struct FollowersResponse {
            data: Option<FollowersData>,
        }
        #[derive(Deserialize)]
        struct FollowersData {
            user: Option<FollowersUser>,
        }
        #[derive(Deserialize)]
        struct FollowersUser {
            followers: CountObj,
        }

        let json = self.graphql(&query).await?;
        let parsed: FollowersResponse = serde_json::from_value(json)
            .context("Failed to deserialize follower_count response")?;

        let count = parsed
            .data
            .and_then(|d| d.user)
            .map(|u| u.followers.total_count)
            .unwrap_or(0);

        Ok(count as u32)
    }

    /// Count of repositories the user has contributed to (totalCount).
    pub async fn contributed_repos(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"
            query {{
                user(login: "{username}") {{
                    repositories(
                        first: 1,
                        ownerAffiliations: [OWNER, COLLABORATOR, ORGANIZATION_MEMBER]
                    ) {{
                        totalCount
                    }}
                }}
            }}
            "#
        );

        #[derive(Deserialize)]
        struct ContribResponse {
            data: Option<ContribData>,
        }
        #[derive(Deserialize)]
        struct ContribData {
            user: Option<ContribUser>,
        }
        #[derive(Deserialize)]
        struct ContribUser {
            repositories: CountObj,
        }

        let json = self.graphql(&query).await?;
        let parsed: ContribResponse = serde_json::from_value(json)
            .context("Failed to deserialize contributed_repos response")?;

        let total = parsed
            .data
            .and_then(|d| d.user)
            .map(|u| u.repositories.total_count)
            .unwrap_or(0);

        Ok(total as u32)
    }

    /// Contribution commit count (year-to-date total commit contributions)
    pub async fn commit_count(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"
            {{
                user(login: "{username}") {{
                    contributionsCollection {{
                        totalCommitContributions
                    }}
                }}
            }}
        "#
        );

        #[derive(Deserialize)]
        struct CommitsResponse {
            data: Option<CommitsData>,
        }
        #[derive(Deserialize)]
        struct CommitsData {
            user: Option<CommitsUser>,
        }
        #[derive(Deserialize)]
        struct CommitsUser {
            #[serde(rename = "contributionsCollection")]
            contributions_collection: Option<ContribCollection>,
        }
        #[derive(Deserialize)]
        struct ContribCollection {
            #[serde(rename = "totalCommitContributions")]
            total_commit_contributions: u64,
        }

        let json = self.graphql(&query).await?;
        let parsed: CommitsResponse =
            serde_json::from_value(json).context("Failed to deserialize commit_count response")?;

        let commits = parsed
            .data
            .and_then(|d| d.user)
            .and_then(|u| u.contributions_collection)
            .map(|c| c.total_commit_contributions)
            .unwrap_or(0);

        Ok(commits as u32)
    }

    /// Sum stargazers for first 100 owned repos (same behavior as original).
    pub async fn star_count(&self, username: &str) -> Result<u32> {
        let query = format!(
            r#"
        {{
            user(login: "{username}") {{
                repositories(ownerAffiliations: OWNER, first: 100) {{
                    nodes {{
                        stargazers {{
                            totalCount
                        }}
                    }}
                }}
            }}
        }}
        "#
        );

        #[derive(Deserialize)]
        struct StarResponse {
            data: Option<StarData>,
        }
        #[derive(Deserialize)]
        struct StarData {
            user: Option<StarUser>,
        }
        #[derive(Deserialize)]
        struct StarUser {
            repositories: StarRepos,
        }
        #[derive(Deserialize)]
        struct StarRepos {
            nodes: Option<Vec<StarNode>>,
        }
        #[derive(Deserialize)]
        struct StarNode {
            stargazers: CountObj,
        }

        let json = self.graphql(&query).await?;
        let parsed: StarResponse =
            serde_json::from_value(json).context("Failed to deserialize star_count response")?;

        let mut total = 0u64;
        if let Some(data) = parsed.data {
            if let Some(user) = data.user {
                if let Some(nodes) = user.repositories.nodes {
                    for n in nodes {
                        total += n.stargazers.total_count;
                    }
                }
            }
        }

        Ok(total as u32)
    }

    /// Get LOC for a single repository by iterating commit history pages.
    pub async fn repo_loc(&self, username: &str, repo: &str) -> Result<LocStats> {
        #[derive(Deserialize)]
        struct RepoHistoryResponse {
            data: Option<RepoHistoryData>,
        }
        #[derive(Deserialize)]
        struct RepoHistoryData {
            repository: Option<RepositoryWrapper>,
        }
        #[derive(Deserialize)]
        struct RepositoryWrapper {
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
                r#"
                {{
                    repository(name: "{repo}", owner: "{username}") {{
                        defaultBranchRef {{
                            target {{
                                ... on Commit {{
                                    history(first: 100, after: {after}) {{
                                        pageInfo {{
                                            hasNextPage
                                            endCursor
                                        }}
                                        nodes {{
                                            additions
                                            deletions
                                            author {{
                                                user {{
                                                    login
                                                }}
                                            }}
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
                "#,
                after = after
            );

            let json = self.graphql(&query).await?;
            let parsed: RepoHistoryResponse = serde_json::from_value(json)
                .context("Failed to deserialize repo_loc (history) response")?;

            let history = parsed
                .data
                .and_then(|d| d.repository)
                .and_then(|r| r.default_branch_ref)
                .and_then(|db| db.target)
                .and_then(|t| t.history)
                .ok_or_else(|| {
                    anyhow::anyhow!("Missing commit history for {}/{}", username, repo)
                })?;

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

    /// Aggregate LOC across owned repos (sequential; consider making concurrent).
    pub async fn total_loc(&self, username: &str) -> Result<LocStats> {
        let repos = self.list_owned_repos(username).await?;
        let mut total = LocStats::default();

        for r in repos {
            match self.repo_loc(username, &r).await {
                Ok(loc) => {
                    total.additions = total.additions.saturating_add(loc.additions);
                    total.deletions = total.deletions.saturating_add(loc.deletions);
                    total.commits = total.commits.saturating_add(loc.commits);
                }
                Err(e) => {
                    // don't fail the whole run for one repo; log and continue.
                    eprintln!("Warning: failed to get LOC for repo {}: {e:#}", r);
                }
            }
        }

        Ok(total)
    }
}
