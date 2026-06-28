#!/usr/bin/env bash
#
# waybar custom/tasks module script
# Calls tdo --next-task to get the earliest pending task,
# formats a relative due date, and emits waybar JSON (text + tooltip).

set -euo pipefail

TODO_BIN="${TDO_BIN:-tdo}"

if ! command -v jq &>/dev/null; then
  printf '{"text": "jq missing", "tooltip": "Install jq: sudo pacman -S jq"}\n'
  exit 0
fi

# --- fetch -------------------------------------------------------------
raw="$("$TODO_BIN" --next-task 2>/dev/null || echo 'null')"

if [[ -z "$raw" || "$raw" == "null" ]]; then
  printf '{"text": "all clear \u2728", "tooltip": "No pending tasks. Nice."}\n'
  exit 0
fi

# --- parse -------------------------------------------------------------
name=$(echo "$raw" | jq -r '.name // empty')
due=$(echo "$raw" | jq -r '.due  // empty')
project=$(echo "$raw" | jq -r '.project // empty')

if [[ -z "$name" || -z "$due" ]]; then
  printf '{"text": "all clear \u2728", "tooltip": "No pending tasks. Nice."}\n'
  exit 0
fi

# --- relative date math ------------------------------------------------
today_epoch=$(date -d "today" +%s)
due_epoch=$(date -d "$due" +%s 2>/dev/null || echo "$today_epoch")
diff_days=$(((due_epoch - today_epoch) / 86400))

if ((diff_days < 0)); then
  rel="overdue $((-diff_days))d"
elif ((diff_days == 0)); then
  rel="due today"
elif ((diff_days == 1)); then
  rel="due in 1d"
else
  rel="due in ${diff_days}d"
fi

text="${name}: ${rel}"

# --- tooltip -----------------------------------------------------------
tooltip="$name"
[[ -n "$project" ]] && tooltip="$tooltip  ($project)"
tooltip="$tooltip\n$rel"

# --- escape for JSON ---------------------------------------------------
text_esc=$(echo "$text" | sed 's/"/\\"/g')
tooltip_esc=$(echo -e "$tooltip" | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')

printf '{"text": "%s", "tooltip": "%s"}\n' "$text_esc" "$tooltip_esc"
