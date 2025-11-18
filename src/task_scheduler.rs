use crate::models::{StartupEntry, StartupSource};
use anyhow::{Context, Result};
use windows::{
    core::*,
    Win32::System::Com::*,
    Win32::System::TaskScheduler::*,
};

pub struct TaskSchedulerScanner;

impl TaskSchedulerScanner {
    pub fn scan() -> Result<Vec<StartupEntry>> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .context("Failed to initialize COM")?;

            let task_service: ITaskService = CoCreateInstance(
                &TaskScheduler::CLSID_TaskScheduler,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .context("Failed to create TaskScheduler COM object")?;

            task_service
                .Connect(
                    VARIANT::default(),
                    VARIANT::default(),
                    VARIANT::default(),
                    VARIANT::default(),
                )
                .ok()
                .context("Failed to connect to Task Scheduler")?;

            let root_folder = task_service
                .GetFolder(&BSTR::from("\\"))
                .context("Failed to get root folder")?;

            let mut entries = Vec::new();
            Self::scan_folder(&root_folder, &mut entries)?;

            CoUninitialize();
            Ok(entries)
        }
    }

    unsafe fn scan_folder(folder: &ITaskFolder, entries: &mut Vec<StartupEntry>) -> Result<()> {
        // Get registered tasks
        let registered_tasks = folder
            .GetTasks(TASK_ENUM_HIDDEN)
            .context("Failed to get registered tasks")?;

        let count = registered_tasks.Count().context("Failed to get task count")?;

        for i in 0..count {
            let task = registered_tasks.Item(i + 1).ok();
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
            let subfolder = subfolders.Item(i + 1).ok();
            if let Some(subfolder) = subfolder {
                Self::scan_folder(&subfolder, entries)?;
            }
        }

        Ok(())
    }

    unsafe fn check_task(task: &IRegisteredTask) -> Result<StartupEntry> {
        let name = task.Name().to_string();
        let enabled = task.Enabled().as_bool();

        let definition = task.Definition().context("Failed to get task definition")?;
        let actions = definition.Actions().context("Failed to get actions")?;
        let action_count = actions.Count().context("Failed to get action count")?;

        let mut command = String::new();
        let mut description = None;

        // Get description
        if let Ok(reg_info) = definition.RegistrationInfo() {
            if let Ok(desc) = reg_info.Description() {
                let desc_str = desc.to_string();
                if !desc_str.is_empty() {
                    description = Some(desc_str);
                }
            }
        }

        // Get trigger information to check if it's a startup trigger
        let triggers = definition.Triggers().context("Failed to get triggers")?;
        let trigger_count = triggers.Count().context("Failed to get trigger count")?;

        let mut is_startup_trigger = false;

        for i in 0..trigger_count {
            if let Ok(trigger) = triggers.Item(i + 1) {
                let trigger_type = trigger.Type_().ok();
                if let Some(trigger_type) = trigger_type {
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
        if action_count > 0 {
            if let Ok(action) = actions.Item(1) {
                if let Ok(exec_action) = action.cast::<IExecAction>() {
                    if let Ok(path) = exec_action.Path() {
                        command = path.to_string();
                        if let Ok(args) = exec_action.Arguments() {
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

            let task_service: ITaskService = CoCreateInstance(
                &TaskScheduler::CLSID_TaskScheduler,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .context("Failed to create TaskScheduler COM object")?;

            task_service
                .Connect(
                    VARIANT::default(),
                    VARIANT::default(),
                    VARIANT::default(),
                    VARIANT::default(),
                )
                .ok()
                .context("Failed to connect to Task Scheduler")?;

            // Find the task by name
            let root_folder = task_service
                .GetFolder(&BSTR::from("\\"))
                .context("Failed to get root folder")?;

            if let Ok(task) = Self::find_task_by_name(&root_folder, &entry.name) {
                task.Enabled(VARIANT::from(false))
                    .context("Failed to disable task")?;
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

            let task_service: ITaskService = CoCreateInstance(
                &TaskScheduler::CLSID_TaskScheduler,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .context("Failed to create TaskScheduler COM object")?;

            task_service
                .Connect(
                    VARIANT::default(),
                    VARIANT::default(),
                    VARIANT::default(),
                    VARIANT::default(),
                )
                .ok()
                .context("Failed to connect to Task Scheduler")?;

            let root_folder = task_service
                .GetFolder(&BSTR::from("\\"))
                .context("Failed to get root folder")?;

            if let Ok((folder, task_name)) = Self::find_task_path(&root_folder, &entry.name) {
                folder.DeleteTask(&BSTR::from(&task_name), 0).ok();
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
            .GetTasks(TASK_ENUM_HIDDEN)
            .context("Failed to get registered tasks")?;

        let count = registered_tasks.Count().context("Failed to get task count")?;

        for i in 0..count {
            let task = registered_tasks.Item(i + 1).ok();
            if let Some(task) = task {
                if task.Name().to_string() == name {
                    return Ok(task);
                }
            }
        }

        // Search in subfolders
        let subfolders = folder
            .GetFolders(0)
            .context("Failed to get subfolders")?;

        let folder_count = subfolders.Count().context("Failed to get folder count")?;

        for i in 0..folder_count {
            let subfolder = subfolders.Item(i + 1).ok();
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
            .GetTasks(TASK_ENUM_HIDDEN)
            .context("Failed to get registered tasks")?;

        let count = registered_tasks.Count().context("Failed to get task count")?;

        for i in 0..count {
            let task = registered_tasks.Item(i + 1).ok();
            if let Some(task) = task {
                if task.Name().to_string() == name {
                    let folder_path = folder.Path().to_string();
                    let full_path = if folder_path == "\\" {
                        format!("\\{}", name)
                    } else {
                        format!("{}\\{}", folder_path, name)
                    };
                    return Ok((folder.clone(), full_path));
                }
            }
        }

        // Search in subfolders
        let subfolders = folder
            .GetFolders(0)
            .context("Failed to get subfolders")?;

        let folder_count = subfolders.Count().context("Failed to get folder count")?;

        for i in 0..folder_count {
            let subfolder = subfolders.Item(i + 1).ok();
            if let Some(subfolder) = subfolder {
                if let Ok((found_folder, path)) = Self::find_task_path(&subfolder, name) {
                    return Ok((found_folder, path));
                }
            }
        }

        anyhow::bail!("Task not found")
    }
}

