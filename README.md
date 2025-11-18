# DeepBoot 🚀

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)](https://www.microsoft.com/windows/)
[![Language](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)

**DeepBoot** is an open-source Rust utility that finds startup apps legacy cleaners miss. It scans Task Scheduler, Services, and deep Registry keys to detect stealthy background processes like OpenAI. Reclaim your system resources and take full control of your Windows boot sequence.

> **The Problem:** Traditional tools like CCleaner or the default Task Manager often check `HKCU\...\Run` and call it a day.
> **The Reality:** Modern apps are smarter. They hide in Task Scheduler, Services, and obscure system folders. DeepBoot finds them.

---

## ⚡ Features

* **Task Scheduler Inspection:** Detects apps that use "At Log On" or "On Idle" scheduled tasks to bypass standard startup checks (the #1 method used by modern Electron apps).
* **Deep Registry Scanning:** Checks `Run`, `RunOnce`, `RunServices`, and WoW6432Nodes.
* **Service Filtering:** Distinguishes between essential Windows services and third-party "update helpers" that slow down boot times.
* **Blazing Fast:** Built with Rust for memory safety and zero-overhead performance.
* **Toggle & Destroy:** Temporarily disable a startup item or completely remove the entry.

## 🛠 Tech Stack

* **Core:** Rust (Edition 2021)
* **Dependencies:** `windows-rs`, `winreg`, `serde`
* **Architecture:** x64 Windows

## 🚀 Getting Started

### Prerequisites

* Windows 10 or 11
* [Rust & Cargo](https://www.rust-lang.org/tools/install) installed
* Administrator privileges (required to read HKLM keys and Task Scheduler)

### Installation & Build

```bash
# Clone the repository
git clone [https://github.com/makalin/DeepBoot.git](https://github.com/makalin/DeepBoot.git)

# Navigate to directory
cd DeepBoot

# Build for Release (Optimized)
cargo build --release

# Run the executable
./target/release/deepboot.exe
````

## 🗺 Roadmap

  * [ ] **Core Engine:** Implement `ITaskService` COM wrapper in Rust.
  * [ ] **Registry Logic:** Map all 6 major registry startup paths using `winreg`.
  * [ ] **TUI:** Implement a Terminal User Interface (using `ratatui`).
  * [ ] **Community Whitelist:** A JSON-based list of "safe" system processes.

## 🤝 Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## 👤 Author

**Mehmet T. AKALIN**

  * **Website:** [dv.com.tr](https://dv.com.tr)
  * **GitHub:** [@makalin](https://github.com/makalin)
  * **LinkedIn:** [Mehmet T. AKALIN](https://www.linkedin.com/in/makalin/)
  * **X (Twitter):** [@makalin](https://x.com/makalin)

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
