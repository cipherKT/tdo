# tdo

A terminal-native project and task manager built for Omarchy/Hyprland. Keyboard-driven, SQLite-backed, with a Waybar integration that surfaces your most urgent pending task.

## Features

- **Project and task management** — organize work into projects, each with tasks carrying priority, due dates, and tags
- **Fuzzy search** — press `/` to search and navigate, or create new projects/tasks when no match is found
- **Vim-style keybindings** — `j`/`k` to navigate, `i` to edit, `d` to delete, `space` to toggle done
- **Stats pane** — live task counts with proportional bars and a per-project breakdown
- **Metadata pane** — description, tags, priority, and due date for the selected item
- **Waybar integration** — shows your earliest pending task with a relative due date, updates every 30 seconds
- **Omarchy theme support** — reads `~/.config/omarchy/current/theme/colors.toml` and applies colors live

## Requirements

- Rust toolchain (`rustup`)
- `jq` (for the Waybar script)
- Omarchy/Hyprland (optional — falls back to built-in defaults if theme file is absent)

## Install

```bash
# clone the repo
git clone https://github.com/yourusername/tdo ~/Projects/tdo
cd ~/Projects/tdo

# install the binary to ~/.cargo/bin/tdo
cargo install --path crates/tdo

# verify
tdo
```

## Waybar integration

```bash
# copy the script
mkdir -p ~/.config/waybar/scripts
cp task-list.sh ~/.config/waybar/scripts/task-list.sh
chmod +x ~/.config/waybar/scripts/task-list.sh
```

Add to your waybar config:

```json
"modules-left": ["custom/omarchy", "hyprland/workspaces", "custom/tasks"],

"custom/tasks": {
    "exec": "~/.config/waybar/scripts/task-list.sh",
    "return-type": "json",
    "interval": 30,
    "format": "󰄨  {}",
    "tooltip": true
}
```

Reload waybar after saving.

## Keybindings

### Browsing mode
| Key | Action |
|-----|--------|
| `j` / `k` | Move selection down / up |
| `enter` | Open selected project |
| `esc` | Go back to home |
| `/` | Fuzzy search — navigate or create |
| `i` | Edit selected project or task |
| `d` | Delete selected (with confirmation) |
| `space` | Toggle task done/pending |
| `q` | Quit |

### Search mode (`/`)
| Key | Action |
|-----|--------|
| Type | Filter list live |
| `enter` (match) | Navigate into project / select task |
| `enter` (no match) | Start creation form with typed name |
| `esc` | Cancel, return to browsing |

### Form mode (`i` or create)
| Key | Action |
|-----|--------|
| `j` / `k` | Move between fields |
| `i` | Enter insert mode for current field |
| `esc` (insert) | Confirm field, exit insert mode |
| `enter` | Save form |
| `esc` (normal) | Cancel form |

## Data

Tasks and projects are stored in `~/.local/share/tdo/tdo.db` (SQLite, WAL mode).

## Stack

| Layer | Tech |
|-------|------|
| Language | Rust |
| TUI | Ratatui + Crossterm |
| Storage | SQLite via rusqlite (bundled) |
| CLI args | clap |
| Serialization | serde + serde_json |
| Dates | chrono |
