use crate::models::{Action, StartupEntry};
use anyhow::Result;

// Action handlers
pub fn handle_action(entry: &StartupEntry, action: Action) -> Result<()> {
    match action {
        Action::Disable => match entry.source {
            crate::models::StartupSource::TaskScheduler => {
                crate::task_scheduler::TaskSchedulerScanner::disable_task(entry)
            }
            crate::models::StartupSource::RegistryRun
            | crate::models::StartupSource::RegistryRunOnce
            | crate::models::StartupSource::RegistryRunServices
            | crate::models::StartupSource::RegistryWow6432Node => {
                crate::registry::RegistryScanner::disable_entry(entry)
            }
            crate::models::StartupSource::Service => {
                crate::services::ServicesScanner::disable_service(entry)
            }
        },
        Action::Remove => match entry.source {
            crate::models::StartupSource::TaskScheduler => {
                crate::task_scheduler::TaskSchedulerScanner::remove_task(entry)
            }
            crate::models::StartupSource::RegistryRun
            | crate::models::StartupSource::RegistryRunOnce
            | crate::models::StartupSource::RegistryRunServices
            | crate::models::StartupSource::RegistryWow6432Node => {
                crate::registry::RegistryScanner::remove_entry(entry)
            }
            crate::models::StartupSource::Service => {
                crate::services::ServicesScanner::remove_service(entry)
            }
        },
        Action::Enable => {
            // Enable logic would go here
            anyhow::bail!("Enable action not yet implemented")
        }
    }
}

