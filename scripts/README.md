# Multi-instance helpers

Utilities for running Purrgres against several databases in the same Docker container, each with its own isolated `.purrs` state.

## `run-multi.sh`

Starts one background Purrgres process per database, listed in a config file (`database:user` per line — see `databases.conf.example`).

### Setup

```bash
cp databases.conf.example databases.conf
# edit databases.conf with your databases/users

./run-multi.sh \
  --container my-postgres-container \
  --bin /opt/purrgres/purrgres \
  --backup-dir /var/backups/purrgres \
  -c databases.conf
```

### Add a New Database

```bash
echo "new_database:user" >> databases.conf
./run-multi.sh -c databases.conf
```

### Configuration

All flags can also be set via environment variables:

| Flag | Environment Variable |
|---|---|
| `--container` | `PURRGRES_CONTAINER` |
| `--bin` | `PURRGRES_BIN` |
| `--backup-dir` | `PURRGRES_BKP_DIR` |

This is handy for wrapping in your own deploy scripts:

```bash
export PURRGRES_CONTAINER="my_postgres"
export PURRGRES_BIN="/usr/local/bin/purrgres"
export PURRGRES_BKP_DIR="/backups/purrgres"

./run-multi.sh -c databases.conf
```

### How It Works

Each instance:
- Runs detached (`nohup`) with its own `$HOME` at `<backup-dir>/<database>/`
- Writes its own log to `<backup-dir>/<database>/.purrs/purrgres.log`
- Tracks its PID to avoid starting duplicate processes on re-run
- Has isolated config at `<backup-dir>/<database>/.purrgres/purrgres.toml`

**Re-running the script is safe:** it skips any database whose instance is already alive.

### Directory Structure

```
~/purrgres_backups/
├── analytics/
│   ├── .purrgres/
│   │   ├── analytics_12_08_2026_22_02_backup.sql
│   │   ├── analytics_11_08_2026_17_05_backup.sql
│   │   └── purrgres.toml
│   └── .purrs/
│       ├── purrgres.log
│       └── purrgres.pid
├── auth/
│   └── ...
└── invoice/
    └── ...
```

---

## `purrgres-quick.sh`

A simplified command-line interface for common Purrgres operations. No need to remember `$HOME` paths — just use database names.

### Usage

```bash
./purrgres-quick.sh <command> [args]
```

### Commands

| Command | Usage | Description |
|---------|-------|-------------|
| `status` | `status <db>` | Show backup status for a database |
| `list` | `list <db>` | List all backups for a database |
| `logs` | `logs <db>` | Show real-time logs (Ctrl+C to exit) |
| `logs-tail` | `logs-tail <db> [lines]` | Show last N lines of logs (default: 50) |
| `running` | `running` | List all running instances |
| `restore` | `restore <db> <backup> <user>` | Restore a backup to a database |
| `stop` | `stop <db>` | Stop a specific database instance |
| `stop-all` | `stop-all` | Stop all instances for this container |
| `is-running` | `is-running <db>` | Check if a database is running |
| `summary` | `summary` | Show summary of all instances |
| `help` | `help` | Show help message |

### Examples

```bash
# Check status of analytics
./purrgres-quick.sh status analytics

# List all backups for auth
./purrgres-quick.sh list auth

# Watch logs for invoice in real-time
./purrgres-quick.sh logs invoice

# Show last 100 lines of logs for wallet
./purrgres-quick.sh logs-tail wallet 100

# List all running instances
./purrgres-quick.sh running

# Restore a backup
./purrgres-quick.sh restore auth auth_12_08_2026_22_02_backup.sql ivy

# Stop analytics instance
./purrgres-quick.sh stop analytics

# Stop all instances
./purrgres-quick.sh stop-all

# Check if db is running
./purrgres-quick.sh is-running db

# Show summary of all instances
./purrgres-quick.sh summary
```

### Configuration

Set environment variables to customize:

```bash
export PURRGRES_BKP_DIR="/custom/backup/path"
export PURRGRES_BIN="/custom/path/purrgres"
export PURRGRES_CONTAINER="my_postgres_container"

./purrgres-quick.sh status analytics
```

---

## `purrgres-helpers.sh`

A collection of bash functions for advanced use cases. Source this script to use the functions in your own scripts.

### Usage

```bash
source ./purrgres-helpers.sh

purrgres_status analytics
purrgres_list auth
purrgres_summary
```

### Available Functions

| Function | Usage | Description |
|----------|-------|-------------|
| `purrgres_list_running` | `purrgres_list_running` | List all running instances |
| `purrgres_status` | `purrgres_status <db>` | Show backup status for a database |
| `purrgres_list` | `purrgres_list <db>` | List all backups for a database |
| `purrgres_logs` | `purrgres_logs <db>` | Show real-time logs |
| `purrgres_logs_tail` | `purrgres_logs_tail <db> [lines]` | Show last N lines of logs |
| `purrgres_restore` | `purrgres_restore <db> <backup> <user>` | Restore a backup |
| `purrgres_stop` | `purrgres_stop <db>` | Stop a specific instance |
| `purrgres_stop_all` | `purrgres_stop_all` | Stop all instances |
| `purrgres_all_dbs` | `purrgres_all_dbs` | List all configured databases |
| `purrgres_is_running` | `purrgres_is_running <db>` | Check if instance is running |
| `purrgres_summary` | `purrgres_summary` | Show summary of all instances |
| `purrgres_restart` | `purrgres_restart <db>` | Restart a database instance |

### Examples

```bash
# Source the helpers
source ./purrgres-helpers.sh

# Get status for analytics
purrgres_status analytics

# List backups for auth
purrgres_list auth

# Watch logs for invoice
purrgres_logs invoice

# Show last 50 lines for wallet
purrgres_logs_tail wallet 50

# Restore a backup
purrgres_restore auth auth_12_08_2026_22_02_backup.sql ivy

# Stop analytics
purrgres_stop analytics

# Stop all
purrgres_stop_all

# List all databases
purrgres_all_dbs

# Show complete summary
purrgres_summary

# Check if running
purrgres_is_running db
```

### Configuration

Set environment variables before sourcing:

```bash
export PURRGRES_BKP_DIR="/var/backups/purrgres"
export PURRGRES_BIN="/usr/local/bin/purrgres"
export PURRGRES_CONTAINER="database_service-postgres-1"

source ./purrgres-helpers.sh
purrgres_status analytics
```

---

## Quick Reference

### Common Tasks

**Start all backups for configured databases:**
```bash
./run-multi.sh -c databases.conf --container postgres --bin purrgres --backup-dir ~/purrgres_backups
```

**Check overall status:**
```bash
./purrgres-quick.sh summary
```

**Monitor a specific database:**
```bash
./purrgres-quick.sh logs analytics
```

**Restore a backup:**
```bash
./purrgres-quick.sh restore auth auth_12_08_2026_22_02_backup.sql ivy
```

**Stop and restart all:**
```bash
./purrgres-quick.sh stop-all
sleep 2
./run-multi.sh -c databases.conf --container postgres --bin purrgres --backup-dir ~/purrgres_backups
```

**Integrate with your deploy:**
```bash
#!/bin/bash
export PURRGRES_CONTAINER="database_service-postgres-1"
export PURRGRES_BIN="/srv/db/services/database_service/purrgres"
export PURRGRES_BKP_DIR="/home/user/backups_db"

./run-multi.sh -c databases.conf
```

---

## Systemd Service

A systemd template unit for production hosts, as an alternative to `nohup` — gives you `systemctl status`, `journalctl`, restart-on-failure, and start-on-boot.

### Setup

```bash
# Copy the service template
sudo cp ../systemd/purrgres@.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Start a backup for a specific database
sudo systemctl start purrgres@mydb

# Enable auto-start on boot
sudo systemctl enable purrgres@mydb

# Check status
sudo systemctl status purrgres@mydb

# View logs
journalctl -u purrgres@mydb -f
```

See the comments in the unit file for additional setup steps.

---

## Troubleshooting

### Instance won't start

```bash
# Check if container is running
docker ps | grep postgres

# Check for stale PID files
ls -la ~/purrgres_backups/analytics/.purrs/purrgres.pid

# View logs
./purrgres-quick.sh logs-tail analytics 50
```

### Restore doesn't work

```bash
# List available backups
./purrgres-quick.sh list auth

# Check the filename exactly
ls ~/purrgres_backups/auth/.purrgres/

# Restore with exact filename
./purrgres-quick.sh restore auth auth_12_08_2026_22_02_backup.sql ivy
```

### Remote sync failing

```bash
# Check logs for remote errors
./purrgres-quick.sh logs-tail analytics 100 | grep -i remote

# Verify server is reachable
curl http://<server-ip>:8443/api/health

# Check if config has [remote] section
cat ~/purrgres_backups/analytics/.purrgres/purrgres.toml | grep -A5 "\[remote\]"
```

### Multiple instances running for same database

```bash
# Find and kill duplicates
ps aux | grep "purrgres.*--database analytics"
pkill -f "purrgres.*--database analytics"

# Restart with run-multi.sh
./run-multi.sh -c databases.conf --container postgres --bin purrgres --backup-dir ~/purrgres_backups
```