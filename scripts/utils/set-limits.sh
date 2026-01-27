#!/usr/bin/env bash

# Define container limits in an associative array
declare -A container_limits=(
    # Pattern       Memory    CPU Allowance
     ["*tables*"]="4096MB:100ms/100ms"
     ["*postgre*"]="4096MB:100ms/100ms"    # PostgreSQL alternative
     ["*dns*"]="2048MB:100ms/100ms"
     ["*oppbot*"]="4048MB:100ms/100ms"
     ["*table-editor*"]="2048MB:25ms/100ms"
     ["*proxy*"]="2048MB:100ms/100ms"
     ["*directory*"]="1024MB:50ms/100ms"
     ["*drive*"]="4096MB:100ms/100ms"
     ["*minio*"]="4096MB:100ms/100ms"      # MinIO alternative
     ["*email*"]="4096MB:100ms/100ms"
     ["*webmail*"]="2096MB:100ms/100ms"
     ["*bot*"]="2048MB:25ms/100ms"
     ["*oppbot*"]="2048MB:50ms/100ms"
     ["*meeting*"]="4096MB:100ms/100ms"
     ["*alm*"]="512MB:50ms/100ms"
     ["*vault*"]="512MB:50ms/100ms"
     ["*alm-ci*"]="8192MB:200ms/100ms"     # CHANGED: 100ms → 200ms (HIGHEST PRIORITY)
     ["*system*"]="4096MB:50ms/100ms"
     ["*mailer*"]="2096MB:25ms/100ms"
)

# Default values (for containers that don't match any pattern)
DEFAULT_MEMORY="512MB"
DEFAULT_CPU_ALLOWANCE="15ms/100ms"
DEFAULT_CPU_COUNT=1

# PRIORITY LEVELS (1-10, where 10 is highest)
declare -A container_priority=(
    ["*alm-ci*"]="10"           # HIGHEST PRIORITY - all sources for alm-ci
    ["*tables*"]="9"            # High priority - PostgreSQL
    ["*postgre*"]="9"           # High priority - PostgreSQL alternative
    ["*drive*"]="8"             # High priority - MinIO
    ["*minio*"]="8"             # High priority - MinIO alternative
    ["*directory*"]="7"         # Medium priority - Zitadel
    ["*dns*"]="5"               # Normal priority
    ["*proxy*"]="5"             # Normal priority
    ["*email*"]="5"             # Normal priority
    ["*webmail*"]="5"           # Normal priority
    ["*meeting*"]="5"           # Normal priority
    ["*system*"]="5"            # Normal priority
    ["*bot*"]="4"               # Lower priority
    ["*doc-editor*"]="4"        # Lower priority
    ["*alm*"]="4"               # Lower priority
    ["*mailer*"]="3"            # Lowest priority
)

# Get all containers once
containers=$(lxc list -c n --format csv)

echo "Starting container configuration with priority levels..."
echo "========================================================="

# Configure all containers
for container in $containers; do
    echo "Configuring $container..."

    memory=$DEFAULT_MEMORY
    cpu_allowance=$DEFAULT_CPU_ALLOWANCE
    cpu_count=$DEFAULT_CPU_COUNT
    cpu_priority=5  # Default priority if not specified

    # Check if container matches any pattern
    matched_pattern=""
    for pattern in "${!container_limits[@]}"; do
        # Convert pattern to regex: *tables* -> .*tables.*
        regex_pattern="${pattern//\*/.*}"
        if [[ $container =~ $regex_pattern ]]; then
            IFS=':' read -r memory cpu_allowance <<< "${container_limits[$pattern]}"
            matched_pattern=$pattern
            echo "  → Matched pattern: $pattern"

            # Set CPU count based on service type
            if [[ $pattern == "*alm-ci*" ]]; then
                cpu_count=2  # More CPUs for alm-ci
            elif [[ $pattern == "*tables*" ]] || [[ $pattern == "*postgre*" ]]; then
                cpu_count=1  # More CPUs for PostgreSQL
            elif [[ $pattern == "*drive*" ]] || [[ $pattern == "*minio*" ]]; then
                cpu_count=1  # More CPUs for MinIO
            else
                cpu_count=1
            fi
            break
        fi
    done

    # Set CPU priority if defined for this pattern
    if [ -n "$matched_pattern" ] && [ -n "${container_priority[$matched_pattern]}" ]; then
        cpu_priority="${container_priority[$matched_pattern]}"
    else
        # Set priority for all other containers (balanced for 10 users)
        cpu_priority=5
    fi

    # Apply configuration
    echo "  → Memory: $memory"
    echo "  → CPU: $cpu_count cores"
    echo "  → CPU Allowance: $cpu_allowance"
    echo "  → CPU Priority: $cpu_priority/10"

    lxc config set "$container" limits.memory "$memory"
    lxc config set "$container" limits.cpu.allowance "$cpu_allowance"
    lxc config set "$container" limits.cpu "$cpu_count"
    lxc config set "$container" limits.cpu.priority "$cpu_priority"

    echo "  → Restarting $container..."
    lxc restart "$container" --timeout=30

    echo "  → Current config:"
    lxc config show "$container" | grep -E "memory|cpu" | sed 's/^/    /'
    echo ""
done

echo "========================================================="
echo "Configuration complete!"
echo ""
echo "PRIORITY SUMMARY:"
echo "1. alm-ci (build-run) → Priority 10/10, 200ms/100ms, 8GB RAM, 4 CPUs"
echo "2. tables/postgre → Priority 9/10, 100ms/100ms, 4GB RAM, 4 CPUs"
echo "3. drive/minio → Priority 8/10, 100ms/100ms, 4GB RAM, 3 CPUs"
echo "4. directory → Priority 7/10, 50ms/100ms, 1GB RAM, 2 CPUs"
echo "5. All others → Priority 5/10, default values (balanced for 10 users)"
echo "========================================================="
