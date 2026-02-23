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
    #[serde(default = "default_ascii_file")]
    pub ascii_file: String,
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

fn default_ascii_file() -> String {
    "config/ascii.txt".to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config_valid() {
        let toml_content = r#"
name = "Test User"
birthday = "1990-01-01"
github_username = "testuser"
github_user_agent = "test-agent"

[system]
os = "Test OS"
host = "Test Host"
kernel = "Test Kernel"
ide = "Test IDE"

[languages]
programming = "Test Prog"
computer = "Test Comp"
real = "Test Real"

[hobbies]
software = "Test Soft"
hardware = "Test Hard"

[contact]
personal_email = "test@example.com"
work_email = "work@example.com"
linkedin = "testlinkedin"

[headers]
contact = "- Contact"
github_stats = "- Stats"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.name, "Test User");
        assert_eq!(config.birthday, "1990-01-01");
        assert_eq!(config.github_username, "testuser");
        assert_eq!(config.system.os, "Test OS");
        assert_eq!(config.languages.programming, "Test Prog");
    }

    #[test]
    fn test_load_config_missing_required_field() {
        let toml_content = r#"
name = "Test User"
birthday = "1990-01-01"
github_username = "testuser"
github_user_agent = "test-agent"

[system]
os = "Test OS"
host = "Test Host"
kernel = "Test Kernel"
ide = "Test IDE"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_invalid_birthday_format() {
        let toml_content = r#"
name = "Test User"
birthday = "invalid-date"
github_username = "testuser"
github_user_agent = "test-agent"

[system]
os = "Test OS"
host = "Test Host"
kernel = "Test Kernel"
ide = "Test IDE"

[languages]
programming = "Test Prog"
computer = "Test Comp"
real = "Test Real"

[hobbies]
software = "Test Soft"
hardware = "Test Hard"

[contact]
personal_email = "test@example.com"
work_email = "work@example.com"
linkedin = "testlinkedin"

[headers]
contact = "- Contact"
github_stats = "- Stats"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.birthday, "invalid-date");
    }
}
