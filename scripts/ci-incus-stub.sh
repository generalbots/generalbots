#!/usr/bin/env bash
# CI-ONLY TEST DOUBLE for the `incus` CLI.
#
# This file implements the tiny subset of Incus that `botagent` uses so the
# real chat/terminal consent flow can run on a GitHub-hosted runner (which has
# no Incus daemon). It is installed on PATH as `incus` by
# .github/workflows/ci.yml (suite-app-console-smoke) and MUST NEVER be installed
# on a developer machine, a server, or shipped in any package. It is not a
# container runtime: `exec` runs the command locally under a throwaway fake
# root (GB_CI_INCUS_ROOT), and the state-changing commands are no-ops.
#
# Supported subset:
#   launch <image> <name> [--config k=v ...]  record <name>, create fake root
#   list [--format json]                     report <name> as RUNNING
#   image list --format json                 report no cached images
#   info <name> --show-log                   empty log
#   exec <name> [--cwd DIR] -- sh -c CMD     run CMD locally under fake root
#   config|start|stop|delete|wait|...        no-op, exit 0
#   self-test                                offline proof of the above
#
# Safety: provisioning shell scripts (which would write /etc/resolv.conf, run
# apt-get/useradd/iptables on the host) are detected and skipped. Absolute
# guest paths under /opt, /tmp, /etc, /home and /usr/local are rewritten under
# the fake root before any command/script is executed.

set -uo pipefail

ROOT="${GB_CI_INCUS_ROOT:-/tmp/gb-ci-incus-root}"
STATE="$ROOT/.ci-incus-stub-vm"

log() { printf '[ci-incus-stub] %s\n' "$*" >&2; }

init_root() {
    mkdir -p "$ROOT/opt/vibe/app/data" "$ROOT/opt/vibe/app/output" \
        "$ROOT/opt/vibe/conversations" || return 1
    return 0
}

# Rewrite guest absolute paths so nothing can escape the fake root.
rewrite_path() {
    local p="$1"
    case "$p" in
        /opt/*|/tmp/*|/etc/*|/home/*|/usr/local/*) printf '%s%s' "$ROOT" "$p" ;;
        *) printf '%s' "$p" ;;
    esac
}

rewrite_script() {
    printf '%s' "$1" | sed -E "s#(^|[ '\"=])/(opt|tmp|etc|home|usr/local)/#\1$ROOT/\2/#g"
}

is_host_mutating_script() {
    local script="$1"
    case "$script" in
        *agent-bootstrap.log*|*agent-prepare.log*|*DEBIAN_FRONTEND*|*.provisioned*|*useradd*|*iptables*) return 0 ;;
        *) return 1 ;;
    esac
}

cmd_launch() {
    local name="${2:-}"
    if [ -z "$name" ]; then
        log "launch needs an instance name"
        return 1
    fi
    init_root || return 1
    printf '%s' "$name" > "$STATE"
    # Workspace-preparation checks `test -f /opt/vibe/.provisioned && grep -q
    # '^v7-<profile>$'`; pre-create the marker so the real provisioning script
    # is never asked to run at all.
    printf 'v7-%s\n' "${GB_VM_PROFILE:-office}" > "$ROOT/opt/vibe/.provisioned"
    log "launched fake instance '$name' under $ROOT"
    return 0
}

cmd_list() {
    if [ -s "$STATE" ]; then
        printf '[{"name":"%s","status":"RUNNING","type":"container"}]' "$(cat "$STATE")"
    else
        printf '[]'
    fi
    printf '\n'
    return 0
}

cmd_exec() {
    local vm="${1:-}"
    [ -n "$vm" ] || { log "exec needs an instance name"; return 1; }
    shift

    local guest_cwd="" arg
    while [ "$#" -gt 0 ]; do
        arg="$1"
        case "$arg" in
            --cwd)
                guest_cwd="${2:-/}"
                shift 2
                ;;
            --cwd=*) guest_cwd="${arg#--cwd=}"; shift ;;
            --env|-e|--tty|-t|--user|--interactive|-i)
                # These options carry a value in real Incus; consume it when
                # present, otherwise skip the flag.
                if [ "$#" -gt 1 ] && [[ "$2" != --* ]]; then shift 2; else shift; fi
                ;;
            --) shift; break ;;
            -*) shift ;;
            *) break ;;
        esac
    done

    if [ "$#" -eq 0 ]; then
        log "exec without a command; nothing to run"
        return 0
    fi

    local host_cwd
    case "$guest_cwd" in
        /*) host_cwd="$ROOT$guest_cwd" ;;
        "") host_cwd="$ROOT/opt/vibe/app" ;;
        *) host_cwd="$ROOT/$guest_cwd" ;;
    esac
    mkdir -p "$host_cwd" || return 1

    local -a argv=()
    for arg in "$@"; do argv+=("$arg"); done

    if [ "${argv[0]:-}" = "sh" ] && [ "${argv[1]:-}" = "-c" ]; then
        local script="${argv[2]:-}"
        if is_host_mutating_script "$script"; then
            log "skipped host-mutating provisioning script in fake VM"
            return 0
        fi
        local rewritten
        rewritten="$(rewrite_script "$script")"
        ( cd "$host_cwd" && exec /bin/sh -c "$rewritten" )
        return $?
    fi

    local -a safe_argv=()
    for arg in "${argv[@]}"; do safe_argv+=("$(rewrite_path "$arg")"); done
    ( cd "$host_cwd" && exec "${safe_argv[@]}" )
    return $?
}

self_test() {
    local saved_root="$ROOT"
    local tmpdir="${TMPDIR:-/tmp}/gb-ci-incus-stub-selftest-$$"
    ROOT="$tmpdir"
    STATE="$ROOT/.ci-incus-stub-vm"
    init_root || return 1

    cmd_launch images:debian/13 ci-stub-test || return 1
    grep -q 'ci-stub-test' "$STATE" || { log "self-test: launch did not record name"; return 1; }
    local listing
    listing="$(cmd_list)"
    [[ "$listing" == *'"status":"RUNNING"'* ]] || { log "self-test: list not RUNNING"; return 1; }

    cmd_exec ci-stub-test --cwd /opt/vibe/app -- sh -c \
        'mkdir -p output && printf OK > output/probe.txt' || return 1
    [ "$(cat "$ROOT/opt/vibe/app/output/probe.txt")" = "OK" ] || {
        log "self-test: exec did not create the file under the fake root"; return 1;
    }

    local sentinel="$tmpdir/host-sentinel"
    cmd_exec ci-stub-test -- sh -c \
        "printf x > '$sentinel'; echo agent-bootstrap.log" || return 1
    [ ! -e "$sentinel" ] || { log "self-test: provisioning script escaped the no-op"; return 1; }

    [ "$(cat "$ROOT/opt/vibe/.provisioned")" = "v7-office" ] || {
        log "self-test: provision marker missing"; return 1;
    }

    rm -rf "$tmpdir"
    ROOT="$saved_root"
    printf 'SELFTEST PASS: ci-incus-stub launch/list/exec/no-op OK (no Incus daemon)\n'
    return 0
}

subcommand="${1:-}"
[ "$#" -gt 0 ] && shift || true

case "$subcommand" in
    self-test) self_test ;;
    launch) cmd_launch "$@" ;;
    list) cmd_list "$@" ;;
    image) printf '[]\n' ;;
    info) printf '' ;;
    exec) cmd_exec "$@" ;;
    config|start|stop|delete|wait|snapshot|restore|pause|resume|restart|move|copy)
        exit 0
        ;;
    *)
        log "unsupported subcommand '$subcommand' (treated as no-op)"
        exit 0
        ;;
esac
