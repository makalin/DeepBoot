use crate::models::{StartupEntry, StartupSource};
use anyhow::{Context, Result};
use windows::{
    core::*,
    Win32::Foundation::VARIANT_BOOL,
    Win32::System::Com::*,
    Win32::System::TaskScheduler::*,
};
use windows::core::GUID;

// TaskScheduler CLSID: {0F87369F-A4E5-4CFC-BD3E-73E6154572DD}
const CLSID_TASK_SCHEDULER: GUID = GUID::from_u128(0x0F87369F_A4E5_4CFC_BD3E_73E6154572DD);

pub struct TaskSchedulerScanner;

impl TaskSchedulerScanner {
    pub fn scan() -> Result<Vec<StartupEntry>> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .context("Failed to initialize COM")?;

            let entries = {
                let task_service: ITaskService = CoCreateInstance(
                    &CLSID_TASK_SCHEDULER,
                    None,
                    CLSCTX_INPROC_SERVER,
                )
                .context("Failed to create TaskScheduler COM object")?;

                task_service
                    .Connect(
                        None,
                        None,
                        None,
                        None,
                    )
                    .ok()
                    .context("Failed to connect to Task Scheduler")?;

                let root_folder = task_service
                    .GetFolder(&BSTR::from("\\"))
                    .context("Failed to get root folder")?;

                let mut entries = Vec::new();
                Self::scan_folder(&root_folder, &mut entries)?;
                entries
            };

            CoUninitialize();
            Ok(entries)
        }
    }

    unsafe fn scan_folder(folder: &ITaskFolder, entries: &mut Vec<StartupEntry>) -> Result<()> {
        // Get registered tasks
        let registered_tasks = folder
            .GetTasks(TASK_ENUM_HIDDEN.0 as i32)
            .context("Failed to get registered tasks")?;

        let count = registered_tasks.Count().context("Failed to get task count")?;

        for i in 0..count {
            let index_variant = VARIANT::from(i + 1);
            let task = registered_tasks.get_Item(&index_variant).ok();
            if let Some(task) = task {
                if let Ok(entry) = Self::check_task(&task) {
                    entries.push(entry);
                }
            }
        }

        // Recursively scan subfolders
        let subfolders = folder
            .GetFolders(0)
            .context("Failed to get subfolders")?;

        let folder_count = subfolders.Count().context("Failed to get folder count")?;

        for i in 0..folder_count {
            let index_variant = VARIANT::from(i + 1);
            let subfolder = subfolders.get_Item(&index_variant).ok();
            if let Some(subfolder) = subfolder {
                Self::scan_folder(&subfolder, entries)?;
            }
        }

        Ok(())
    }

    unsafe fn check_task(task: &IRegisteredTask) -> Result<StartupEntry> {
        let name = task.Name()?.to_string();
        let enabled = task.Enabled()?.as_bool();

        let definition = task.Definition().context("Failed to get task definition")?;
        let actions = definition.Actions().context("Failed to get actions")?;

        let mut command = String::new();
        let mut description = None;

        // Get description
        if let Ok(reg_info) = definition.RegistrationInfo() {
            let mut desc = BSTR::default();
            if reg_info.Description(&mut desc).is_ok() {
                let desc_str = desc.to_string();
                if !desc_str.is_empty() {
                    description = Some(desc_str);
                }
            }
        }

        // Get trigger information to check if it's a startup trigger
        let triggers = definition.Triggers().context("Failed to get triggers")?;
        let mut trigger_count = 0i32;
        triggers.Count(&mut trigger_count).context("Failed to get trigger count")?;

        let mut is_startup_trigger = false;

        for i in 0..trigger_count {
            if let Ok(trigger) = triggers.get_Item(i + 1) {
                let mut trigger_type = TASK_TRIGGER_TYPE2::default();
                if trigger.Type(&mut trigger_type).is_ok() {
                    // Check for logon trigger (TASK_TRIGGER_LOGON = 9)
                    // or boot trigger (TASK_TRIGGER_BOOT = 8)
                    // or idle trigger (TASK_TRIGGER_IDLE = 6)
                    if trigger_type == TASK_TRIGGER_LOGON
                        || trigger_type == TASK_TRIGGER_BOOT
                        || trigger_type == TASK_TRIGGER_IDLE
                    {
                        is_startup_trigger = true;
                        break;
                    }
                }
            }
        }

        // Get the command from the first action
        let mut action_count = 0i32;
        actions.Count(&mut action_count).context("Failed to get action count")?;
        if action_count > 0 {
            if let Ok(action) = actions.get_Item(1) {
                if let Ok(exec_action) = action.cast::<IExecAction>() {
                    let mut path = BSTR::default();
                    if exec_action.Path(&mut path).is_ok() {
                        command = path.to_string();
                        let mut args = BSTR::default();
                        if exec_action.Arguments(&mut args).is_ok() {
                            let args_str = args.to_string();
                            if !args_str.is_empty() {
                                command.push_str(" ");
                                command.push_str(&args_str);
                            }
                        }
                    }
                }
            }
        }

        // Only include tasks that have startup triggers
        if is_startup_trigger && !command.is_empty() {
            let mut entry = StartupEntry::new(name, command, StartupSource::TaskScheduler, enabled);
            if let Some(desc) = description {
                entry = entry.with_description(desc);
            }
            Ok(entry)
        } else {
            anyhow::bail!("Not a startup task")
        }
    }

    pub fn disable_task(entry: &StartupEntry) -> Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .context("Failed to initialize COM")?;

            {
                let task_service: ITaskService = CoCreateInstance(
                    &CLSID_TASK_SCHEDULER,
                    None,
                    CLSCTX_INPROC_SERVER,
                )
                .context("Failed to create TaskScheduler COM object")?;

                task_service
                    .Connect(
                        None,
                        None,
                        None,
                        None,
                    )
                    .ok()
                    .context("Failed to connect to Task Scheduler")?;

                // Find the task by name
                let root_folder = task_service
                    .GetFolder(&BSTR::from("\\"))
                    .context("Failed to get root folder")?;

                if let Ok((folder, task_path)) = Self::find_task_path(&root_folder, &entry.name) {
                    // Use schtasks command line tool as a reliable way to disable tasks
                    // The COM interface's put_Enabled method is not easily accessible in windows-rs
                    use std::process::Command;
                    let output = Command::new("schtasks")
                        .args(&["/Change", "/TN", &task_path, "/Disable"])
                        .output()
                        .context("Failed to execute schtasks command")?;
                    
                    if !output.status.success() {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        anyhow::bail!("Failed to disable task: {}", error_msg);
                    }
                }
            }

            CoUninitialize();
            Ok(())
        }
    }

    pub fn remove_task(entry: &StartupEntry) -> Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .context("Failed to initialize COM")?;

            {
                let task_service: ITaskService = CoCreateInstance(
                    &CLSID_TASK_SCHEDULER,
                    None,
                    CLSCTX_INPROC_SERVER,
                )
                .context("Failed to create TaskScheduler COM object")?;

                task_service
                    .Connect(
                        None,
                        None,
                        None,
                        None,
                    )
                    .ok()
                    .context("Failed to connect to Task Scheduler")?;

                let root_folder = task_service
                    .GetFolder(&BSTR::from("\\"))
                    .context("Failed to get root folder")?;

                if let Ok((folder, task_name)) = Self::find_task_path(&root_folder, &entry.name) {
                    folder.DeleteTask(&BSTR::from(&task_name), 0).ok();
                }
            }

            CoUninitialize();
            Ok(())
        }
    }

    unsafe fn find_task_by_name(
        folder: &ITaskFolder,
        name: &str,
    ) -> Result<IRegisteredTask> {
        let registered_tasks = folder
            .GetTasks(TASK_ENUM_HIDDEN.0 as i32)
            .context("Failed to get registered tasks")?;

        let count = registered_tasks.Count().context("Failed to get task count")?;

        for i in 0..count {
            let index_variant = VARIANT::from(i + 1);
            let task = registered_tasks.get_Item(&index_variant).ok();
            if let Some(task) = task {
                if let Ok(task_name) = task.Name() {
                    if task_name.to_string() == name {
                        return Ok(task);
                    }
                }
            }
        }

        // Search in subfolders
        let subfolders = folder
            .GetFolders(0)
            .context("Failed to get subfolders")?;

        let folder_count = subfolders.Count().context("Failed to get folder count")?;

        for i in 0..folder_count {
            let index_variant = VARIANT::from(i + 1);
            let subfolder = subfolders.get_Item(&index_variant).ok();
            if let Some(subfolder) = subfolder {
                if let Ok(task) = Self::find_task_by_name(&subfolder, name) {
                    return Ok(task);
                }
            }
        }

        anyhow::bail!("Task not found")
    }

    unsafe fn find_task_path(
        folder: &ITaskFolder,
        name: &str,
    ) -> Result<(ITaskFolder, String)> {
        let registered_tasks = folder
            .GetTasks(TASK_ENUM_HIDDEN.0 as i32)
            .context("Failed to get registered tasks")?;

        let count = registered_tasks.Count().context("Failed to get task count")?;

        for i in 0..count {
            let index_variant = VARIANT::from(i + 1);
            let task = registered_tasks.get_Item(&index_variant).ok();
            if let Some(task) = task {
                if let Ok(task_name) = task.Name() {
                    if task_name.to_string() == name {
                        let folder_path = if let Ok(path) = folder.Path() {
                            path.to_string()
                        } else {
                            "\\".to_string()
                        };
                        let full_path = if folder_path == "\\" {
                            format!("\\{}", name)
                        } else {
                            format!("{}\\{}", folder_path, name)
                        };
                        return Ok((folder.clone(), full_path));
                    }
                }
            }
        }

        // Search in subfolders
        let subfolders = folder
            .GetFolders(0)
            .context("Failed to get subfolders")?;

        let folder_count = subfolders.Count().context("Failed to get folder count")?;

        for i in 0..folder_count {
            let index_variant = VARIANT::from(i + 1);
            let subfolder = subfolders.get_Item(&index_variant).ok();
            if let Some(subfolder) = subfolder {
                if let Ok((found_folder, path)) = Self::find_task_path(&subfolder, name) {
                    return Ok((found_folder, path));
                }
            }
        }

        anyhow::bail!("Task not found")
    }
}

