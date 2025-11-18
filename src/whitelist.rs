use crate::models::StartupEntry;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistConfig {
    pub safe_processes: HashSet<String>,
    pub safe_services: HashSet<String>,
    pub safe_tasks: HashSet<String>,
}

impl Default for WhitelistConfig {
    fn default() -> Self {
        let mut safe_processes = HashSet::new();
        // Add common safe Windows processes
        safe_processes.insert("explorer.exe".to_lowercase());
        safe_processes.insert("winlogon.exe".to_lowercase());
        safe_processes.insert("csrss.exe".to_lowercase());
        safe_processes.insert("services.exe".to_lowercase());
        safe_processes.insert("lsass.exe".to_lowercase());
        safe_processes.insert("svchost.exe".to_lowercase());
        safe_processes.insert("dwm.exe".to_lowercase());
        safe_processes.insert("conhost.exe".to_lowercase());

        Self {
            safe_processes,
            safe_services: HashSet::new(),
            safe_tasks: HashSet::new(),
        }
    }
}

pub struct WhitelistManager {
    config: WhitelistConfig,
    config_path: PathBuf,
}

impl WhitelistManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
            .join("deepboot");

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }

        let config_path = config_dir.join("whitelist.json");

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read whitelist config")?;
            serde_json::from_str(&content).unwrap_or_else(|_| WhitelistConfig::default())
        } else {
            let default_config = WhitelistConfig::default();
            let content = serde_json::to_string_pretty(&default_config)
                .context("Failed to serialize default config")?;
            fs::write(&config_path, content)
                .context("Failed to write default whitelist config")?;
            default_config
        };

        Ok(Self {
            config,
            config_path,
        })
    }

    pub fn is_whitelisted(&self, entry: &StartupEntry) -> bool {
        let name_lower = entry.name.to_lowercase();
        let command_lower = entry.command.to_lowercase();

        // Check process name in command
        if let Some(process_name) = Self::extract_process_name(&command_lower) {
            if self.config.safe_processes.contains(&process_name) {
                return true;
            }
        }

        // Check service name
        if let Some(service_name) = entry.description.as_ref() {
            if let Some(name) = service_name.strip_prefix("Service: ") {
                if self.config.safe_services.contains(&name.to_lowercase()) {
                    return true;
                }
            }
        }

        // Check task name
        if matches!(entry.source, crate::models::StartupSource::TaskScheduler) {
            if self.config.safe_tasks.contains(&name_lower) {
                return true;
            }
        }

        false
    }

    pub fn add_to_whitelist(&mut self, entry: &StartupEntry) -> Result<()> {
        match entry.source {
            crate::models::StartupSource::Service => {
                if let Some(service_name) = entry.description.as_ref() {
                    if let Some(name) = service_name.strip_prefix("Service: ") {
                        self.config.safe_services.insert(name.to_lowercase());
                    }
                }
            }
            crate::models::StartupSource::TaskScheduler => {
                self.config.safe_tasks.insert(entry.name.to_lowercase());
            }
            _ => {
                if let Some(process_name) = Self::extract_process_name(&entry.command.to_lowercase()) {
                    self.config.safe_processes.insert(process_name);
                }
            }
        }

        self.save()
    }

    pub fn remove_from_whitelist(&mut self, entry: &StartupEntry) -> Result<()> {
        match entry.source {
            crate::models::StartupSource::Service => {
                if let Some(service_name) = entry.description.as_ref() {
                    if let Some(name) = service_name.strip_prefix("Service: ") {
                        self.config.safe_services.remove(&name.to_lowercase());
                    }
                }
            }
            crate::models::StartupSource::TaskScheduler => {
                self.config.safe_tasks.remove(&entry.name.to_lowercase());
            }
            _ => {
                if let Some(process_name) = Self::extract_process_name(&entry.command.to_lowercase()) {
                    self.config.safe_processes.remove(&process_name);
                }
            }
        }

        self.save()
    }

    pub fn filter_whitelisted(&self, entries: Vec<StartupEntry>) -> Vec<StartupEntry> {
        entries
            .into_iter()
            .filter(|entry| !self.is_whitelisted(entry))
            .collect()
    }

    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize whitelist config")?;
        fs::write(&self.config_path, content)
            .context("Failed to save whitelist config")?;
        Ok(())
    }

    fn extract_process_name(command: &str) -> Option<String> {
        // Extract executable name from command
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_part) = parts.first() {
            // Remove quotes if present
            let cleaned = first_part.trim_matches('"');
            // Extract just the filename
            if let Some(filename) = std::path::Path::new(cleaned).file_name() {
                return Some(filename.to_string_lossy().to_lowercase());
            }
        }
        None
    }

    pub fn get_config(&self) -> &WhitelistConfig {
        &self.config
    }
}

