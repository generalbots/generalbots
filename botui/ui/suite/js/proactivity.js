"use strict";
/* Proactivity notifications (#1185): polls the backend suggestion-card
   feed and surfaces unseen cards through the desktop notification tray.
   Consented cards only — the server never emits cards for triggers the
   user has denied. */

const Proactivity = (() => {
  const CARDS_API = "/api/vibe/proactivity/cards?include_seen=false";
  const POLL_MS = 60000;
  const SEEN_KEY = "gb-proactivity-seen";
  let timer = null;

  function seenSet() {
    try {
      return new Set(JSON.parse(localStorage.getItem(SEEN_KEY) || "[]"));
    } catch (e) {
      return new Set();
    }
  }

  function markSeen(id) {
    try {
      const set = seenSet();
      set.add(id);
      localStorage.setItem(SEEN_KEY, JSON.stringify(Array.from(set).slice(-200)));
    } catch (e) {
      /* non-fatal */
    }
  }

  function poll() {
    fetch(CARDS_API, { headers: { "Content-Type": "application/json" } })
      .then((r) => r.json())
      .then((data) => {
        const cards = (data && data.cards) || [];
        const seen = seenSet();
        for (const card of cards) {
          if (seen.has(card.card_id)) continue;
          seen.add(card.card_id);
          if (window.GBToasts && window.GBToasts.show) {
            window.GBToasts.show(card.title || "Suggestion", card.body || "", "info");
          } else {
            window.dispatchEvent(
              new CustomEvent("gb-proactivity-card", { detail: card })
            );
          }
          markSeen(card.card_id);
          // Acknowledge server-side so the feed does not grow unbounded.
          fetch("/api/vibe/proactivity/cards/" + card.card_id + "/seen", { method: "POST" }).catch(function () {});
        }
      })
      .catch(() => {
        /* API unavailable — try again next tick */
      });
  }

  function init() {
    if (window.__gbProactivityStarted) return;
    window.__gbProactivityStarted = true;
    // First poll after a short delay so auth/bootstrap has settled.
    timer = setTimeout(function tick() {
      poll();
      timer = setTimeout(tick, POLL_MS);
    }, 8000);
  }

  function stop() {
    if (timer) clearTimeout(timer);
    timer = null;
    window.__gbProactivityStarted = false;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  return { init, poll, stop };
})();

window.Proactivity = Proactivity;
