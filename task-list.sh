#!/usr/bin/env bash
#
# waybar custom/tasks module script
# Calls tdo --today to get today's pending tasks,
# formats a relative due date, and emits waybar JSON (text + tooltip).

set -euo pipefail

TODO_BIN="${TDO_BIN:-tdo}"

if ! command -v jq &>/dev/null; then
  printf '{"text": "jq missing", "tooltip": "Install jq: sudo pacman -S jq"}\n'
  exit 0
fi

# --- fetch -------------------------------------------------------------
raw="$("$TODO_BIN" --today 2>/dev/null || echo '[]')"

if [[ -z "$raw" || "$raw" == "[]" || "$raw" == "null" ]]; then
  printf '{"text": "all clear \u2728", "tooltip": "No pending tasks. Nice."}\n'
  exit 0
fi

# --- parse first task for bar text -------------------------------------
first_task=$(echo "$raw" | jq -r '.[0] // empty')
if [[ -z "$first_task" || "$first_task" == "null" ]]; then
  printf '{"text": "all clear \u2728", "tooltip": "No pending tasks. Nice."}\n'
  exit 0
fi

name=$(echo "$first_task" | jq -r '.name // empty')
due=$(echo "$first_task" | jq -r '.due  // empty')
project=$(echo "$first_task" | jq -r '.project // empty')
priority=$(echo "$first_task" | jq -r '.priority // empty')

if [[ -z "$name" || -z "$due" ]]; then
  printf '{"text": "all clear \u2728", "tooltip": "No pending tasks. Nice."}\n'
  exit 0
fi

# Format text with priority prefix if available
prefix=""
if [[ -n "$priority" ]]; then
  prefix="[P${priority}] "
fi

# --- relative date math for first task ---------------------------------
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
else
  rel="due today"
fi

text="${prefix}${name}: ${rel}"

# --- tooltip construction ---------------------------------------------
tooltip=""
num_tasks=$(echo "$raw" | jq '. | length')
for ((i=0; i<num_tasks; i++)); do
  t_raw=$(echo "$raw" | jq -r ".[$i]")
  t_name=$(echo "$t_raw" | jq -r '.name')
  t_due=$(echo "$t_raw" | jq -r '.due')
  t_project=$(echo "$t_raw" | jq -r '.project // empty')
  t_priority=$(echo "$t_raw" | jq -r '.priority // empty')

  t_due_date=$(date -d "$t_due" +%Y-%m-%d 2>/dev/null || echo "$today_date")
  t_due_epoch=$(date -d "$t_due_date" +%s)
  t_diff_seconds=$((t_due_epoch - today_epoch))
  if ((t_diff_seconds >= 0)); then
    t_diff_days=$(((t_diff_seconds + 43200) / 86400))
  else
    t_diff_days=$(((t_diff_seconds - 43200) / 86400))
  fi

  if ((t_diff_days < 0)); then
    t_rel="overdue $((-t_diff_days))d"
  else
    t_rel="due today"
  fi

  line=""
  [[ -n "$t_priority" ]] && line="[P${t_priority}] "
  line="${line}${t_name}"
  [[ -n "$t_project" ]] && line="${line}  (${t_project})"
  line="${line} - ${t_rel}"

  if [[ -z "$tooltip" ]]; then
    tooltip="$line"
  else
    tooltip="${tooltip}
${line}"
  fi
done

# --- output JSON safely via jq -----------------------------------------
jq -c -n --arg text "$text" --arg tooltip "$tooltip" '{"text": $text, "tooltip": $tooltip}'
