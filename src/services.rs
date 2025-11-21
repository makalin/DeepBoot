use crate::models::{StartupEntry, StartupSource};
use anyhow::{Context, Result};
use serde_json;
use std::process::Command;

// Common Windows system services that should be filtered out
const SYSTEM_SERVICES: &[&str] = &[
    "AudioSrv", "BITS", "Browser", "CryptSvc", "DcomLaunch", "Dhcp", "Dnscache",
    "EventLog", "EventSystem", "FontCache", "gpsvc", "hidserv", "IKEEXT", "iphlpsvc",
    "KeyIso", "LanmanServer", "LanmanWorkstation", "lmhosts", "MMCSS", "MpsSvc",
    "MSiSCSI", "Netlogon", "netprofm", "NlaSvc", "nsi", "p2pimsvc", "p2psvc",
    "PlugPlay", "PolicyAgent", "ProfSvc", "RasMan", "RemoteAccess", "RpcEptMapper",
    "RpcSs", "SamSs", "Schedule", "SENS", "SessionEnv", "Spooler", "SysMain",
    "Themes", "TrkWks", "TrustedInstaller", "UmRdpService", "VaultSvc", "VSS",
    "W32Time", "Wcmsvc", "WcsPlugInService", "WdiServiceHost", "Winmgmt", "WinRM",
    "WlanSvc", "wmiApSrv", "WMPNetworkSvc", "WSearch", "wuauserv", "WudfSvc",
    "wscsvc", "WbioSrvc", "WinHttpAutoProxySvc", "WerSvc", "WebClient", "WaaSMedicSvc",
    "UsoSvc", "UevAgentService", "TabletInputService", "SysMain", "StiSvc", "SstpSvc",
    "SSDPSRV", "Spooler", "SstpSvc", "ShellHWDetection", "SCardSvr", "SCPolicySvc",
    "SCardSvr", "SCPolicySvc", "RpcLocator", "RemoteRegistry", "RemoteAccess",
    "RasAuto", "QWAVE", "PNRPsvc", "PNRPsvc", "PcaSvc", "PcaSvc", "PcaSvc",
    "PcaSvc", "PcaSvc", "PcaSvc", "PcaSvc", "PcaSvc", "PcaSvc", "PcaSvc",
];

pub struct ServicesScanner;

impl ServicesScanner {
    pub fn scan() -> Result<Vec<StartupEntry>> {
        // Use PowerShell to get services more reliably
        let ps_command = r#"
            Get-WmiObject Win32_Service | Where-Object {
                $_.StartMode -eq 'Auto' -and 
                $_.PathName -ne $null -and
                $_.SystemService -eq $false
            } | Select-Object Name, DisplayName, PathName | ConvertTo-Json
        "#;

        let output = Command::new("powershell")
            .args(&["-Command", ps_command])
            .output()
            .context("Failed to execute PowerShell command. Make sure you're on Windows.")?;

        if !output.status.success() {
            // Fallback to sc query if PowerShell fails
            return Self::scan_with_sc();
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse JSON output
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
            let mut entries = Vec::new();
            
            let services: Vec<&serde_json::Value> = if json.is_array() {
                json.as_array().unwrap().iter().collect()
            } else if json.is_object() {
                // Single service
                vec![&json]
            } else {
                return Ok(Vec::new());
            };

            for service in services {
                if let (Some(name), Some(display_name), Some(path_name)) = (
                    service.get("Name").and_then(|v| v.as_str()),
                    service.get("DisplayName").and_then(|v| v.as_str()),
                    service.get("PathName").and_then(|v| v.as_str()),
                ) {
                    if !Self::is_system_service(name) {
                        entries.push(
                            StartupEntry::new(
                                display_name.to_string(),
                                path_name.to_string(),
                                StartupSource::Service,
                                true,
                            )
                            .with_description(format!("Service: {}", name)),
                        );
                    }
                }
            }

            Ok(entries)
        } else {
            // Fallback to sc query
            Self::scan_with_sc()
        }
    }

    fn scan_with_sc() -> Result<Vec<StartupEntry>> {
        let output = Command::new("sc")
            .args(&["query"])
            .output()
            .context("Failed to execute 'sc query' command")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();
        let mut current_service = None;

        for line in output_str.lines() {
            let line = line.trim();
            
            if line.starts_with("SERVICE_NAME:") {
                if let Some(name) = line.strip_prefix("SERVICE_NAME:") {
                    current_service = Some(name.trim().to_string());
                }
            } else if let Some(service_name) = &current_service {
                if line.starts_with("DISPLAY_NAME:") {
                    let display_name = line
                        .strip_prefix("DISPLAY_NAME:")
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    // Check if it's a third-party service (not a Windows system service)
                    if !Self::is_system_service(service_name) {
                        // Get service binary path
                        let binary_path = Self::get_service_binary_path(service_name)
                            .unwrap_or_else(|_| "Unknown".to_string());

                        let enabled = Self::is_service_enabled(service_name);

                        entries.push(
                            StartupEntry::new(
                                display_name,
                                binary_path,
                                StartupSource::Service,
                                enabled,
                            )
                            .with_description(format!("Service: {}", service_name)),
                        );
                    }
                }
            }
        }

        Ok(entries)
    }

    fn is_system_service(service_name: &str) -> bool {
        SYSTEM_SERVICES.contains(&service_name)
    }

    fn get_service_binary_path(service_name: &str) -> Result<String> {
        let output = Command::new("sc")
            .args(&["qc", service_name])
            .output()
            .context("Failed to query service configuration")?;

        if !output.status.success() {
            return Ok("Unknown".to_string());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.trim().starts_with("BINARY_PATH_NAME") {
                if let Some(path) = line.split(':').nth(1) {
                    return Ok(path.trim().to_string());
                }
            }
        }

        Ok("Unknown".to_string())
    }

    fn is_service_enabled(service_name: &str) -> bool {
        let output = Command::new("sc")
            .args(&["qc", service_name])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Check if service is set to auto-start
            output_str.contains("AUTO_START") || output_str.contains("DEMAND_START")
        } else {
            false
        }
    }

    pub fn disable_service(entry: &StartupEntry) -> Result<()> {
        // Extract service name from description
        let service_name = entry
            .description
            .as_ref()
            .and_then(|d| d.strip_prefix("Service: "))
            .ok_or_else(|| anyhow::anyhow!("Invalid service entry"))?;

        Command::new("sc")
            .args(&["config", service_name, "start=", "disabled"])
            .output()
            .context("Failed to disable service")?;

        Ok(())
    }

    pub fn remove_service(_entry: &StartupEntry) -> Result<()> {
        // Note: Removing services is dangerous and typically requires
        // stopping the service first and then deleting it.
        // This is a placeholder - actual implementation would need admin rights
        // and proper service deletion logic.
        anyhow::bail!("Service removal is not implemented for safety reasons")
    }
}

