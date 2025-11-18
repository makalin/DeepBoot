mod actions;
mod backup;
mod batch;
mod config;
mod export;
mod filter;
mod logger;
mod models;
mod registry;
mod services;
mod stats;
mod task_scheduler;
mod tui;
mod whitelist;

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use models::{Action, StartupEntry};
use ratatui::prelude::*;
use std::io;
use tui::App;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    // Load configuration
    let config_manager = config::ConfigManager::new()?;
    let config = config_manager.get();

    println!("DeepBoot Pro - Advanced Startup Manager");
    println!("Scanning startup entries...");
    println!("This may take a few moments...\n");

    // Initialize logger
    let action_logger = logger::ActionLogger::new()?;

    // Scan all startup locations
    let mut all_entries = Vec::new();

    // Scan Task Scheduler
    println!("Scanning Task Scheduler...");
    match task_scheduler::TaskSchedulerScanner::scan() {
        Ok(entries) => {
            println!("  Found {} entries", entries.len());
            let _ = action_logger.log_scan("Task Scheduler", entries.len());
            all_entries.extend(entries);
        }
        Err(e) => {
            eprintln!("  Warning: Failed to scan Task Scheduler: {}", e);
        }
    }

    // Scan Registry
    println!("Scanning Registry...");
    match registry::RegistryScanner::scan_all() {
        Ok(entries) => {
            println!("  Found {} entries", entries.len());
            let _ = action_logger.log_scan("Registry", entries.len());
            all_entries.extend(entries);
        }
        Err(e) => {
            eprintln!("  Warning: Failed to scan Registry: {}", e);
        }
    }

    // Scan Services
    println!("Scanning Services...");
    match services::ServicesScanner::scan() {
        Ok(entries) => {
            println!("  Found {} entries", entries.len());
            let _ = action_logger.log_scan("Services", entries.len());
            all_entries.extend(entries);
        }
        Err(e) => {
            eprintln!("  Warning: Failed to scan Services: {}", e);
        }
    }

    // Apply whitelist filter if configured
    let whitelist_manager = whitelist::WhitelistManager::new()?;
    if !config.show_whitelisted {
        let original_count = all_entries.len();
        all_entries = whitelist_manager.filter_whitelisted(all_entries);
        if original_count != all_entries.len() {
            println!("  Filtered {} whitelisted entries", original_count - all_entries.len());
        }
    }

    // Create backup if configured
    if config.auto_backup {
        let backup_manager = backup::BackupManager::new()?;
        match backup_manager.create_backup(&all_entries) {
            Ok(path) => {
                println!("  Backup created: {:?}", path);
            }
            Err(e) => {
                eprintln!("  Warning: Failed to create backup: {}", e);
            }
        }
    }

    // Generate statistics
    let stats = stats::ScanStatistics::from_entries(&all_entries);
    println!("\n{}", stats.get_summary());

    println!("\nTotal entries found: {}", all_entries.len());
    
    if all_entries.is_empty() {
        println!("No startup entries found. Exiting...");
        return Ok(());
    }
    
    println!("Press Enter to continue to the TUI...");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create app with all the managers
    // Note: We need to pass config_manager as mutable, but App will handle it
    let mut app = App::new(
        all_entries,
        whitelist_manager,
        action_logger,
        config_manager,
    );

    // Run the TUI
    let result = tui::run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    result
}


