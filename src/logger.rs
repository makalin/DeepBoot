use anyhow::{Context, Result};
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone)]
pub struct ActionLogger {
    log_file_path: std::path::PathBuf,
}

impl ActionLogger {
    pub fn new() -> Result<Self> {
        let log_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?
            .join("deepboot")
            .join("logs");

        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir)
                .context("Failed to create log directory")?;
        }

        let log_file_path = log_dir.join(format!("deepboot_{}.log", 
            Local::now().format("%Y%m%d")));

        Ok(Self {
            log_file_path,
        })
    }

    fn write_log(&self, message: &str) -> Result<()> {
        lazy_static::lazy_static! {
            static ref LOG_MUTEX: Mutex<()> = Mutex::new(());
        }
        
        let _guard = LOG_MUTEX.lock().unwrap();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .context("Failed to open log file")?;
        
        file.write_all(message.as_bytes())
            .context("Failed to write to log file")?;
        file.flush().context("Failed to flush log file")?;
        
        Ok(())
    }

    pub fn log_action(&self, action: &str, entry_name: &str, success: bool, error: Option<&str>) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let status = if success { "SUCCESS" } else { "FAILED" };
        
        let mut log_entry = format!(
            "[{}] {} - Entry: '{}' - Status: {}",
            timestamp, action, entry_name, status
        );

        if let Some(err) = error {
            log_entry.push_str(&format!(" - Error: {}", err));
        }

        log_entry.push('\n');
        self.write_log(&log_entry)
    }

    pub fn log_scan(&self, source: &str, count: usize) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!(
            "[{}] SCAN - Source: {} - Found: {} entries\n",
            timestamp, source, count
        );
        self.write_log(&log_entry)
    }

    pub fn log_batch_action(&self, action: &str, count: usize, success_count: usize) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!(
            "[{}] BATCH {} - Total: {} - Successful: {} - Failed: {}\n",
            timestamp, action, count, success_count, count - success_count
        );
        self.write_log(&log_entry)
    }
}


