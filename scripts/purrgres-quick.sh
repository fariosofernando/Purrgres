#!/bin/bash
#
# purrgres-quick.sh — Quick status and management commands for Purrgres instances
#
# A simplified interface for common Purrgres operations without needing
# to remember the HOME variable path structure.
#
# Usage:
#   ./purrgres-quick.sh status <db>
#   ./purrgres-quick.sh list <db>
#   ./purrgres-quick.sh logs <db>
#   ./purrgres-quick.sh restore <db> <backup_file> <user>
#   ./purrgres-quick.sh stop <db>
#   ./purrgres-quick.sh summary
#

set -euo pipefail
 
BACKUP_DIR="${PURRGRES_BKP_DIR:-$HOME/purrgres_backups}"
PURRGRES_BIN="${PURRGRES_BIN:-purrgres}"
CONTAINER="${PURRGRES_CONTAINER:-postgres}"

# ─────────────────────────────────────────────────
# Utility Functions
# ─────────────────────────────────────────────────

show_help() {
    cat << EOF
purrgres-quick.sh — Quick Purrgres instance management

Usage:
  ./purrgres-quick.sh <command> [args]

Commands:

  status <database>              Show backup status for a database
  list <database>                List backups for a database
  logs <database>                Show real-time logs (Ctrl+C to exit)
  logs-tail <database> [lines]   Show last N lines of logs (default: 50)
  running                        List all running instances
  restore <db> <backup> <user>   Restore a backup to a database
  stop <database>                Stop a database instance
  stop-all                       Stop all instances
  is-running <database>          Check if database is running
  summary                        Show summary of all instances
  help                           Show this help message

Configuration:
  Set environment variables to override defaults:
    BACKUP_DIR         Base backup directory (default: ~/purrgres_backups)
    PURRGRES_BIN       Path to purrgres binary (default: purrgres)
    PURRGRES_CONTAINER Docker container name (default: postgres)

Examples:
  ./purrgres-quick.sh status analytics
  ./purrgres-quick.sh list auth
  ./purrgres-quick.sh logs invoice
  ./purrgres-quick.sh restore auth auth_12_08_2026_22_02_backup.sql ivy
  ./purrgres-quick.sh summary
  BACKUP_DIR=/custom/path ./purrgres-quick.sh status analytics

EOF
}

error() {
    echo " Error: $*" >&2
    exit 1
}

validate_db() {
    local db=$1
    if [ -z "$db" ]; then
        error "database name is required"
    fi
    if [ ! -d "$BACKUP_DIR/$db" ]; then
        error "database directory not found: $BACKUP_DIR/$db"
    fi
} 

cmd_status() {
    local db=$1
    validate_db "$db"
    echo " Status for: $db"
    echo ""
    HOME="$BACKUP_DIR/$db" $PURRGRES_BIN --stats
}

cmd_list() {
    local db=$1
    validate_db "$db"
    echo " Backups for: $db"
    echo ""
    HOME="$BACKUP_DIR/$db" $PURRGRES_BIN --list-purrs
}

cmd_logs() {
    local db=$1
    validate_db "$db"
    local log_file="$BACKUP_DIR/$db/.purrs/purrgres.log"
    if [ ! -f "$log_file" ]; then
        error "log file not found: $log_file"
    fi
    echo " Live logs for: $db (Ctrl+C to exit)"
    echo ""
    tail -f "$log_file"
}

cmd_logs_tail() {
    local db=$1
    local lines=${2:-50}
    validate_db "$db"
    local log_file="$BACKUP_DIR/$db/.purrs/purrgres.log"
    if [ ! -f "$log_file" ]; then
        error "log file not found: $log_file"
    fi
    echo " Last $lines lines of logs for: $db"
    echo ""
    tail -n "$lines" "$log_file"
}

cmd_running() {
    echo " Running Purrgres instances:"
    echo ""
    pgrep -fa "purrgres --container $CONTAINER" | sed 's/^/  /' || echo "  (none)"
}

cmd_restore() {
    local db=$1
    local backup_file=$2
    local db_user=$3
    
    if [ -z "$db" ] || [ -z "$backup_file" ] || [ -z "$db_user" ]; then
        error "restore requires: <database> <backup_file> <user>"
    fi
    
    validate_db "$db"
    
    local backup_path="$BACKUP_DIR/$db/.purrgres/$backup_file"
    if [ ! -f "$backup_path" ]; then
        error "backup file not found: $backup_path"
    fi
    
    echo " Restoring $backup_file to database $db..."
    echo ""
    HOME="$BACKUP_DIR/$db" $PURRGRES_BIN \
        --rpurry "$backup_file" \
        -u "$db_user" \
        -d "$db" \
        -c "$CONTAINER"
}

cmd_stop() {
    local db=$1
    if [ -z "$db" ]; then
        error "database name is required"
    fi
    
    if pkill -f "purrgres.*--database $db"; then
        echo " Stopped: $db"
    else
        echo "  No running process found for: $db"
        return 1
    fi
}

cmd_stop_all() {
    echo " Stopping all Purrgres instances for container: $CONTAINER"
    if pkill -f "purrgres --container $CONTAINER"; then
        echo " All instances stopped"
        sleep 2
    else
        echo "  No running processes found"
        return 1
    fi
}

cmd_is_running() {
    local db=$1
    validate_db "$db"
    
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

cmd_summary() {
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
        error "backup directory not found: $BACKUP_DIR"
    fi
    
    local db_count=0
    local running_count=0
    
    for db in $(ls -1 "$BACKUP_DIR" 2>/dev/null | grep -v "^\."); do
        db_count=$((db_count + 1))
        
        echo "───────────────────────────────────────────────────────────────"
        echo " Database: $db"
         
        local pid_file="$BACKUP_DIR/$db/.purrs/purrgres.pid"
        if [ -f "$pid_file" ]; then
            local pid=$(cat "$pid_file")
            if kill -0 "$pid" 2>/dev/null; then
                echo "   Status:  Running (PID $pid)"
                running_count=$((running_count + 1))
            else
                echo "   Status:  Not Running (stale PID)"
            fi
        else
            echo "   Status:  Not Running"
        fi
         
        local last_backup=$(ls -t "$BACKUP_DIR/$db/.purrgres"/*.sql 2>/dev/null | head -1)
        if [ -n "$last_backup" ]; then
            local backup_name=$(basename "$last_backup")
            local backup_size=$(ls -lh "$last_backup" | awk '{print $5}')
            echo "   Last backup: $backup_name ($backup_size)"
        else
            echo "   Last backup: (none)"
        fi
         
        local backup_count=$(ls -1 "$BACKUP_DIR/$db/.purrgres"/*.sql 2>/dev/null | wc -l)
        echo "   Total backups: $backup_count"
    done
    
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "Summary: $running_count / $db_count instances running"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
}
 
main() {
    local cmd="${1:-}"
    
    if [ -z "$cmd" ] || [ "$cmd" = "help" ] || [ "$cmd" = "-h" ] || [ "$cmd" = "--help" ]; then
        show_help
        exit 0
    fi
    
    case "$cmd" in
        status)    cmd_status "$2" ;;
        list)      cmd_list "$2" ;;
        logs)      cmd_logs "$2" ;;
        logs-tail) cmd_logs_tail "$2" "$3" ;;
        running)   cmd_running ;;
        restore)   cmd_restore "$2" "$3" "$4" ;;
        stop)      cmd_stop "$2" ;;
        stop-all)  cmd_stop_all ;;
        is-running) cmd_is_running "$2" ;;
        summary)   cmd_summary ;;
        *)
            echo "Unknown command: $cmd" >&2
            echo ""
            show_help
            exit 1
            ;;
    esac
}

main "$@"
