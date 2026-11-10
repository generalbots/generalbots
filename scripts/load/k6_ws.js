// WebSocket stability and first-token profile (#2115).
//
// Measures the two chat SLOs that only exist on a live socket:
//
//   chat.first_token_ms          -> ws_first_token_ms (Trend)
//   ws.unexpected_disconnect_pct -> ws_unexpected_disconnect_pct (Rate)
//
//   k6 run -e SCENARIO=smoke -e WS_URL=ws://localhost:8080/ws \
//          --summary-export=/tmp/k6-ws.json scripts/load/k6_ws.js
//   python3 scripts/load/k6_to_snapshot.py --summary /tmp/k6-ws.json \
//          --out /tmp/slo-ws.json --class chat
//   python3 scripts/check_slo.py --snapshot /tmp/slo-ws.json --class chat
//
// A disconnect counts as unexpected unless this script closed the socket after
// a full answer: the counter is what the SLO is about, not the raw close rate.
import ws from "k6/ws";
import { check } from "k6";
import { Rate, Trend } from "k6/metrics";

const WS_URL = __ENV.WS_URL || "ws://localhost:8080/ws";
const SCENARIO = __ENV.SCENARIO || "smoke";
const TOKEN = __ENV.BEARER_TOKEN || "";
const MESSAGE = __ENV.MESSAGE || "hello";
const ANSWER_DEADLINE_MS = Number(__ENV.ANSWER_DEADLINE_MS || 15000);

const firstToken = new Trend("ws_first_token_ms", true);
const unexpectedDisconnect = new Rate("ws_unexpected_disconnect_pct");

export const options = {
  scenarios: {
    [SCENARIO]: {
      executor: "constant-vus",
      vus: Number(__ENV.VUS || (SCENARIO === "smoke" ? 10 : 50)),
      duration: __ENV.DURATION || (SCENARIO === "soak" ? "60m" : "30s"),
    },
  },
  thresholds: {
    // ops/slo.yaml -> chat.first_token_ms (p95 2000 ms, p99 4000 ms)
    ws_first_token_ms: ["p(95)<2000", "p(99)<4000"],
    // ops/slo.yaml -> ws.unexpected_disconnect_pct (0.5 %)
    ws_unexpected_disconnect_pct: ["rate<0.005"],
  },
  summaryTrendStats: ["avg", "min", "med", "p(50)", "p(90)", "p(95)", "p(99)", "max"],
};

export default function () {
  const url = TOKEN ? `${WS_URL}?token=${TOKEN}` : WS_URL;
  let answered = false;
  let sentAt = 0;

  const response = ws.connect(url, {}, function (socket) {
    socket.on("open", function () {
      sentAt = Date.now();
      socket.send(JSON.stringify({ type: 1, message: MESSAGE, message_type: 1 }));
    });

    socket.on("message", function (raw) {
      if (!sentAt) {
        return;
      }
      if (!answered) {
        answered = true;
        firstToken.add(Date.now() - sentAt);
      }
      check(raw, { "chunk is not empty": (payload) => (payload || "").length > 0 });
    });

    socket.setTimeout(function () {
      if (!answered) {
        // The server never produced a token inside the deadline: a failure of
        // the first-token SLO, recorded as an unexpected disconnect.
        unexpectedDisconnect.add(true);
      }
      socket.close();
    }, ANSWER_DEADLINE_MS);
  });

  check(response, { "socket upgraded": (r) => r && r.status === 101 });
  if (!answered) {
    unexpectedDisconnect.add(true);
  } else {
    unexpectedDisconnect.add(false);
  }
}
