use reqwest::Client;
use std::error::Error;

pub struct GithubClient {
    token: String,
    http: Client,
}

#[derive(Debug, Default)]
pub struct LocStats {
    pub additions: u64,
    pub deletions: u64,
    pub commits: u64,
}

impl GithubClient {
    /// Create a GitHub GraphQL client using ACCESS_TOKEN env variable.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let token = std::env::var("ACCESS_TOKEN")?;

        Ok(Self {
            token,
            http: Client::new(),
        })
    }

    /// Send a GraphQL query to GitHub API.
    async fn graphql(&self, query: &str) -> Result<serde_json::Value, String> {
        let resp = self
            .http
            .post("https://api.github.com/graphql")
            .bearer_auth(&self.token)
            .header("User-Agent", "halfguru-stats")
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Invalid JSON from GitHub: {e}"))?;

        if !status.is_success() {
            return Err(format!("GitHub API returned HTTP {status}: {json:?}"));
        }

        Ok(json)
    }

    /// Fetch number of repositories owned by USER_NAME
    pub async fn owned_repo_count(&self, username: &str) -> Result<u32, String> {
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

        let json = self.graphql(&query).await?;
        let count = json["data"]["user"]["repositories"]["totalCount"]
            .as_u64()
            .unwrap_or(0) as u32;

        Ok(count)
    }

    pub async fn list_owned_repos(&self, username: &str) -> Result<Vec<String>, String> {
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

        let json = self.graphql(&query).await?;

        let repos = json["data"]["user"]["repositories"]["nodes"]
            .as_array()
            .ok_or("Malformed list_owned_repos response")?;

        let mut out = Vec::new();
        for repo in repos {
            if let Some(name) = repo["name"].as_str() {
                out.push(name.to_string());
            }
        }

        Ok(out)
    }

    pub async fn follower_count(&self, username: &str) -> Result<u32, String> {
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

        let json = self.graphql(&query).await?;

        let followers = json["data"]["user"]["followers"]["totalCount"]
            .as_u64()
            .unwrap_or(0);

        Ok(followers as u32)
    }

    pub async fn contributed_repos(&self, username: &str) -> Result<u32, String> {
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

        let json = self.graphql(&query).await?;

        let total = json["data"]["user"]["repositories"]["totalCount"]
            .as_u64()
            .unwrap_or(0);

        Ok(total as u32)
    }

    pub async fn commit_count(&self, username: &str) -> Result<u32, String> {
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

        let json = self.graphql(&query).await?;

        let commits = json["data"]["user"]["contributionsCollection"]["totalCommitContributions"]
            .as_u64()
            .unwrap_or(0);

        Ok(commits as u32)
    }

    pub async fn star_count(&self, username: &str) -> Result<u32, String> {
        // Sum stargazers over the first 100 owned repos.
        // (We can add pagination later if you ever have >100 repos.)
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

        let json = self.graphql(&query).await?;

        let repos = json["data"]["user"]["repositories"]["nodes"]
            .as_array()
            .ok_or_else(|| "Malformed GitHub star response".to_string())?;

        let mut total = 0u64;
        for repo in repos {
            if let Some(count) = repo["stargazers"]["totalCount"].as_u64() {
                total += count;
            }
        }

        Ok(total as u32)
    }

    pub async fn repo_loc(&self, username: &str, repo: &str) -> Result<LocStats, String> {
        let mut stats = LocStats::default();
        let mut cursor: Option<String> = None;

        loop {
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
                after = cursor
                    .as_ref()
                    .map(|c| format!("\"{c}\""))
                    .unwrap_or("null".to_string())
            );

            let json = self.graphql(&query).await?;

            let history = &json["data"]["repository"]["defaultBranchRef"]["target"]["history"];

            let commits = history["nodes"]
                .as_array()
                .ok_or("Malformed commit history response")?;

            for commit in commits {
                let author = commit["author"]["user"]["login"].as_str().unwrap_or("");

                // only count your own commits
                if author == username {
                    stats.commits += 1;
                    stats.additions += commit["additions"].as_u64().unwrap_or(0);
                    stats.deletions += commit["deletions"].as_u64().unwrap_or(0);
                }
            }

            // pagination
            let has_next = history["pageInfo"]["hasNextPage"]
                .as_bool()
                .unwrap_or(false);
            if !has_next {
                break;
            }

            cursor = history["pageInfo"]["endCursor"]
                .as_str()
                .map(|s| s.to_string());
        }

        Ok(stats)
    }

    pub async fn total_loc(&self, username: &str) -> Result<LocStats, String> {
        let repos = self.list_owned_repos(username).await?;

        let mut total = LocStats::default();

        for r in repos {
            let loc = self.repo_loc(username, &r).await?;
            total.additions += loc.additions;
            total.deletions += loc.deletions;
            total.commits += loc.commits;
        }

        Ok(total)
    }
}
