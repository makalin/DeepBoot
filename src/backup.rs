use crate::models::StartupEntry;
use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub entry: StartupEntry,
    pub original_path: String,
    pub backup_timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Backup {
    pub timestamp: String,
    pub entries: Vec<BackupEntry>,
}

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        let backup_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?
            .join("deepboot")
            .join("backups");

        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)
                .context("Failed to create backup directory")?;
        }

        Ok(Self { backup_dir })
    }

    pub fn create_backup(&self, entries: &[StartupEntry]) -> Result<PathBuf> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_file = self.backup_dir.join(format!("backup_{}.json", timestamp));

        let backup = Backup {
            timestamp: Local::now().to_rfc3339(),
            entries: entries
                .iter()
                .map(|entry| BackupEntry {
                    entry: entry.clone(),
                    original_path: Self::get_entry_path(entry),
                    backup_timestamp: Local::now().to_rfc3339(),
                })
                .collect(),
        };

        let content = serde_json::to_string_pretty(&backup)
            .context("Failed to serialize backup")?;
        fs::write(&backup_file, content)
            .context("Failed to write backup file")?;

        Ok(backup_file)
    }

    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups: Vec<PathBuf> = fs::read_dir(&self.backup_dir)
            .context("Failed to read backup directory")?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.extension() == Some(std::ffi::OsStr::new("json"))
                        && path.file_name()?.to_string_lossy().starts_with("backup_")
                    {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();

        backups.sort();
        backups.reverse(); // Most recent first
        Ok(backups)
    }

    pub fn load_backup(&self, path: &PathBuf) -> Result<Backup> {
        let content = fs::read_to_string(path)
            .context("Failed to read backup file")?;
        let backup: Backup = serde_json::from_str(&content)
            .context("Failed to parse backup file")?;
        Ok(backup)
    }

    pub fn restore_backup(&self, backup: &Backup) -> Result<()> {
        // This would restore entries from backup
        // Implementation depends on the entry type
        // For now, we'll just log what would be restored
        log::info!("Restoring backup from {}", backup.timestamp);
        log::info!("Entries to restore: {}", backup.entries.len());
        
        // TODO: Implement actual restoration logic
        // This would involve:
        // 1. For registry entries: Write back to registry
        // 2. For task scheduler: Recreate tasks
        // 3. For services: Re-enable services
        
        Ok(())
    }

    pub fn delete_backup(&self, path: &PathBuf) -> Result<()> {
        fs::remove_file(path)
            .context("Failed to delete backup file")?;
        Ok(())
    }

    fn get_entry_path(entry: &StartupEntry) -> String {
        match entry.source {
            crate::models::StartupSource::TaskScheduler => {
                format!("TaskScheduler:{}", entry.name)
            }
            crate::models::StartupSource::RegistryRun => {
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string()
            }
            crate::models::StartupSource::RegistryRunOnce => {
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce".to_string()
            }
            crate::models::StartupSource::RegistryRunServices => {
                "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\RunServices".to_string()
            }
            crate::models::StartupSource::RegistryWow6432Node => {
                "HKLM\\Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Run".to_string()
            }
            crate::models::StartupSource::Service => {
                entry.description.as_deref().unwrap_or("Unknown Service").to_string()
            }
        }
    }
}

