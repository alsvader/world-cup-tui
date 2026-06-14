#!/usr/bin/env bash
# Run once after `gh auth login` to set repository topics on GitHub.
set -euo pipefail
gh repo edit alsvader/world-cup-tui \
  --add-topic rust \
  --add-topic tui \
  --add-topic ratatui \
  --add-topic terminal \
  --add-topic cli \
  --add-topic world-cup \
  --add-topic fifa \
  --add-topic soccer \
  --add-topic football
