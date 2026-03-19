#!/bin/bash
#
# Migrate LXD containers from ZFS to Incus directory storage
# Usage: ./migrate_zfs_to_incus_dir.sh [options]
#
# Options:
#   --dry-run          Show what would be done without executing
#   --source-pool NAME LXD storage pool name (default: default)
#   --dest-pool NAME   Incus storage pool name (default: default)
#   --containers LIST  Comma-separated list of containers (default: all)
#   --incus-host HOST  Remote Incus host (optional)
#   --help             Show this help

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Defaults
DRY_RUN=false
SOURCE_POOL="default"
DEST_POOL="default"
CONTAINERS=""
INCUS_HOST=""
BACKUP_DIR="/tmp/lxd-backups"

# Print banner
print_banner() {
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║  LXD to Incus Migration Tool (ZFS -> Directory Storage)    ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Show help
show_help() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Migrate LXD containers from ZFS storage to Incus directory storage."
    echo ""
    echo "Options:"
    echo "  --dry-run          Show what would be done without executing"
    echo "  --source-pool NAME LXD storage pool name (default: default)"
    echo "  --dest-pool NAME   Incus storage pool name (default: default)"
    echo "  --containers LIST  Comma-separated list of containers (default: all)"
    echo "  --incus-host HOST  Remote Incus host (ssh)"
    echo "  --help             Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 --dry-run"
    echo "  $0 --source-pool zfs-storage --dest-pool dir-storage"
    echo "  $0 --containers container1,container2 --incus-host user@remote-host"
}

# Check dependencies
check_dependencies() {
    local missing=()
    if ! command -v lxc &> /dev/null; then
        missing+=("lxc")
    fi
    if ! command -v incus &> /dev/null; then
        missing+=("incus")
    fi
    if [ ${#missing[@]} -gt 0 ]; then
        echo -e "${RED}Missing dependencies: ${missing[*]}${NC}"
        echo "Please install them and try again."
        exit 1
    fi
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --source-pool)
            SOURCE_POOL="$2"
            shift 2
            ;;
        --dest-pool)
            DEST_POOL="$2"
            shift 2
            ;;
        --containers)
            CONTAINERS="$2"
            shift 2
            ;;
        --incus-host)
            INCUS_HOST="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Execute command (dry-run aware)
exec_cmd() {
    local cmd="$1"
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY-RUN] $cmd${NC}"
    else
        echo -e "${GREEN}[EXEC] $cmd${NC}"
        eval "$cmd"
    fi
}

# Main migration logic
migrate_container() {
    local container="$1"
    local backup_file="$BACKUP_DIR/${container}-$(date +%Y%m%d%H%M%S).tar.gz"

    echo -e "${BLUE}Migrating container: $container${NC}"

    # Create backup directory
    exec_cmd "mkdir -p $BACKUP_DIR"

    # Create snapshot
    local snapshot_name="migrate-$(date +%Y%m%d%H%M%S)"
    exec_cmd "lxc snapshot $container $snapshot_name --stateful"

    # Export container
    exec_cmd "lxc export $container $backup_file --snapshot $snapshot_name"

    # Transfer to remote Incus host if specified
    if [ -n "$INCUS_HOST" ]; then
        exec_cmd "scp $backup_file $INCUS_HOST:$BACKUP_DIR/"
        exec_cmd "rm $backup_file"
        backup_file="$BACKUP_DIR/$(basename $backup_file)"
    fi

    # Import into Incus
    local import_cmd="incus import $backup_file $container"
    if [ -n "$INCUS_HOST" ]; then
        import_cmd="ssh $INCUS_HOST '$import_cmd'"
    fi
    exec_cmd "$import_cmd"

    # Cleanup snapshot
    exec_cmd "lxc delete $container/$snapshot_name"

    # Cleanup local backup file if not remote
    if [ -z "$INCUS_HOST" ] && [ "$DRY_RUN" = false ]; then
        rm "$backup_file"
    fi

    echo -e "${GREEN}Completed migration for: $container${NC}"
}

# Main
print_banner
check_dependencies

# Get list of containers
if [ -z "$CONTAINERS" ]; then
    CONTAINERS=$(lxc list --format csv | cut -d',' -f1)
fi

# Convert comma-separated list to array
IFS=',' read -ra CONTAINER_ARRAY <<< "$CONTAINERS"

# Migrate each container
for container in "${CONTAINER_ARRAY[@]}"; do
    migrate_container "$container"
done

echo -e "${GREEN}Migration complete!${NC}"
