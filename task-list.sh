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
today_date=$(date -d "today" +%Y-%m-%d)
today_epoch=$(date -d "$today_date" +%s)

due_date=$(date -d "$due" +%Y-%m-%d 2>/dev/null || echo "$today_date")
due_epoch=$(date -d "$due_date" +%s)

diff_seconds=$((due_epoch - today_epoch))
if ((diff_seconds >= 0)); then
  diff_days=$(((diff_seconds + 43200) / 86400))
else
  diff_days=$(((diff_seconds - 43200) / 86400))
fi

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
