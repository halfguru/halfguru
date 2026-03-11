mod config;
mod github;
mod svg;
mod theme;

use config::load_config;
use github::GithubClient;
use std::fs;
use svg::Stats;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config("config/profile.toml")?;

    let client = GithubClient::new(&config.github_user_agent)?;
    let username = &config.github_username;

    let loc = client.total_loc(username).await?;
    let stats = Stats {
        repos: client.owned_repo_count(username).await?,
        stars: client.star_count(username).await?,
        followers: client.follower_count(username).await?,
        commits_total: client.commit_count(username).await?,
        contributed_repos: client.contributed_repos(username).await?,
        loc_add: loc.additions as i64,
        loc_del: loc.deletions as i64,
        loc_total: (loc.additions as i64) - (loc.deletions as i64),
    };

    let svg_dark = svg::generate_svg(&stats, &config, svg::OutputMode::Dark);
    let svg_light = svg::generate_svg(&stats, &config, svg::OutputMode::Light);

    fs::write("dark_mode.svg", svg_dark)?;
    fs::write("light_mode.svg", svg_light)?;

    println!("Generated dark_mode.svg and light_mode.svg successfully.");

    Ok(())
}
