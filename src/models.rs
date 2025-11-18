use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StartupSource {
    TaskScheduler,
    RegistryRun,
    RegistryRunOnce,
    RegistryRunServices,
    RegistryWow6432Node,
    Service,
}

impl fmt::Display for StartupSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartupSource::TaskScheduler => write!(f, "Task Scheduler"),
            StartupSource::RegistryRun => write!(f, "Registry (Run)"),
            StartupSource::RegistryRunOnce => write!(f, "Registry (RunOnce)"),
            StartupSource::RegistryRunServices => write!(f, "Registry (RunServices)"),
            StartupSource::RegistryWow6432Node => write!(f, "Registry (WoW6432Node)"),
            StartupSource::Service => write!(f, "Service"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupEntry {
    pub name: String,
    pub command: String,
    pub source: StartupSource,
    pub enabled: bool,
    pub description: Option<String>,
}

impl StartupEntry {
    pub fn new(
        name: String,
        command: String,
        source: StartupSource,
        enabled: bool,
    ) -> Self {
        Self {
            name,
            command,
            source,
            enabled,
            description: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Disable,
    Remove,
    Enable,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Disable => write!(f, "Disable"),
            Action::Remove => write!(f, "Remove"),
            Action::Enable => write!(f, "Enable"),
        }
    }
}

