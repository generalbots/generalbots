#!/usr/bin/env bash
# One action: regression — timeseries_db uses generic timeseries paths inside
# its Incus container, never the legacy influxdb directories (#2582).
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="reg-timeseries-paths"
uat_step_banner "$id — generic timeseries paths in installed container"
if ! uat_live_gate "$id"; then
    exit 3
fi

container="$(uat_tenant)-timeseries_db"
generic='test -d /opt/gbo/data/timeseries/engine && test -e /opt/gbo/data/timeseries/influxd.bolt && test -d /opt/gbo/conf/timeseries'
legacy='test ! -e /opt/gbo/data/influxdb && test ! -e /opt/gbo/conf/influxdb'

if ! incus exec "$container" -- sh -c "$generic" 2>/dev/null; then
    printf 'generic paths missing in %s\n' "$container" >&2
    uat_record "$id" FAIL "generic timeseries paths absent in $container"
    exit 1
fi
printf 'generic: data/timeseries/engine, data/timeseries/influxd.bolt, conf/timeseries (ok)\n'
if ! incus exec "$container" -- sh -c "$legacy" 2>/dev/null; then
    printf 'legacy influxdb paths still present in %s\n' "$container" >&2
    uat_record "$id" FAIL "legacy influxdb directories present in $container"
    exit 1
fi
printf 'legacy: data/influxdb and conf/influxdb absent (ok)\n'
uat_record "$id" PASS "generic timeseries paths present; no influxdb directories in $container"
