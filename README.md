# ⚡ Process Killer TUI

A fast, minimal, and safe terminal-based process viewer and killer built with Rust and Ratatui. 

Built because traditional process managers either clutter the screen with 50 child processes for a single browser instance, or calculate memory in ways that make a 4GB app look like it's using 16GB. This tool groups them up and gives you the honest numbers.

## ✨ Features

* **Smart Grouping:** Related processes are grouped under a single app name (e.g., all Vivaldi/Spotify sub-processes appear as one entry).
* **Always-On Fuzzy Search:** Just start typing. No need to hit `/` or switch modes. Results filter instantly using a fuzzy matcher.
* **Quick Kill:** Instantly terminate all processes in a grouped tree with a single keypress (`k`).
* **Live Updates:** Process data refreshes automatically without interrupting your search query or cursor position.
* **Honest Memory Stats:** Bypasses the "shared memory trap" on Linux by displaying the max memory footprint of a group, color-coded by severity.
* **Sorting Modes:** Cycle seamlessly between sorting by Name, CPU usage, or Memory usage.

## 🚀 Installation

You will need [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system. 

**1. Clone the repository**
```bash
git clone https://github.com/purinsu14/process-killer-tui.git
cd process-killer-tui
```

**2. Build and Run (Development)**
```bash
cargo run
```

**3. Build Optimized Binary (Recommended)**
For the best performance and lowest overhead, build the release version:
```bash
cargo build --release
```
You can then run the binary directly or move it to your PATH:
```bash
./target/release/process-killer-tui
# Optional: sudo mv ./target/release/process-killer-tui /usr/local/bin/pkiller
```

## 🎮 Controls

| Key | Action |
| :--- | :--- |
| `↑` / `↓` | Navigate the process list |
| `k` | **Kill** the selected process group |
| `s` | Cycle **Sort** mode (Name → CPU → Memory) |
| `q` | **Quit** the application |
| `[Type]` | **Search** — just start typing letters/numbers |
| `Backspace`| Delete last search character |
| `Esc` | Clear search query |

## 🧠 Technical Notes

* **Memory Calculation:** Memory is reported as the highest (peak) usage among processes in a specific group. It may differ slightly from tools like `btop` due to how shared memory is deduplicated.
* **Kill Signals:** Pressing `k` sends a kill signal to *all* PIDs associated with that group name. 
* **Permissions:** If you are trying to kill system-level processes or apps owned by other users, you will need to run this tool with `sudo`.

## 🛠️ Built With
* [Ratatui](https://ratatui.rs/) - Terminal UI rendering
* [sysinfo](https://crates.io/crates/sysinfo) - Cross-platform system information
* [crossterm](https://crates.io/crates/crossterm) - Terminal backend
* [fuzzy-matcher](https://crates.io/crates/fuzzy-matcher) - Skim-style fuzzy search

---
*Made by [purinsu14](https://github.com/purinsu14)*
