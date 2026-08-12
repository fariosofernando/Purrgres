#!/bin/bash
#
# purrgres-helpers.sh — Helper functions for managing multiple Purrgres instances
#
# Utilities for monitoring, checking status, restoring backups, and managing
# isolated Purrgres instances created by run-multi.sh
#
# Usage:
#   source ./purrgres-helpers.sh
#   purrgres_status analytics
#   purrgres_list auth
#   purrgres_logs invoice
#   purrgres_summary
#
# Configuration:
#   Edit BACKUP_DIR, PURRGRES_BIN, and CONTAINER below to match your setup,
#   or export them as environment variables before sourcing this script.

set -euo pipefail
 
BACKUP_DIR="${PURRGRES_BKP_DIR:-$HOME/purrgres_backups}"
PURRGRES_BIN="${PURRGRES_BIN:-purrgres}"
CONTAINER="${PURRGRES_CONTAINER:-postgres}"

# ─────────────────────────────────────────────────
# Helper Functions
# ─────────────────────────────────────────────────
 
purrgres_list_running() {
    echo "Running Purrgres instances:"
    pgrep -fa "purrgres --container $CONTAINER" || echo "  (none)"
}
 
purrgres_status() {
    local db=$1
    if [ -z "$db" ]; then
        echo "Usage: purrgres_status <database>"
        return 1
    fi
    
    if [ ! -d "$BACKUP_DIR/$db" ]; then
        echo " Database directory not found: $BACKUP_DIR/$db"
        return 1
    fi
    
    HOME="$BACKUP_DIR/$db" $PURRGRES_BIN --stats
}
 
purrgres_list() {
    local db=$1
    if [ -z "$db" ]; then
        echo "Usage: purrgres_list <database>"
        return 1
    fi
    
    if [ ! -d "$BACKUP_DIR/$db" ]; then
        echo " Database directory not found: $BACKUP_DIR/$db"
        return 1
    fi
    
    HOME="$BACKUP_DIR/$db" $PURRGRES_BIN --list-purrs
}
 
purrgres_logs() {
    local db=$1
    if [ -z "$db" ]; then
        echo "Usage: purrgres_logs <database>"
        return 1
    fi
    
    local log_file="$BACKUP_DIR/$db/.purrs/purrgres.log"
    if [ ! -f "$log_file" ]; then
        echo " Log file not found: $log_file"
        return 1
    fi
    
    tail -f "$log_file"
}
 
purrgres_logs_tail() {
    local db=$1
    local lines=${2:-50}
    
    if [ -z "$db" ]; then
        echo "Usage: purrgres_logs_tail <database> [lines]"
        return 1
    fi
    
    local log_file="$BACKUP_DIR/$db/.purrs/purrgres.log"
    if [ ! -f "$log_file" ]; then
        echo " Log file not found: $log_file"
        return 1
    fi
    
    tail -n "$lines" "$log_file"
}
 
purrgres_restore() {
    local db=$1
    local backup_file=$2
    local db_user=$3
    
    if [ -z "$db" ] || [ -z "$backup_file" ] || [ -z "$db_user" ]; then
        echo "Usage: purrgres_restore <database> <backup_file> <db_user>"
        echo "Example: purrgres_restore auth auth_12_08_2026_22_02_backup.sql ivy"
        return 1
    fi
    
    local backup_path="$BACKUP_DIR/$db/.purrgres/$backup_file"
    if [ ! -f "$backup_path" ]; then
        echo " Backup file not found: $backup_path"
        echo ""
        echo "Available backups for $db:"
        ls -1 "$BACKUP_DIR/$db/.purrgres"/*.sql 2>/dev/null || echo "  (none)"
        return 1
    fi
    
    echo " Restoring $backup_file to database $db..."
    HOME="$BACKUP_DIR/$db" $PURRGRES_BIN \
        --rpurry "$backup_file" \
        -u "$db_user" \
        -d "$db" \
        -c "$CONTAINER"
}
 
purrgres_stop() {
    local db=$1
    if [ -z "$db" ]; then
        echo "Usage: purrgres_stop <database>"
        return 1
    fi
    
    if pkill -f "purrgres.*--database $db"; then
        echo " Stopped: $db"
    else
        echo "  No running process found for: $db"
        return 1
    fi
}
 
purrgres_stop_all() {
    echo " Stopping all Purrgres instances for container: $CONTAINER"
    if pkill -f "purrgres --container $CONTAINER"; then
        echo " All instances stopped"
    else
        echo "  No running processes found"
        return 1
    fi
}
 
purrgres_all_dbs() {
    echo "Databases being backed up:"
    if [ -d "$BACKUP_DIR" ]; then
        ls -1 "$BACKUP_DIR" | grep -v "^\." || echo "  (none)"
    else
        echo "   Backup directory not found: $BACKUP_DIR"
        return 1
    fi
}
 
purrgres_is_running() {
    local db=$1
    if [ -z "$db" ]; then
        echo "Usage: purrgres_is_running <database>"
        return 1
    fi
    
    local pid_file="$BACKUP_DIR/$db/.purrs/purrgres.pid"
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            echo " $db is running (PID $pid)"
            return 0
        else
            echo " $db is NOT running (stale PID $pid)"
            return 1
        fi
    else
        echo " $db is NOT running (no PID file)"
        return 1
    fi
}
 
purrgres_summary() {
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "  Purrgres Multi-Instance Summary"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
    echo "Container: $CONTAINER"
    echo "Backup Directory: $BACKUP_DIR"
    echo "Purrgres Binary: $PURRGRES_BIN"
    echo ""
    
    if [ ! -d "$BACKUP_DIR" ]; then
        echo " Backup directory not found: $BACKUP_DIR"
        return 1
    fi
    
    local db_count=0
    local running_count=0
    
    for db in $(ls -1 "$BACKUP_DIR" 2>/dev/null | grep -v "^\."); do
        db_count=$((db_count + 1))
        
        echo "───────────────────────────────────────────────────────────────"
        echo "Database: $db"
         
        local pid_file="$BACKUP_DIR/$db/.purrs/purrgres.pid"
        if [ -f "$pid_file" ]; then
            local pid=$(cat "$pid_file")
            if kill -0 "$pid" 2>/dev/null; then
                echo "  Status:  Running (PID $pid)"
                running_count=$((running_count + 1))
            else
                echo "  Status:  Not Running (stale PID)"
            fi
        else
            echo "  Status:  Not Running (no PID file)"
        fi
         
        local last_backup=$(ls -t "$BACKUP_DIR/$db/.purrgres"/*.sql 2>/dev/null | head -1)
        if [ -n "$last_backup" ]; then
            local backup_name=$(basename "$last_backup")
            local backup_size=$(ls -lh "$last_backup" | awk '{print $5}')
            local backup_date=$(stat -c %y "$last_backup" 2>/dev/null | cut -d' ' -f1-2 || stat -f %Sm "$last_backup" 2>/dev/null || echo "unknown")
            echo "  Last backup: $backup_name"
            echo "  Size: $backup_size"
            echo "  Date: $backup_date"
        else
            echo "  Last backup: (none)"
        fi
         
        local backup_count=$(ls -1 "$BACKUP_DIR/$db/.purrgres"/*.sql 2>/dev/null | wc -l)
        echo "  Total backups: $backup_count"
    done
    
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "Summary: $running_count / $db_count instances running"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
}
 
purrgres_restart() {
    local db=$1
    if [ -z "$db" ]; then
        echo "Usage: purrgres_restart <database>"
        return 1
    fi
    
    echo " Restarting $db..."
    purrgres_stop "$db" || true
    sleep 2
     
    local config_file="$BACKUP_DIR/$db/.purrgres/purrgres.toml"
    if [ ! -f "$config_file" ]; then
        echo " Config file not found: $config_file"
        return 1
    fi
    
    echo "  Manual restart required - re-run run-multi.sh or manually start:"
    echo "   HOME=$BACKUP_DIR/$db $PURRGRES_BIN --container $CONTAINER --database $db --user <user>"
}
 
export -f purrgres_list_running purrgres_status purrgres_list purrgres_logs purrgres_logs_tail
export -f purrgres_restore purrgres_stop purrgres_stop_all purrgres_all_dbs purrgres_is_running
export -f purrgres_summary purrgres_restart
 
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    cat << EOF
purrgres-helpers.sh — Helper functions for managing multiple Purrgres instances

Usage:
  source ./purrgres-helpers.sh
  purrgres_<command> [args]

Available Commands:

  purrgres_list_running            List all running instances for this container
  purrgres_status <db>             Show backup status for a database
  purrgres_list <db>               List all backups for a database
  purrgres_logs <db>               Show real-time logs for a database
  purrgres_logs_tail <db> [lines]  Show last N lines of logs (default: 50)
  purrgres_restore <db> <file> <user>
                                   Restore a backup to a database
  purrgres_stop <db>               Stop a specific database instance
  purrgres_stop_all                Stop all instances for this container
  purrgres_all_dbs                 List all configured databases
  purrgres_is_running <db>         Check if a database instance is running
  purrgres_summary                 Show summary of all instances
  purrgres_restart <db>            Restart a database instance

Configuration:
  Set these environment variables to customize:
    BACKUP_DIR        Base directory for backups (default: ~/purrgres_backups)
    PURRGRES_BIN      Path to purrgres binary (default: purrgres)
    PURRGRES_CONTAINER Docker container name (default: postgres)

Examples:
  purrgres_status analytics
  purrgres_list auth
  purrgres_logs invoice
  purrgres_logs_tail wallet 100
  purrgres_restore auth auth_12_08_2026_22_02_backup.sql ivy
  purrgres_stop analytics
  purrgres_summary
  purrgres_is_running piminder

EOF
    exit 0
fi
