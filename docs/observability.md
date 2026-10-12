# Observability and recovery

The API exposes Prometheus metrics at `/metrics`, liveness at `/health/live`, and dependency-aware readiness at `/health/ready`. The gateway exposes the same paths on `GATEWAY_METRICS_BIND` (port 3011 by default). Metrics deliberately avoid guild IDs, user IDs, command text, and other high-cardinality or personal labels.

Both services export readiness, uptime, failures, and latency. The gateway also exports gateway-event, interaction, and scheduler counters. The API adds database-pool and durable due-job backlog gauges. Every API response includes an `x-request-id`; supplied IDs are preserved when valid and the same value is attached to structured request logs and mutation audit records.

## Local monitoring stack

```sh
docker compose -f docker-compose.yml -f docker-compose.monitoring.yml \
  --profile monitoring up --build -d
```

Prometheus is available on `127.0.0.1:9090`, Alertmanager on `127.0.0.1:9093`, and Grafana on `127.0.0.1:3000`. Configure an authenticated Alertmanager receiver before production use; the repository default keeps alerts local so it never contains notification credentials.

The checked-in rules alert on service/readiness failures, API and interaction errors, interaction p95 latency, gateway reconnect errors, scheduler failures, due-job backlog, and database pool saturation.

## Backups

`scripts/backup-postgres.sh` creates a permission-restricted custom-format dump, validates its table of contents, and writes a SHA-256 checksum. Set `ORIVANE_BACKUP_DIR` to encrypted durable storage and schedule it outside the application containers. `scripts/verify-backup.sh` verifies structure and checksum without changing a database.

A real recovery exercise must restore into a separate empty database, run API readiness and row-count checks against it, and then delete that isolated database. Never test a restore against the active production database. Redis is treated as rebuildable cache/coordination state; durable work must remain in PostgreSQL.
