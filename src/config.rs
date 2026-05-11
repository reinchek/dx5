use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use serde::Deserialize;
use std::fs;
use rocket::serde::Serialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub blog: BlogConfig,
    pub languages: Languages,
    pub home: HomeConfig,
    pub audio: AudioConfig,
    pub theme: ThemeConfig,
    pub admin: Option<AdminConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminConfig {
    pub enabled: bool,
    /// Token Bearer - Overridable via DX5_ADMIN_TOKEN env var at runtime.
    pub token: String,
}

// Languages(HashMap<LangCode, Label>
#[derive(Debug, Deserialize, Clone)]
pub struct Languages(pub HashMap<String, String>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HomeConfig {
    pub label: String,
    pub icon: Option<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct BlogConfig {
    pub title: String,
    pub author: String,
    pub base_url: String,
    pub language: String,
    pub spa_enabled: bool,
    pub debug_enabled: Option<bool>
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioConfig {
    pub enabled: bool,
    pub soundtracks_dir: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeConfig {
    pub templates_dir: String,
    pub start_with_framework: Option<bool>,
    pub framework: Option<String>
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = std::env::var("DX5_CONFIG").unwrap_or_else(|_| "config/dx5.toml".to_string());

        let raw = fs::read_to_string(&path).map_err(|_| {
            ConfigError(format!(
                "Configuration file '{}' not found.\nCreate a dx5.toml file in the project's root.",
                path
            ))
        })?;

        let mut config: Self = toml::from_str(&raw).map_err(|e| ConfigError(format!("Error during dx5.toml file parsing: {}", e)))?;

        // DX5_ADMIN_TOKEN env var overrides admin token from config file
        if let Ok(env_token) = std::env::var("DX5_ADMIN_TOKEN") {
            if let Some(ref mut admin) = config.admin {
                if !env_token.is_empty() {
                    admin.token = env_token;
                }
            }
        }

        Ok(config)
    }
}

#[derive(Debug)]
pub struct ConfigError(pub String);

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ConfigError {}
