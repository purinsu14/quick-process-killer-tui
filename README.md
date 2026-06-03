## Process Killer TUI in Rust

An easy to use terminal-based process viewer and killer written in Rust.
Equipped with fuzzy search, live updates, sorting modes, and smart grouping.

Pre-built binary for Linux available on the [releases page](https://github.com/purinsu14/quick-process-killer-tui/releases).

## Install (with Rust)
```
git clone https://github.com/purinsu14/quick-process-killer-tui.git
cd process-killer-tui
cargo install --path .
quick-process-killer-tui
```
or just run directly (in the project directory):
```
cargo run
```

## Install with binary (no Rust needed)

Download the binary, open terminal in the download path, then:
```
chmod +x <filename>
./<filename>
```

## Controls

| Key | Action |
| :--- | :--- |
| `↑` / `↓` | Navigate the process list |
| `k` | Kill the selected process group |
| `s` | Cycle sort mode (Name → CPU → Memory) |
| `q` | Quit |
| `/` | Open search bar |
| `Backspace` | Delete last search character |
| `Esc` | Clear search |

## Notes

- Memory is reported as the peak usage among processes in a group.
- `k` sends a kill signal to all PIDs in that group.
- Run with `sudo` to kill system-level or other-user processes.

---
*Made by [purinsu14](https://github.com/purinsu14)*
