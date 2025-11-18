use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_backup: bool,
    pub show_whitelisted: bool,
    pub default_sort: String,
    pub log_level: String,
    pub auto_export: Option<String>, // "json", "csv", "markdown", or None
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_backup: true,
            show_whitelisted: false,
            default_sort: "name".to_string(),
            log_level: "info".to_string(),
            auto_export: None,
        }
    }
}

pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
            .join("deepboot");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }

        let config_path = config_dir.join("config.json");

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            serde_json::from_str(&content).unwrap_or_else(|_| AppConfig::default())
        } else {
            let default_config = AppConfig::default();
            let content = serde_json::to_string_pretty(&default_config)
                .context("Failed to serialize default config")?;
            fs::write(&config_path, content)
                .context("Failed to write default config")?;
            default_config
        };

        Ok(Self {
            config,
            config_path,
        })
    }

    pub fn get(&self) -> &AppConfig {
        &self.config
    }

    pub fn get_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;
        fs::write(&self.config_path, content)
            .context("Failed to save config")?;
        Ok(())
    }
}

