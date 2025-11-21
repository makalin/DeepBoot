use crate::models::StartupEntry;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScanStatistics {
    pub total_entries: usize,
    pub enabled_count: usize,
    pub disabled_count: usize,
    pub by_source: HashMap<String, usize>,
    pub by_status: HashMap<String, usize>,
}

impl ScanStatistics {
    pub fn from_entries(entries: &[StartupEntry]) -> Self {
        let mut by_source = HashMap::new();
        let mut by_status = HashMap::new();
        let mut enabled_count = 0;
        let mut disabled_count = 0;

        for entry in entries {
            // Count by source
            let source_str = entry.source.to_string();
            *by_source.entry(source_str).or_insert(0) += 1;

            // Count by status
            if entry.enabled {
                enabled_count += 1;
            } else {
                disabled_count += 1;
            }
        }

        by_status.insert("Enabled".to_string(), enabled_count);
        by_status.insert("Disabled".to_string(), disabled_count);

        Self {
            total_entries: entries.len(),
            enabled_count,
            disabled_count,
            by_source,
            by_status,
        }
    }

    pub fn get_summary(&self) -> String {
        let mut summary = format!("Total Entries: {}\n", self.total_entries);
        summary.push_str(&format!("  Enabled: {} ({:.1}%)\n", 
            self.enabled_count,
            if self.total_entries > 0 {
                (self.enabled_count as f64 / self.total_entries as f64) * 100.0
            } else {
                0.0
            }
        ));
        summary.push_str(&format!("  Disabled: {} ({:.1}%)\n",
            self.disabled_count,
            if self.total_entries > 0 {
                (self.disabled_count as f64 / self.total_entries as f64) * 100.0
            } else {
                0.0
            }
        ));
        summary.push_str("\nBy Source:\n");
        for (source, count) in &self.by_source {
            summary.push_str(&format!("  {}: {} ({:.1}%)\n",
                source,
                count,
                if self.total_entries > 0 {
                    (*count as f64 / self.total_entries as f64) * 100.0
                } else {
                    0.0
                }
            ));
        }
        summary
    }
}

