#!/usr/bin/env bash
set -euo pipefail

backup="${1:-}"
if [[ -z "$backup" || ! -f "$backup" ]]; then
  echo "usage: $0 /path/to/orivane.dump" >&2
  exit 2
fi

pg_restore --list "$backup" >/dev/null
if [[ -f "$backup.sha256" ]]; then
  sha256sum --check "$backup.sha256"
fi
echo "Backup structure and checksum are valid: $backup"
