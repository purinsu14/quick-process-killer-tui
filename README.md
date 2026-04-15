# Process Killer TUI

A fast and minimal terminal-based process viewer and killer built with Rust and Ratatui.

## Features

* **Grouped Processes:** Groups related processes under a single app name (e.g. browser instances).
* **Fuzzy Search:** Quickly find processes by typing part of the name.
* **Live Updates:** Automatically refreshes process data every few seconds.
* **CPU & Memory Stats:** Displays CPU usage and memory usage per group.
* **Sorting Modes:** Sort by name, CPU usage, or memory usage.
* **Quick Kill:** Instantly terminate all processes in a group.

## Prerequisites

You need Rust installed on your system.

Install it from:
https://www.rust-lang.org/tools/install

Verify installation:

```
rustc --version
cargo --version
```

## How to Run

1. **Clone the repository**

```
git clone https://github.com/YOUR_USERNAME/YOUR_REPO.git
cd YOUR_REPO
```

2. **Run the app**

```
cargo run
```

Cargo will automatically download dependencies and build the project.

## Build Optimized Binary (Optional)

For better performance, you can build the optimized release version:

```bash
cargo build --release
```

The compiled binary will be located at:

```bash
target/release/process-killer-tui
```

You can run it directly:

```bash
./target/release/process-killer-tui
```

## Controls

```
↑ / ↓        Navigate
k            Kill selected process group
s            Change sorting mode
type         Search (fuzzy)
Backspace    Delete search input
Esc          Clear search
q            Quit
```

## Notes

* Memory usage is estimated per process group and may differ from tools like `btop` due to shared memory handling.
* Killing a group terminates all associated processes.

## Author

Made by purinsu14

