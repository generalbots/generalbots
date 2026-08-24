"use strict";
/* Spotlight (#1157): universal search overlay (Ctrl+Space) that searches
   apps, actions, and drive files, and hands goals to the AgentExecutor. */

const Spotlight = (() => {
  let index = 0;
  let results = [];

  function isOpen() {
    return document.getElementById("gb-spotlight") !== null;
  }

  function open() {
    if (isOpen()) return;
    const overlay = document.createElement("div");
    overlay.id = "gb-spotlight";
    overlay.className = "gb-spotlight";
    overlay.innerHTML = `
      <div class="gb-spotlight-box">
        <div class="gb-spotlight-input-row">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input type="text" id="gbSpotlightInput" placeholder="Search apps, files… or type a goal for the Concierge" autocomplete="off" spellcheck="false" />
          <span class="gb-spotlight-kbd">Esc</span>
        </div>
        <div class="gb-spotlight-results" id="gbSpotlightResults"></div>
        <div class="gb-spotlight-footer">↑↓ navigate · Enter run · Ctrl+Space or Esc close</div>
      </div>
    `;
    document.body.appendChild(overlay);
    overlay.addEventListener("mousedown", (e) => {
      if (e.target === overlay) close();
    });
    const input = document.getElementById("gbSpotlightInput");
    if (input) {
      setTimeout(() => input.focus(), 30);
      input.addEventListener("input", () => search(input.value));
      input.addEventListener("keydown", (e) => {
        if (e.key === "ArrowDown") { e.preventDefault(); move(1); }
        else if (e.key === "ArrowUp") { e.preventDefault(); move(-1); }
        else if (e.key === "Enter") { e.preventDefault(); run(); }
        else if (e.key === "Escape") { close(); }
      });
    }
    search("");
  }

  function close() {
    const overlay = document.getElementById("gb-spotlight");
    if (overlay) overlay.remove();
  }

  function move(dir) {
    index = (index + dir + results.length) % results.length;
    highlight();
  }

  function highlight() {
    const box = document.getElementById("gbSpotlightResults");
    if (!box) return;
    Array.from(box.querySelectorAll(".gb-spotlight-item")).forEach((el, i) => {
      el.classList.toggle("active", i === index);
    });
    const active = box.querySelector(".gb-spotlight-item.active");
    if (active) active.scrollIntoView({ block: "nearest" });
  }

  function search(q) {
    const box = document.getElementById("gbSpotlightResults");
    if (!box) return;
    const query = (q || "").trim().toLowerCase();
    results = [];
    index = 0;

    const apps = window.APPS_REGISTRY || [];
    apps.forEach((a) => {
      if (!query || (a.title + " " + a.category + " " + (a.description || "")).toLowerCase().indexOf(query) !== -1) {
        results.push({ kind: "app", id: a.id, title: a.title, subtitle: a.category, icon: a.icon, color: a.color, url: a.hxGet });
      }
    });

    // Concierge goal entry: if the query looks like an intent, offer it.
    if (query.length > 2) {
      results.push({
        kind: "goal",
        id: "goal",
        title: "Run goal: “" + q.trim().substring(0, 60) + "”",
        subtitle: "Concierge · plan & execute across apps",
        icon: '<path d="M12 2l2.4 4.9 5.4.8-3.9 3.8.9 5.4-4.8-2.5-4.8 2.5.9-5.4L4.2 7.7l5.4-.8z"/>',
        color: "#84d669",
        goal: q.trim(),
      });
    }

    if (!results.length) {
      box.innerHTML = '<div class="gb-spotlight-empty">No results</div>';
      return;
    }
    box.innerHTML = results
      .map((r, i) => `
        <div class="gb-spotlight-item ${i === 0 ? "active" : ""}" data-i="${i}">
          <div class="gb-spotlight-icon" style="color:${r.color || "#88ccff"}"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${r.icon || ""}</svg></div>
          <div class="gb-spotlight-text">
            <div class="gb-spotlight-title">${escapeHtml(r.title)}</div>
            <div class="gb-spotlight-sub">${escapeHtml(r.subtitle)}</div>
          </div>
        </div>
      `)
      .join("");
    Array.from(box.querySelectorAll(".gb-spotlight-item")).forEach((el) => {
      el.addEventListener("click", () => {
        index = parseInt(el.dataset.i, 10);
        run();
      });
      el.addEventListener("mousemove", () => {
        index = parseInt(el.dataset.i, 10);
        highlight();
      });
    });
  }

  function run() {
    const r = results[index];
    if (!r) return;
    close();
    if (r.kind === "goal") {
      if (window.AgentExecutor && r.goal) window.AgentExecutor.execute(r.goal);
      return;
    }
    if (window.WindowManager) {
      window.WindowManager.open(r.id, r.title, "");
      const sep = r.url.indexOf("?") === -1 ? "?" : "&";
      fetch(r.url + sep + "_=" + Date.now())
        .then((resp) => resp.text())
        .then((html) => {
          const body = document.getElementById("window-body-" + r.id);
          if (body && window.WindowManager._injectBodyContent) {
            window.WindowManager._injectBodyContent(r.id, html);
          }
        })
        .catch(() => {});
    }
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  function toggle() {
    if (isOpen()) close(); else open();
  }

  function init() {
    document.addEventListener("keydown", (e) => {
      if (e.ctrlKey && e.key === " ") {
        e.preventDefault();
        toggle();
      }
    });
  }

  return { init, open, close, toggle, isOpen, search };
})();

window.Spotlight = Spotlight;