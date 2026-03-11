mod config;
mod github;
mod svg;
mod theme;

use anyhow::Result;
use config::load_config;
use github::GithubClient;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config("config/profile.toml")?;

    let client = GithubClient::new(&config.github_user_agent)?;
    let stats = github::Stats::fetch(&client, &config.github_username).await?;

    let svg_dark = svg::generate_svg(&stats, &config, svg::OutputMode::Dark);
    let svg_light = svg::generate_svg(&stats, &config, svg::OutputMode::Light);

    fs::write("dark_mode.svg", svg_dark)?;
    fs::write("light_mode.svg", svg_light)?;

    Ok(())
}
