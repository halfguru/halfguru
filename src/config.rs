use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub name: String,
    pub birthday: String,
    #[serde(rename = "github_username")]
    pub github_username: String,
    #[serde(rename = "github_user_agent")]
    pub github_user_agent: String,
    #[serde(rename = "system")]
    pub system: SystemConfig,
    #[serde(rename = "languages")]
    pub languages: LanguagesConfig,
    #[serde(rename = "hobbies")]
    pub hobbies: HobbiesConfig,
    #[serde(rename = "contact")]
    pub contact: ContactConfig,
    #[serde(rename = "headers")]
    pub headers: HeadersConfig,
}

#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    pub os: String,
    pub host: String,
    pub kernel: String,
    pub ide: String,
}

#[derive(Debug, Deserialize)]
pub struct LanguagesConfig {
    pub programming: String,
    pub computer: String,
    pub real: String,
}

#[derive(Debug, Deserialize)]
pub struct HobbiesConfig {
    pub software: String,
    pub hardware: String,
}

#[derive(Debug, Deserialize)]
pub struct ContactConfig {
    #[serde(rename = "personal_email")]
    pub personal_email: String,
    #[serde(rename = "work_email")]
    pub work_email: String,
    pub linkedin: String,
}

#[derive(Debug, Deserialize)]
pub struct HeadersConfig {
    pub contact: String,
    #[serde(rename = "github_stats")]
    pub github_stats: String,
}

pub fn load_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
