#!/bin/bash
#
# run-multi.sh — Orchestrates multiple Purrgres instances, one per database.
#
# Useful when a single Docker container hosts several databases and you want
# an isolated backup process (and isolated .purrs history) for each one.
#
# Usage:
#   ./run-multi.sh -c my-databases.conf
#   PURRGRES_BIN=/opt/purrgres ./run-multi.sh -c my-databases.conf
#
# Config file format (one database per line, comments with # allowed):
#   # database:user
#   analytics:app_user
#   billing:billing_user
#
# Each instance runs in background (nohup) with its own $HOME, so Purrgres'
# .purrs state directory doesn't collide between databases.

set -euo pipefail

usage() {
    cat <<EOF
Usage: $(basename "$0") -c <config_file> [options]

Required:
  -c, --config <file>       Path to the database list (name:user per line)

Options:
  --container <name>        Docker container name (default: \$PURRGRES_CONTAINER or "postgres")
  --bin <path>               Path to the purrgres binary (default: \$PURRGRES_BIN or "purrgres" in PATH)
  --backup-dir <path>        Base directory for backups (default: \$PURRGRES_BKP_DIR or "\$HOME/purrgres_backups")
  -h, --help                  Show this help

Environment variables (used as defaults if flags are not given):
  PURRGRES_CONTAINER, PURRGRES_BIN, PURRGRES_BKP_DIR
EOF
    exit 1
}

CONFIG_FILE=""
CONTAINER="${PURRGRES_CONTAINER:-postgres}"
PURRGRES_BIN="${PURRGRES_BIN:-purrgres}"
BASE_BKP_DIR="${PURRGRES_BKP_DIR:-$HOME/purrgres_backups}"

while [ $# -gt 0 ]; do
    case "$1" in
        -c|--config) CONFIG_FILE="$2"; shift 2 ;;
        --container) CONTAINER="$2"; shift 2 ;;
        --bin) PURRGRES_BIN="$2"; shift 2 ;;
        --backup-dir) BASE_BKP_DIR="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "Unknown argument: $1"; usage ;;
    esac
done

if [ -z "$CONFIG_FILE" ]; then
    echo "❌ Error: config file is required (-c <file>)"
    usage
fi

if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Error: config file not found: $CONFIG_FILE"
    exit 1
fi

check_requirements() {
    if ! command -v "$PURRGRES_BIN" >/dev/null 2>&1 && [ ! -x "$PURRGRES_BIN" ]; then
        echo "❌ Error: purrgres binary not found or not executable: $PURRGRES_BIN"
        echo "   Set --bin <path> or the PURRGRES_BIN environment variable."
        exit 1
    fi

    if ! command -v docker >/dev/null 2>&1; then
        echo "❌ Error: docker command not found."
        exit 1
    fi

    if ! docker ps --format '{{.Names}}' | grep -qx "$CONTAINER"; then
        echo "❌ Error: container '$CONTAINER' is not running."
        echo "   Set --container <name> or the PURRGRES_CONTAINER environment variable."
        exit 1
    fi
}

start_instance() {
    local db_name="$1"
    local db_user="$2"
    local instance_home="$BASE_BKP_DIR/$db_name"
    local state_dir="$instance_home/.purrs"
    local pid_file="$state_dir/purrgres.pid"

    mkdir -p "$state_dir"

    if [ -f "$pid_file" ] && kill -0 "$(cat "$pid_file")" 2>/dev/null; then
        echo "⚠️  Instance for [$db_name] already running (PID $(cat "$pid_file")). Skipping."
        return
    fi

    echo "▶ Starting backup monitor for database [$db_name] (user: $db_user)..."
    env HOME="$instance_home" nohup "$PURRGRES_BIN" \
        --container "$CONTAINER" \
        --user "$db_user" \
        --database "$db_name" > "$state_dir/purrgres.log" 2>&1 &

    echo $! > "$pid_file"
}

main() {
    check_requirements

    local count=0
    while IFS= read -r line || [ -n "$line" ]; do
        # Skip empty lines and comments
        [ -z "$line" ] && continue
        case "$line" in \#*) continue ;; esac

        case "$line" in
            *:*) ;;
            *)
                echo "⚠️  Skipping malformed line (expected 'database:user'): $line"
                continue
                ;;
        esac

        db_name="${line%%:*}"
        db_user="${line##*:}"

        if [ -z "$db_name" ] || [ -z "$db_user" ]; then
            echo "⚠️  Skipping malformed line (expected 'database:user'): $line"
            continue
        fi

        start_instance "$db_name" "$db_user"
        count=$((count + 1))
    done < "$CONFIG_FILE"

    echo "--------------------------------------------------------"
    echo "Started/checked $count Purrgres instance(s) in background."
    echo "Backups will land in: $BASE_BKP_DIR/<database>/"
    echo "Per-instance logs:    $BASE_BKP_DIR/<database>/.purrs/purrgres.log"
    echo "--------------------------------------------------------"
}

main