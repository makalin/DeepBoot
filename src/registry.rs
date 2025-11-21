use crate::models::{StartupEntry, StartupSource};
use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::{RegKey, HKEY};

pub struct RegistryScanner;

impl RegistryScanner {
    pub fn scan_all() -> Result<Vec<StartupEntry>> {
        let mut entries = Vec::new();

        // HKCU\Software\Microsoft\Windows\CurrentVersion\Run
        entries.extend(Self::scan_run_key(HKEY_CURRENT_USER, StartupSource::RegistryRun)?);

        // HKCU\Software\Microsoft\Windows\CurrentVersion\RunOnce
        entries.extend(Self::scan_run_key(HKEY_CURRENT_USER, StartupSource::RegistryRunOnce)?);

        // HKLM\Software\Microsoft\Windows\CurrentVersion\Run
        entries.extend(Self::scan_run_key(HKEY_LOCAL_MACHINE, StartupSource::RegistryRun)?);

        // HKLM\Software\Microsoft\Windows\CurrentVersion\RunOnce
        entries.extend(Self::scan_run_key(HKEY_LOCAL_MACHINE, StartupSource::RegistryRunOnce)?);

        // HKLM\Software\Microsoft\Windows\CurrentVersion\RunServices
        entries.extend(Self::scan_run_services()?);

        // HKLM\Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Run
        entries.extend(Self::scan_wow6432_node()?);

        Ok(entries)
    }

    fn scan_run_key(hkey: HKEY, source: StartupSource) -> Result<Vec<StartupEntry>> {
        let mut entries = Vec::new();
        let base_path = match hkey {
            HKEY_CURRENT_USER => "Software\\Microsoft\\Windows\\CurrentVersion",
            HKEY_LOCAL_MACHINE => "Software\\Microsoft\\Windows\\CurrentVersion",
            _ => return Ok(entries),
        };

        let subkey_name = match source {
            StartupSource::RegistryRun => "Run",
            StartupSource::RegistryRunOnce => "RunOnce",
            _ => return Ok(entries),
        };

        let hkey_root = match hkey {
            HKEY_CURRENT_USER => RegKey::predef(HKEY_CURRENT_USER),
            HKEY_LOCAL_MACHINE => RegKey::predef(HKEY_LOCAL_MACHINE),
            _ => return Ok(entries),
        };

        if let Ok(subkey) = hkey_root.open_subkey(base_path) {
            if let Ok(run_key) = subkey.open_subkey(subkey_name) {
                for (name, value) in run_key.enum_values().flatten() {
                    let command = value.to_string();
                    entries.push(StartupEntry::new(
                        name,
                        command,
                        source.clone(),
                        true,
                    ));
                }
            }
        }

        Ok(entries)
    }

    fn scan_run_services() -> Result<Vec<StartupEntry>> {
        let mut entries = Vec::new();
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = "Software\\Microsoft\\Windows\\CurrentVersion\\RunServices";

        if let Ok(run_services) = hklm.open_subkey(path) {
            for (name, value) in run_services.enum_values().flatten() {
                let command = value.to_string();
                entries.push(StartupEntry::new(
                    name,
                    command,
                    StartupSource::RegistryRunServices,
                    true,
                ));
            }
        }

        Ok(entries)
    }

    fn scan_wow6432_node() -> Result<Vec<StartupEntry>> {
        let mut entries = Vec::new();
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = "Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Run";

        if let Ok(wow_key) = hklm.open_subkey(path) {
            for (name, value) in wow_key.enum_values().flatten() {
                let command = value.to_string();
                entries.push(StartupEntry::new(
                    name,
                    command,
                    StartupSource::RegistryWow6432Node,
                    true,
                ));
            }
        }

        Ok(entries)
    }

    pub fn disable_entry(entry: &StartupEntry) -> Result<()> {
        let (hkey, base_path, subkey_name) = match entry.source {
            StartupSource::RegistryRun | StartupSource::RegistryRunOnce => {
                // Try HKCU first, then HKLM
                if Self::entry_exists_in_hkey(HKEY_CURRENT_USER, &entry.source, &entry.name)? {
                    (
                        RegKey::predef(HKEY_CURRENT_USER),
                        "Software\\Microsoft\\Windows\\CurrentVersion",
                        match entry.source {
                            StartupSource::RegistryRun => "Run",
                            StartupSource::RegistryRunOnce => "RunOnce",
                            _ => return Ok(()),
                        },
                    )
                } else {
                    (
                        RegKey::predef(HKEY_LOCAL_MACHINE),
                        "Software\\Microsoft\\Windows\\CurrentVersion",
                        match entry.source {
                            StartupSource::RegistryRun => "Run",
                            StartupSource::RegistryRunOnce => "RunOnce",
                            _ => return Ok(()),
                        },
                    )
                }
            }
            StartupSource::RegistryRunServices => (
                RegKey::predef(HKEY_LOCAL_MACHINE),
                "Software\\Microsoft\\Windows\\CurrentVersion",
                "RunServices",
            ),
            StartupSource::RegistryWow6432Node => (
                RegKey::predef(HKEY_LOCAL_MACHINE),
                "Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion",
                "Run",
            ),
            _ => return Ok(()),
        };

        let base = hkey
            .open_subkey_with_flags(base_path, KEY_WRITE)
            .context("Failed to open registry key for writing")?;
        let mut run_key = base
            .open_subkey_with_flags(subkey_name, KEY_WRITE)
            .context("Failed to open Run subkey")?;

        // Disable by deleting the value (we can't rename in winreg 0.52)
        // The entry will be removed, which effectively disables it
        run_key.delete_value(&entry.name).context("Failed to disable entry")
    }

    pub fn remove_entry(entry: &StartupEntry) -> Result<()> {
        let (hkey, base_path, subkey_name) = match entry.source {
            StartupSource::RegistryRun | StartupSource::RegistryRunOnce => {
                if Self::entry_exists_in_hkey(HKEY_CURRENT_USER, &entry.source, &entry.name)? {
                    (
                        RegKey::predef(HKEY_CURRENT_USER),
                        "Software\\Microsoft\\Windows\\CurrentVersion",
                        match entry.source {
                            StartupSource::RegistryRun => "Run",
                            StartupSource::RegistryRunOnce => "RunOnce",
                            _ => return Ok(()),
                        },
                    )
                } else {
                    (
                        RegKey::predef(HKEY_LOCAL_MACHINE),
                        "Software\\Microsoft\\Windows\\CurrentVersion",
                        match entry.source {
                            StartupSource::RegistryRun => "Run",
                            StartupSource::RegistryRunOnce => "RunOnce",
                            _ => return Ok(()),
                        },
                    )
                }
            }
            StartupSource::RegistryRunServices => (
                RegKey::predef(HKEY_LOCAL_MACHINE),
                "Software\\Microsoft\\Windows\\CurrentVersion",
                "RunServices",
            ),
            StartupSource::RegistryWow6432Node => (
                RegKey::predef(HKEY_LOCAL_MACHINE),
                "Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion",
                "Run",
            ),
            _ => return Ok(()),
        };

        let base = hkey
            .open_subkey_with_flags(base_path, KEY_WRITE)
            .context("Failed to open registry key for writing")?;
        let mut run_key = base
            .open_subkey_with_flags(subkey_name, KEY_WRITE)
            .context("Failed to open Run subkey")?;

        run_key.delete_value(&entry.name).context("Failed to remove entry")
    }

    fn entry_exists_in_hkey(
        hkey: HKEY,
        source: &StartupSource,
        name: &str,
    ) -> Result<bool> {
        let hkey_root = match hkey {
            HKEY_CURRENT_USER => RegKey::predef(HKEY_CURRENT_USER),
            HKEY_LOCAL_MACHINE => RegKey::predef(HKEY_LOCAL_MACHINE),
            _ => return Ok(false),
        };

        let base_path = "Software\\Microsoft\\Windows\\CurrentVersion";
        let subkey_name = match source {
            StartupSource::RegistryRun => "Run",
            StartupSource::RegistryRunOnce => "RunOnce",
            _ => return Ok(false),
        };

        if let Ok(subkey) = hkey_root.open_subkey(base_path) {
            if let Ok(run_key) = subkey.open_subkey(subkey_name) {
                return Ok(run_key.get_value::<String, _>(name).is_ok());
            }
        }

        Ok(false)
    }
}

