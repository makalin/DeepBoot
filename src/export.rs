use crate::models::StartupEntry;
use anyhow::{Context, Result};
use chrono::Local;
use std::fs::File;
use std::path::PathBuf;

pub struct Exporter;

impl Exporter {
    pub fn export_json(entries: &[StartupEntry], path: Option<PathBuf>) -> Result<PathBuf> {
        let file_path = path.unwrap_or_else(|| {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            PathBuf::from(format!("deepboot_export_{}.json", timestamp))
        });

        let file = File::create(&file_path)
            .with_context(|| format!("Failed to create file: {:?}", file_path))?;

        serde_json::to_writer_pretty(file, entries)
            .context("Failed to write JSON data")?;

        Ok(file_path)
    }

    pub fn export_csv(entries: &[StartupEntry], path: Option<PathBuf>) -> Result<PathBuf> {
        let file_path = path.unwrap_or_else(|| {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            PathBuf::from(format!("deepboot_export_{}.csv", timestamp))
        });

        let mut writer = csv::Writer::from_path(&file_path)
            .with_context(|| format!("Failed to create CSV file: {:?}", file_path))?;

        // Write header
        writer
            .write_record(&["Name", "Command", "Source", "Enabled", "Description"])
            .context("Failed to write CSV header")?;

        // Write entries
        for entry in entries {
            writer
                .write_record(&[
                    &entry.name,
                    &entry.command,
                    &entry.source.to_string(),
                    &entry.enabled.to_string(),
                    entry.description.as_deref().unwrap_or(""),
                ])
                .context("Failed to write CSV record")?;
        }

        writer.flush().context("Failed to flush CSV writer")?;

        Ok(file_path)
    }

    pub fn export_markdown(entries: &[StartupEntry], path: Option<PathBuf>) -> Result<PathBuf> {
        let file_path = path.unwrap_or_else(|| {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            PathBuf::from(format!("deepboot_export_{}.md", timestamp))
        });

        let mut content = String::new();
        content.push_str("# DeepBoot Scan Report\n\n");
        content.push_str(&format!("Generated: {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        content.push_str(&format!("Total Entries: {}\n\n", entries.len()));
        content.push_str("## Startup Entries\n\n");
        content.push_str("| Name | Command | Source | Enabled | Description |\n");
        content.push_str("|------|---------|--------|---------|-------------|\n");

        for entry in entries {
            content.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                entry.name,
                entry.command,
                entry.source,
                if entry.enabled { "Yes" } else { "No" },
                entry.description.as_deref().unwrap_or("")
            ));
        }

        std::fs::write(&file_path, content)
            .with_context(|| format!("Failed to write markdown file: {:?}", file_path))?;

        Ok(file_path)
    }
}

