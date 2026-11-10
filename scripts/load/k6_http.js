// REST load profile for the SLO budgets (#2115).
//
//   smoke : 10 concurrent sessions for 30 s   — every pull request, mock LLM
//   load  : 50 or 200 concurrent sessions     — nightly
//   soak  : 50 sessions for 60 minutes        — nightly, watch RSS/FD growth
//
//   k6 run -e SCENARIO=smoke -e BASE_URL=http://localhost:8080 scripts/load/k6_http.js
//   k6 run -e SCENARIO=soak  -e BASE_URL=http://localhost:8080 \
//          --summary-export=/tmp/k6-summary.json scripts/load/k6_http.js
//   python3 scripts/load/k6_to_snapshot.py --summary /tmp/k6-summary.json \
//          --out /tmp/slo-api.json --class api
//   python3 scripts/check_slo.py --snapshot /tmp/slo-api.json --class api
//
// The thresholds below mirror ops/slo.yaml (rest.api_ms p95 300 ms / p99 900 ms,
// turn.error_pct 1 %). They are duplicated on purpose: k6 fails the run as soon
// as it crosses them instead of waiting for the snapshot conversion.
import http from "k6/http";
import { check, sleep } from "k6";
import { Rate } from "k6/metrics";

const BASE_URL = __ENV.BASE_URL || "http://localhost:8080";
const SCENARIO = __ENV.SCENARIO || "smoke";
const TOKEN = __ENV.BEARER_TOKEN || "";
// Anonymous endpoints only by default: a load run must never touch a paid path.
const PATHS = (__ENV.PATHS || "/health,/api/product").split(",");

const serverErrors = new Rate("server_error_rate");

const scenarios = {
  smoke: {
    executor: "constant-vus",
    vus: 10,
    duration: "30s",
  },
  load: {
    executor: "ramping-vus",
    startVUs: 10,
    stages: [
      { duration: "2m", target: Number(__ENV.LOAD_VUS || 50) },
      { duration: "6m", target: Number(__ENV.LOAD_VUS || 50) },
      { duration: "2m", target: 0 },
    ],
  },
  soak: {
    executor: "constant-vus",
    vus: Number(__ENV.SOAK_VUS || 50),
    duration: __ENV.SOAK_DURATION || "60m",
  },
};

export const options = {
  scenarios: { [SCENARIO]: scenarios[SCENARIO] || scenarios.smoke },
  thresholds: {
    // ops/slo.yaml -> rest.api_ms
    http_req_duration: ["p(95)<300", "p(99)<900"],
    // ops/slo.yaml -> turn.error_pct (non-2xx/3xx share)
    http_req_failed: ["rate<0.01"],
    // provider/dependency 5xx must not be masked by the aggregate rate
    server_error_rate: ["rate<0.01"],
  },
  summaryTrendStats: ["avg", "min", "med", "p(50)", "p(90)", "p(95)", "p(99)", "max"],
};

function headers() {
  const base = { Accept: "application/json" };
  if (TOKEN) {
    base.Authorization = `Bearer ${TOKEN}`;
  }
  return base;
}

export default function () {
  for (const path of PATHS) {
    const response = http.get(`${BASE_URL}${path}`, {
      headers: headers(),
      tags: { class: "api", endpoint: path },
    });

    // 5xx is a server-side failure: the dependency/error SLO, not a client bug.
    serverErrors.add(response.status >= 500);

    check(response, {
      "status is 2xx": (r) => r.status >= 200 && r.status < 300,
      "body is not empty": (r) => (r.body || "").length > 0,
    });
  }

  // A short think time keeps the VUs from becoming a pure CPU spin on a
  // single-container dev box, which would measure k6 rather than botserver.
  sleep(Number(__ENV.THINK_TIME || 0.2));
}
