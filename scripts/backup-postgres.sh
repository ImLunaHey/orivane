#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "DATABASE_URL is required" >&2
  exit 2
fi

backup_dir="${ORIVANE_BACKUP_DIR:-./backups}"
mkdir -p -- "$backup_dir"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
destination="$backup_dir/orivane-$timestamp.dump"
temporary="$destination.partial"

umask 077
pg_dump --dbname="$DATABASE_URL" --format=custom --compress=9 --file="$temporary"
pg_restore --list "$temporary" >/dev/null
mv -- "$temporary" "$destination"
sha256sum "$destination" > "$destination.sha256"
echo "$destination"
