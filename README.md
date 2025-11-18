# DeepBoot Pro 🚀

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)](https://www.microsoft.com/windows/)
[![Language](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)

**DeepBoot Pro** is a professional-grade, open-source Rust utility that finds startup apps legacy cleaners miss. It scans Task Scheduler, Services, and deep Registry keys to detect stealthy background processes. Reclaim your system resources and take full control of your Windows boot sequence with advanced features like batch operations, export capabilities, whitelist management, and comprehensive logging.

> **The Problem:** Traditional tools like CCleaner or the default Task Manager often check `HKCU\...\Run` and call it a day.
> **The Reality:** Modern apps are smarter. They hide in Task Scheduler, Services, and obscure system folders. DeepBoot Pro finds them all.

---

## ⚡ Core Features

### 🔍 Advanced Scanning
* **Task Scheduler Inspection:** Detects apps that use "At Log On" or "On Idle" scheduled tasks to bypass standard startup checks (the #1 method used by modern Electron apps).
* **Deep Registry Scanning:** Checks `Run`, `RunOnce`, `RunServices`, and WoW6432Nodes across both HKCU and HKLM.
* **Service Filtering:** Distinguishes between essential Windows services and third-party "update helpers" that slow down boot times.
* **Comprehensive Detection:** Scans all major Windows startup locations in a single pass.

### 🎯 Professional Features

* **📊 Statistics & Analytics:** Real-time statistics showing entry counts by source, enabled/disabled status, and percentage breakdowns.
* **💾 Export Functionality:** Export scan results to JSON, CSV, or Markdown formats with timestamped filenames.
* **✅ Whitelist Management:** Community-based whitelist system to mark safe processes. Add/remove entries with persistent storage.
* **🔄 Batch Operations:** Select multiple entries and perform batch disable/remove operations with success tracking.
* **💿 Backup & Restore:** Automatic backups before modifications. List, restore, or delete backups with timestamp tracking.
* **📝 Action Logging:** Comprehensive logging system that records all actions, scans, and batch operations with timestamps.
* **🔎 Search & Filter:** Real-time search by name, command, or description. Filter by source, status, and more.
* **📋 Multi-Select:** Select multiple entries for batch operations with visual indicators.
* **⚙️ Configuration Management:** Persistent settings for auto-backup, whitelist visibility, default sorting, and more.
* **🎨 Enhanced TUI:** Beautiful terminal interface with multiple view modes, status bar, and intuitive navigation.

### ⚡ Performance

* **Blazing Fast:** Built with Rust for memory safety and zero-overhead performance.
* **Efficient Scanning:** Optimized algorithms for quick startup detection.
* **Low Resource Usage:** Minimal memory footprint even with large entry lists.

## 🛠 Tech Stack

* **Core:** Rust (Edition 2021)
* **Dependencies:**
  - `windows-rs` - Windows API bindings
  - `winreg` - Registry access
  - `ratatui` - Terminal UI framework
  - `crossterm` - Cross-platform terminal manipulation
  - `serde` / `serde_json` - Serialization
  - `csv` - CSV export
  - `chrono` - Date/time handling
  - `dirs` - Config/data directories
  - `anyhow` - Error handling
  - `log` / `env_logger` - Logging
* **Architecture:** x64 Windows

## 🚀 Getting Started

### Prerequisites

* Windows 10 or 11
* [Rust & Cargo](https://www.rust-lang.org/tools/install) installed
* Administrator privileges (required to read HKLM keys and Task Scheduler)

### Installation & Build

```bash
# Clone the repository
git clone https://github.com/makalin/DeepBoot.git

# Navigate to directory
cd DeepBoot

# Build for Release (Optimized)
cargo build --release

# Run the executable
./target/release/deepboot.exe
```

## 📖 Usage Guide

### Keyboard Shortcuts

#### Navigation
- `↑` / `k` - Move up
- `↓` / `j` - Move down
- `Space` - Toggle selection (for batch operations)
- `Esc` / `q` - Quit (or cancel current operation)

#### Actions
- `d` - Disable selected entry(ies)
- `r` - Remove selected entry(ies)
- `w` - Add selected entry to whitelist
- `e` - Export current view to JSON
- `y` - Confirm action
- `n` - Cancel action

#### Views & Features
- `s` - Show statistics view
- `h` - Toggle help view
- `/` - Start search (type to search, Enter to apply, Esc to cancel)
- `1` - Sort by name
- `2` - Sort by source
- `3` - Sort by status (enabled/disabled)
- `4` - Sort by command

### Basic Workflow

1. **Launch DeepBoot Pro** - The application will automatically scan all startup locations.
2. **Review Statistics** - Press `s` to view detailed statistics about your startup entries.
3. **Search & Filter** - Press `/` to search for specific entries by name, command, or description.
4. **Select Entries** - Use `Space` to select multiple entries for batch operations.
5. **Take Action** - Press `d` to disable or `r` to remove selected entries.
6. **Export Results** - Press `e` to export your scan results to JSON.
7. **Whitelist Safe Entries** - Press `w` to add trusted entries to your whitelist.

### Export Formats

DeepBoot Pro supports exporting to multiple formats:

- **JSON** - Structured data with full entry details (default export)
- **CSV** - Spreadsheet-compatible format
- **Markdown** - Human-readable report format

Export files are automatically timestamped: `deepboot_export_YYYYMMDD_HHMMSS.{format}`

### Configuration

Configuration files are stored in:
- **Windows:** `%APPDATA%\deepboot\config.json`
- **Backups:** `%LOCALAPPDATA%\deepboot\backups\`
- **Logs:** `%LOCALAPPDATA%\deepboot\logs\`

You can customize:
- Auto-backup on scan
- Show/hide whitelisted entries
- Default sort preference
- Log level

## 🗺 Project Status

### ✅ Completed Features

- [x] **Core Engine:** `ITaskService` COM wrapper in Rust
- [x] **Registry Logic:** All 6 major registry startup paths using `winreg`
- [x] **TUI:** Full-featured Terminal User Interface using `ratatui`
- [x] **Community Whitelist:** JSON-based list of "safe" system processes
- [x] **Export Functionality:** JSON, CSV, and Markdown export
- [x] **Backup System:** Automatic backups with restore capability
- [x] **Logging System:** Comprehensive action logging
- [x] **Statistics:** Real-time analytics and reporting
- [x] **Batch Operations:** Multi-select and batch actions
- [x] **Search & Filter:** Advanced filtering capabilities
- [x] **Configuration Management:** Persistent settings

### 🔮 Future Enhancements

- [ ] **GUI Version:** Native Windows GUI using egui or tauri
- [ ] **Scheduled Scans:** Automatic periodic scanning
- [ ] **Cloud Sync:** Sync whitelist across devices
- [ ] **Boot Time Analysis:** Measure actual boot impact
- [ ] **Process Monitoring:** Real-time process tracking
- [ ] **Plugin System:** Extensible architecture for custom scanners
- [ ] **Multi-language Support:** Internationalization

## 📁 Project Structure

```
DeepBoot/
├── src/
│   ├── main.rs              # Application entry point
│   ├── actions.rs           # Action handlers (disable/remove)
│   ├── backup.rs            # Backup/restore system
│   ├── batch.rs             # Batch operations
│   ├── config.rs            # Configuration management
│   ├── export.rs            # Export functionality
│   ├── filter.rs            # Search and filtering
│   ├── logger.rs            # Action logging
│   ├── models.rs            # Data models
│   ├── registry.rs          # Registry scanner
│   ├── services.rs          # Services scanner
│   ├── stats.rs             # Statistics and analytics
│   ├── task_scheduler.rs    # Task Scheduler scanner
│   ├── tui.rs               # Terminal UI
│   └── whitelist.rs         # Whitelist management
├── Cargo.toml               # Project dependencies
├── README.md                 # This file
└── LICENSE                   # MIT License
```

## 🛡️ Safety Features

- **Automatic Backups:** All modifications are backed up automatically
- **Whitelist Protection:** Safe processes are protected from accidental removal
- **Confirmation Prompts:** All destructive actions require confirmation
- **Comprehensive Logging:** Every action is logged for audit trails
- **Error Handling:** Graceful error handling with informative messages

## 🤝 Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Contribution Guidelines

- Follow Rust coding standards
- Add tests for new features
- Update documentation
- Ensure all features work on Windows 10/11

## 📊 Performance

DeepBoot Pro is optimized for performance:
- **Scan Time:** Typically completes full system scan in < 5 seconds
- **Memory Usage:** < 50MB even with 1000+ entries
- **Startup Impact:** Minimal - only runs when executed

## 🐛 Troubleshooting

### Common Issues

**Issue:** "Failed to scan Task Scheduler"
- **Solution:** Run as Administrator

**Issue:** "Failed to read registry keys"
- **Solution:** Ensure you have administrator privileges

**Issue:** Export fails
- **Solution:** Check write permissions in the current directory

**Issue:** Whitelist not saving
- **Solution:** Verify config directory permissions

## 📝 License

Distributed under the MIT License. See `LICENSE` for more information.

## 👤 Author

**Mehmet T. AKALIN**

* **Website:** [dv.com.tr](https://dv.com.tr)
* **GitHub:** [@makalin](https://github.com/makalin)
* **LinkedIn:** [Mehmet T. AKALIN](https://www.linkedin.com/in/makalin/)
* **X (Twitter):** [@makalin](https://x.com/makalin)

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- UI powered by [ratatui](https://github.com/ratatui-org/ratatui)
- Windows API via [windows-rs](https://github.com/microsoft/windows-rs)

---

**⭐ If you find DeepBoot Pro useful, please consider giving it a star on GitHub!**
