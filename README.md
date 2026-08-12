<p align="center">
  <img width="150px" src="https://github.com/hi-im-aurelio/purrgres/raw/master/static/icone.webp">
</p>

<h1 align="center">Purrgres</h1>

<p align="center">
  An automated backup tool for PostgreSQL in Docker containers — with remote sync, retention policies, and a built-in receiver server.
</p>

> **Purrgres** is a play on _Postgres_, but with a _"purr"_ feel — as if a kitten were taking care of your database. 🐱

---

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
  - [Download Binary](#download-binary)
  - [Compile from Source](#compile-from-source)
  - [Add to PATH](#add-to-path)
- [Configuration](#configuration)
  - [Config File (`purrgres.toml`)](#config-file-purrgrestoml)
  - [Full Example](#full-config-example)
- [Usage — Client Mode (Backups)](#usage--client-mode-backups)
  - [Start a Backup Process](#start-a-backup-process)
  - [Run in Background](#run-in-background)
  - [List Backups](#list-backups)
  - [Restore a Backup](#restore-a-backup)
  - [Check Status](#check-status)
  - [Stop the Backup Process](#stop-the-backup-process)
- [Usage — Server Mode (Receive Backups)](#usage--server-mode-receive-backups)
  - [Start the Server](#start-the-server)
  - [API Endpoints](#api-endpoints)
  - [Testing the Server](#testing-the-server)
- [Remote Backup Sync](#remote-backup-sync)
  - [How It Works](#how-it-works)
  - [Checksum Verification](#checksum-verification)
- [Retention Policies](#retention-policies)
- [Multi-Database Backups](#multi-database-backups)
  - [Config File Format](#config-file-format)
  - [Running Multiple Instances](#running-multiple-instances)
  - [Environment Variables](#environment-variables)
- [Running as a Systemd Service](#running-as-a-systemd-service)
- [CLI Reference](#cli-reference)
- [Semantic Versioning](#semantic-versioning)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **Automatic backups** — Runs `pg_dump` on a Docker container every 24 hours automatically.
- **Restore backups** — Restore any `.sql` backup directly into the database.
- **List backups** — View all backups performed, with dates and restore points.
- **Monitor status** — Check if a backup process is running, and for how long.
- **Remote sync** — Automatically send backups to a remote Purrgres server after each dump.
- **Server mode** — Turn any machine into a backup receiver with a built-in API.
- **Checksum verification** — SHA256 integrity check on every remote transfer.
- **Compression** — Optional gzip compression before sending backups remotely.
- **Retention policies** — Automatically delete old backups (locally and remotely).
- **Multi-database support** — Orchestrate isolated backup processes for multiple databases via a helper script.
- **Retry on failure** — Remote sends retry up to 3 times with exponential backoff.

---

## Quick Start

```bash
# 1. Download or compile purrgres (see Installation below)

# 2. Start backing up a database (foreground)
purrgres --user postgres --database mydb --container my_postgres_container

# 3. Or run in background
nohup purrgres -u postgres -d mydb -c my_container > /dev/null 2>&1 &

# 4. Check status
purrgres --stats

# 5. List backups
purrgres --list-purrs

# 6. Restore a backup
purrgres --rpurry 11_08_2026_03_00_backup.sql -u postgres -d mydb -c my_container
```

---

## Installation

### Download Binary

Pre-compiled binaries for Linux are available on the [Releases](https://github.com/hi-im-aurelio/purrgres/releases) page.

```bash
# Download and extract
tar -xvf purrgres-*.tar.gz

# Make it executable
chmod +x purrgres

# Test it
./purrgres --version
```

### Compile from Source

Requires [Rust](https://www.rust-lang.org/tools/install) (1.70+).

```bash
git clone https://github.com/fariosofernando/Purrgres.git
cd purrgres
cargo build --release

# Binary will be at: target/release/purrgres
```

### Add to PATH

To run `purrgres` from anywhere:

**Bash:**

```bash
echo 'export PATH="$PATH:/path/to/purrgres/directory"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**

```bash
echo 'export PATH="$PATH:/path/to/purrgres/directory"' >> ~/.zshrc
source ~/.zshrc
```

Or copy the binary directly:

```bash
sudo cp purrgres /usr/local/bin/
```

---

## Configuration

Purrgres works out of the box with **zero configuration** — just pass the CLI flags. But for advanced features (remote sync, retention, server mode), you can create a config file.

### Config File (`purrgres.toml`)

Place it at `~/.purrgres/purrgres.toml`, or specify a custom path with `--config`.

```bash
mkdir -p ~/.purrgres
nano ~/.purrgres/purrgres.toml
```

### Full Config Example

```toml
# ─────────────────────────────────────────────────
# Remote Sync (Client Side)
# After each backup, send it to a remote server.
# ─────────────────────────────────────────────────
[remote]
enabled = true                     # Set to false to disable remote sync
host = "192.168.1.50"              # IP or hostname of the Purrgres server
port = 8443                        # Port the server is listening on
api_key = "your-secret-api-key"    # Must match the server's api_key
compress = true                    # gzip the backup before sending (recommended)

# ─────────────────────────────────────────────────
# Server Mode
# Turn this machine into a backup receiver.
# Only needed on the machine that RECEIVES backups.
# ─────────────────────────────────────────────────
[server]
port = 8443                                      # Port to listen on
storage_path = "/var/lib/purrgres/backups"        # Where to store received backups
api_key = "your-secret-api-key"                   # Clients must send this key
max_upload_size_mb = 500                          # Max file size per upload (in MB)
max_remote_backups = 30                           # Keep only the last 30 backups

# ─────────────────────────────────────────────────
# Retention (Client Side)
# Automatically delete old local backups.
# ─────────────────────────────────────────────────
[retention]
max_local_backups = 7              # Keep only the last 7 backups locally
```

> **Note:** You don't need all sections. Use only what you need:
> - Just doing local backups? → No config file needed.
> - Want remote sync? → Add `[remote]` section.
> - Setting up a receiver? → Add `[server]` section.
> - Want auto-cleanup? → Add `[retention]` section.

---

## Usage — Client Mode (Backups)

Client mode is the default. It performs `pg_dump` on a PostgreSQL database running inside a Docker container.

### Start a Backup Process

```bash
purrgres --user <PG_USER> --database <DB_NAME> --container <DOCKER_CONTAINER>

# Short flags
purrgres -u postgres -d mydb -c my_postgres_container
```

This will:
1. Run `pg_dump` immediately.
2. Save the backup to `~/.purrgres/<date>_backup.sql`.
3. If `[remote]` is enabled, compress and send it to the remote server.
4. If `[retention]` is configured, delete old backups beyond the limit.
5. Repeat every **24 hours**.

### Run in Background

For production, run Purrgres in the background so it survives terminal closure:

```bash
nohup purrgres -u postgres -d mydb -c my_container > /dev/null 2>&1 &
```

| Method | Command | Terminal Closes → Process... |
|---|---|---|
| Foreground | `purrgres -u ... -d ... -c ...` |  Stops |
| Background | `nohup purrgres ... > /dev/null 2>&1 &` |  Keeps running |
| Systemd | `systemctl start purrgres@mydb` |  Keeps running + auto-restarts |

### List Backups

```bash
purrgres --list-purrs
```

Output:

```
======================================================================
   backups                 |    date              | restore point
======================================================================
11_08_2026_03_00_backup.sql | 11/08/2026 03:00    | last
10_08_2026_03_00_backup.sql | 10/08/2026 03:00    |
09_08_2026_03_00_backup.sql | 09/08/2026 03:00    |
======================================================================
```

### Restore a Backup

```bash
purrgres --rpurry <BACKUP_FILENAME> -u <PG_USER> -d <DB_NAME> -c <CONTAINER>
```

Example:

```bash
purrgres --rpurry 11_08_2026_03_00_backup.sql -u postgres -d mydb -c my_container
```

This will:
1. Copy the backup file into the Docker container.
2. Run `psql` to apply it.
3. Log the restore in `~/.purrgres/.purrs`.

### Check Status

```bash
purrgres --stats
```

Output:

```
=== Status purrgres ===
Backup running: PID: 12345
Execution time: 2h 30m 15s
=========================
```

### Stop the Backup Process

```bash
purrgres --stop
```

---

## Usage — Server Mode (Receive Backups)

Server mode turns a machine into a **backup receiver**. It runs an HTTP API that accepts backup files from Purrgres clients.

```
┌─────────────────────┐        HTTP POST         ┌─────────────────────┐
│  Machine A (prod)   │  ──── backup.sql.gz ───► │  Machine B (vault)  │
│  purrgres (client)  │  ◄──── 200 OK ─────────  │  purrgres --server  │
└─────────────────────┘                           └─────────────────────┘
```

### Start the Server

**1. Create the config file** on the receiver machine:

```bash
mkdir -p ~/.purrgres
cat > ~/.purrgres/purrgres.toml << 'EOF'
[server]
port = 8443
storage_path = "/var/lib/purrgres/backups"
api_key = "your-secret-api-key"
max_upload_size_mb = 500
max_remote_backups = 30
EOF
```

**2. Create the storage directory:**

```bash
sudo mkdir -p /var/lib/purrgres/backups
sudo chown $USER:$USER /var/lib/purrgres/backups
```

**3. Start the server:**

```bash
purrgres --server
```

Output:

```
🐱 Purrgres server listening on 0.0.0.0:8443
   Storage: /var/lib/purrgres/backups
   Max upload: 500 MB
```

**4. For production, run in background or as a systemd service:**

```bash
nohup purrgres --server > /var/log/purrgres-server.log 2>&1 &
```

**Optional:** Override the port via CLI:

```bash
purrgres --server --port 9090
```

**Optional:** Use a custom config file:

```bash
purrgres --server --config /etc/purrgres/purrgres.toml
```

### API Endpoints

| Endpoint | Method | Auth Required | Description |
|---|---|---|---|
| `/api/health` | GET |  No | Health check — returns version and status |
| `/api/upload` | POST |  Yes | Upload a backup file (multipart form) |
| `/api/backups` | GET |  Yes | List all received backups |

**Authentication:** Send the API key in the `X-API-Key` header.

### Testing the Server

```bash
# Health check (no auth needed)
curl http://localhost:8443/api/health

# Upload a backup
echo "CREATE TABLE test;" > /tmp/test.sql
CHECKSUM=$(sha256sum /tmp/test.sql | awk '{print $1}')

curl -X POST http://localhost:8443/api/upload \
  -H "X-API-Key: your-secret-api-key" \
  -H "X-Checksum-SHA256: $CHECKSUM" \
  -F "file=@/tmp/test.sql"

# List received backups
curl -H "X-API-Key: your-secret-api-key" http://localhost:8443/api/backups
```

---

## Remote Backup Sync

When `[remote]` is enabled in the config, Purrgres will **automatically send each backup** to the remote server immediately after it's created.

### How It Works

```
1. pg_dump runs → saves backup.sql locally
2. Compresses → backup.sql.gz (if compress = true)
3. Computes SHA256 checksum
4. Sends to remote server via POST /api/upload
   - Includes X-API-Key header
   - Includes X-Checksum-SHA256 header
5. Server receives, verifies checksum, saves file
6. If it fails → retries up to 3 times (5s, 10s, 15s delays)
```

**Client config** (on the machine that makes backups):

```toml
[remote]
enabled = true
host = "192.168.1.50"
port = 8443
api_key = "your-secret-api-key"
compress = true
```

**Server config** (on the machine that receives backups):

```toml
[server]
port = 8443
storage_path = "/var/lib/purrgres/backups"
api_key = "your-secret-api-key"
max_upload_size_mb = 500
max_remote_backups = 30
```

### Checksum Verification

Every backup transfer includes a **SHA256 checksum** to ensure the file wasn't corrupted during transfer:

- The **client** calculates the SHA256 hash of the file before sending.
- The **server** recalculates the SHA256 hash after receiving.
- If they **don't match**, the server rejects the file with a `400 Checksum mismatch` error.
- The upload response includes `checksum_verified: true` on success.

This protects against network corruption, partial uploads, and tampered files — critical when dealing with database backups for disaster recovery.

---

## Retention Policies

Purrgres can automatically delete old backups to prevent disk space from filling up.

### Local Retention (Client Side)

Add to `~/.purrgres/purrgres.toml`:

```toml
[retention]
max_local_backups = 7    # Keep only the 7 most recent backups
```

After each backup, Purrgres will count `.sql` and `.sql.gz` files in `~/.purrgres/`. If there are more than `max_local_backups`, the oldest files are deleted automatically.

### Remote Retention (Server Side)

Add to the server's `purrgres.toml`:

```toml
[server]
# ... other settings ...
max_remote_backups = 30    # Keep only the 30 most recent backups
```

After each upload is received, the server will delete the oldest files if the count exceeds `max_remote_backups`.

### Example

With `max_local_backups = 3`, after one week of daily backups:

```
Day 1: backup_01.sql  ← deleted on Day 4
Day 2: backup_02.sql  ← deleted on Day 5
Day 3: backup_03.sql  ← deleted on Day 6
Day 4: backup_04.sql  ← deleted on Day 7
Day 5: backup_05.sql   kept
Day 6: backup_06.sql   kept
Day 7: backup_07.sql   kept
```

> **Tip:** If `[retention]` is not defined, no automatic cleanup occurs — all backups are kept indefinitely.

---

## Multi-Database Backups

If your Docker container hosts **multiple PostgreSQL databases**, you can use the included `run-multi.sh` script to orchestrate one Purrgres instance per database.

Each instance runs in the background with its own isolated `$HOME`, so PID files, logs, and backup history don't collide between databases.

### Config File Format

Create a config file listing your databases, one per line, in `database:user` format:

```bash
cp scripts/databases.conf.example databases.conf
nano databases.conf
```

```conf
# Format: database:user
# Lines starting with # are ignored. Empty lines are ignored.

analytics:app_user
billing:billing_user
main:postgres
```

### Running Multiple Instances

```bash
./scripts/run-multi.sh -c databases.conf
```

Output:

```
▶ Starting backup monitor for database [analytics] (user: app_user)...
▶ Starting backup monitor for database [billing] (user: billing_user)...
▶ Starting backup monitor for database [main] (user: postgres)...
--------------------------------------------------------
Started/checked 3 Purrgres instance(s) in background.
Backups will land in: /home/user/purrgres_backups/<database>/
Per-instance logs:    /home/user/purrgres_backups/<database>/.purrs/purrgres.log
--------------------------------------------------------
```

### Script Options

```bash
./scripts/run-multi.sh -c <config_file> [options]
```

| Flag | Description | Default |
|---|---|---|
| `-c, --config <file>` | Path to the database list file | **(required)** |
| `--container <name>` | Docker container name | `$PURRGRES_CONTAINER` or `postgres` |
| `--bin <path>` | Path to the `purrgres` binary | `$PURRGRES_BIN` or `purrgres` in PATH |
| `--backup-dir <path>` | Base directory for backups | `$PURRGRES_BKP_DIR` or `~/purrgres_backups` |
| `-h, --help` | Show help | — |

### Environment Variables

Instead of passing flags every time, you can set environment variables:

```bash
export PURRGRES_CONTAINER="my_postgres"
export PURRGRES_BIN="/usr/local/bin/purrgres"
export PURRGRES_BKP_DIR="/backups/purrgres"

./scripts/run-multi.sh -c databases.conf
```

### Directory Structure

After running, the backup directory will look like:

```
~/purrgres_backups/
├── analytics/
│   ├── .purrgres/
│   │   ├── 11_08_2026_03_00_backup.sql
│   │   └── 12_08_2026_03_00_backup.sql
│   └── .purrs/
│       ├── purrgres.log
│       └── purrgres.pid
├── billing/
│   ├── .purrgres/
│   │   └── ...
│   └── .purrs/
│       └── ...
└── main/
    └── ...
```

### Re-running the Script

The script is **idempotent** — if an instance is already running for a database, it will skip it:

```
Instance for [analytics] already running (PID 12345). Skipping.
▶ Starting backup monitor for database [billing] (user: billing_user)...
```

---

## Running as a Systemd Service

A systemd service template is included for running Purrgres as a managed service.

### Setup

```bash
# Copy the service file
sudo cp systemd/purrgres@.service /etc/systemd/system/

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

> **Note:** You may need to edit the service file to set the correct `User`, `ExecStart` path, and environment variables for your setup.

### Running the Server as a Service

You can also run the server mode as a systemd service. Create `/etc/systemd/system/purrgres-server.service`:

```ini
[Unit]
Description=Purrgres Backup Receiver Server
After=network.target

[Service]
Type=simple
User=purrgres
ExecStart=/usr/local/bin/purrgres --server --config /etc/purrgres/purrgres.toml
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now purrgres-server
```

---

## CLI Reference

```
purrgres [OPTIONS]
```

| Flag | Short | Description | Required |
|---|---|---|---|
| `--user <USER>` | `-u` | PostgreSQL user | Yes (client mode) |
| `--database <DB>` | `-d` | Database name | Yes (client mode) |
| `--container <NAME>` | `-c` | Docker container name | Yes (client mode) |
| `--stats` | | Show status of running backup | No |
| `--stop` | | Stop the running backup process | No |
| `--list-purrs` | | List all performed backups | No |
| `--rpurry <FILE>` | | Restore a backup from file | No |
| `--server` | | Start in server mode (API) | No |
| `--config <PATH>` | | Path to `purrgres.toml` | No |
| `--port <PORT>` | | Override server port (requires `--server`) | No |
| `--version` | | Show version | No |
| `--help` | `-h` | Show help | No |

### Examples

```bash
# Basic backup
purrgres -u postgres -d mydb -c my_container

# Background backup
nohup purrgres -u postgres -d mydb -c my_container > /dev/null 2>&1 &

# Check status
purrgres --stats

# List backups
purrgres --list-purrs

# Restore
purrgres --rpurry 11_08_2026_03_00_backup.sql -u postgres -d mydb -c my_container

# Stop
purrgres --stop

# Start server
purrgres --server

# Server with custom config and port
purrgres --server --config /etc/purrgres/purrgres.toml --port 9090
```

---

## Semantic Versioning

This project follows [Semantic Versioning (SemVer)](https://semver.org/):

| Part | When it changes | Example |
|---|---|---|
| **MAJOR** (X.0.0) | Incompatible/breaking API changes | `1.0.0` → `2.0.0` |
| **MINOR** (0.X.0) | New features, backwards compatible | `1.0.0` → `1.1.0` |
| **PATCH** (0.0.X) | Bug fixes, no API changes | `1.0.0` → `1.0.1` |

---

## Contributing

Contributions are welcome! Feel free to open pull requests or report issues.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -m 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request

Please follow coding best practices and provide clear descriptions of your changes.

---

## License

This project is licensed under the [MIT License](./LICENSE) — see the LICENSE file for details.