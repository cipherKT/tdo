# TDO TUI Redesign Proposals

This document outlines structural and visual enhancements for the `tdo` terminal interface. It proposes moving from the current simple split-pane view to a highly functional, aesthetic, and interactive multi-column layout.

---

## 1. Three-Column Dashboard Layout

- **Left Column (Sidebar)**: Scrollable list of projects, letting users switch projects instantly without changing full-screen context.
- **Middle Column (Main Pane)**: The list of tasks belonging to the currently selected project, with checkboxes (`[ ]` / `[x]`) and status indicators.
- **Right Column (Details/Inspect Pane)**: Rich description, creation date, due date, priority badges, and formatted tags for the currently selected project/task.
- **Status/Progress Bar**: A sleek, ~2-3 pixel-tall multi-colored progress bar.

### Selected Progress Bar Design (Option A: Lower One-Quarter Block `▂`)
We will use the Unicode lower one-quarter block element (`▂` / `\u{2582}`) to render a thin, colored bar representing task status ratios within a single row:
- **Green**: Completed tasks
- **Yellow**: Pending tasks (on time)
- **Red**: Overdue tasks

```
Progress: ▂▂▂▂▂▂ ▂▂▂ ▂  [6 Done / 3 Pending / 1 Overdue]
          (grn) (ylw)(red)
```

---

## 2. Interactive Vim-Modal Popup for Editing

When editing or creating a task/project:
- Pressing `i` on a task opens the edit modal popup.
- The popup displays the task title in a larger, highly distinguished font.
- Navigation inside the modal uses **Vim motions**.

### Modal Modes of Operation:
1. **Normal Mode**:
   - `j` / `k`: Navigate selection between editable fields (Name, Description, Tags, Priority, Due Date).
   - `i`: Enter **Insert Mode** on the currently highlighted field.
   - `Enter` / `:w`: Save and submit form.
   - `Esc` / `:q`: Close the modal and discard changes.
2. **Insert Mode**:
   - Type to edit the field content.
   - `Esc`: Return to **Normal Mode** to resume navigation with `j`/`k`.

```
+─────────────────────────────────────────────────────────────+
|  TASK EDIT: REFACTOR TUI LAYOUT                             |
| ─────────────────────────────────────────────────────────── |
|  ▶ Name:        Refactor TUI Layout_______________________  |
|    Description: Restructure views into columns____________  |
|    Tags:        #tui #refactor____________________________  |
|    Priority:    Medium                                      |
|    Due Date:    2026-07-01                                  |
| ─────────────────────────────────────────────────────────── |
|  [Normal]  j/k: Select Field  ·  i: Edit Field  ·  Enter: Save |
+─────────────────────────────────────────────────────────────+
```

---

## 3. Themes & Style (Omarchy Support)

The TUI color configuration will be isolated in `themes.rs` to allow the user to easily configure or inherit the **Omarchy** theme palette.

- **Completed**: Muted Green / Slate
- **Pending**: Soft Gold / Yellow
- **Overdue**: Vivid Red / Coral
- **Focused borders / Highlight Cursor**: Cyan / Lavender (consistent with Omarchy primary accent)
