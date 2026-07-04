# tdo

A terminal-native project and task manager built for Omarchy/Hyprland. Keyboard-driven, SQLite-backed, with a Waybar integration that surfaces your pending tasks for today.

## Features

- **Project and task management** — organize work into projects, each with tasks carrying priority, due dates, and tags
- **Lightweight checklists (Subtasks)** — add subtasks under tasks with strict completion invariants (parent task cannot be finished while subtasks are pending; adding/marking a subtask undone automatically reopens the parent task)
- **Fuzzy search** — press `/` to search and navigate, or create new projects/tasks when no match is found
- **Vim-style keybindings** — `j`/`k` to navigate, `i` to edit, `d` to delete, `space` to toggle done
- **Smart date parsing** — use natural shortcuts (`today`, `tomorrow`, `mon`), relative offsets (`+3`, `+1w`, `+1m`), or day-of-month (`15`) with instant normalization
- **Stats pane** — live task counts with proportional bars and a per-project breakdown
- **Metadata pane** — description, tags, priority, due date, or parent task status details for the selected item
- **Waybar integration** — shows your highest priority pending task for today, with a tooltip listing all of today's pending tasks, updating dynamically
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
    "interval": 5,
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
| `esc` | Go back to home / project selection |
| `/` | Fuzzy search — navigate or create |
| `i` | Edit selected project, task, or subtask |
| `d` | Delete selected task or subtask (with confirmation) |
| `space` | Toggle task or subtask done/pending |
| `s` | Add subtask under selected task |
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
| `esc` / `enter` (insert) | Confirm field value, exit insert mode |
| `enter` (normal) | Trigger save confirmation popup |
| `esc` (normal) | Cancel form |

### Save Confirmation popup
| Key | Action |
|-----|--------|
| `j`/`k` / `Down`/`Up` | Toggle between "Save" and "Keep editing" |
| `enter` | Confirm chosen option |
| `esc` | Close confirmation popup (equivalent to "Keep editing") |

### Smart Date Inputs

When editing the **due date** field, you don't need to type the full `YYYY-MM-DD` date. You can enter:
- **Relative offsets**: `+3` or `+3d` (3 days from now), `+1w` (1 week from now), `+2m` (2 months from now)
- **Shortcuts**: `today`, `tomorrow`, `tod`, `tom`, `t`, `tm`
- **Weekdays**: `mon`, `tue`, `wednesday` etc. (resolves to the next occurrence of that weekday)
- **Partial dates**: `MM-DD` or `MM/DD` (e.g. `07-04` or `7/4`, assuming current year)
- **Day of month**: `DD` (e.g. `15` — assumes current month if the day is in the future, otherwise next month)

Upon exiting insert mode (`esc`), the shortcut is normalized instantly to `YYYY-MM-DD`.

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
