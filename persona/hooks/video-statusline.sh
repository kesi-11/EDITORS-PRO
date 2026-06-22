#!/usr/bin/env bash
#
# video-statusline.sh
#
# Statusline script for the EDITORS-PRO video persona. Reads the
# .video-active flag file and prints a short badge indicating the
# active mode. Inspired by ponytail's ponytail-statusline.sh.
#
# Usage in ~/.claude/settings.json:
#   "statusLine": {
#     "type": "command",
#     "command": "bash /path/to/persona/hooks/video-statusline.sh"
#   }
#
# Output:
#   [VIDEO:FULL]   (green, broadcast default)
#   [VIDEO:LITE]   (cyan, social cut)
#   [VIDEO:ULTRA]  (magenta, feature grade)
#   (empty)        (off / no flag file)

set -euo pipefail

FLAG_FILE="${PWD}/.video-active"

if [[ ! -f "$FLAG_FILE" ]]; then
  exit 0
fi

MODE="$(cat "$FLAG_FILE" 2>/dev/null || echo 'off')"

# ANSI 256-color codes
GREEN=$'\033[38;5;34m'
CYAN=$'\033[38;5;45m'
MAGENTA=$'\033[38;5;177m'
RESET=$'\033[0m'

case "$MODE" in
  full)
    printf '%s[VIDEO:FULL]%s' "$GREEN" "$RESET"
    ;;
  lite)
    printf '%s[VIDEO:LITE]%s' "$CYAN" "$RESET"
    ;;
  ultra)
    printf '%s[VIDEO:ULTRA]%s' "$MAGENTA" "$RESET"
    ;;
  off|*)
    # Off — print nothing
    ;;
esac
