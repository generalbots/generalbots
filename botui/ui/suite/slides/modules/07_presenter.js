"use strict";
/* slides presenter module — laser pointer, live Q&A, remote control.
 *
 * The slides WebSocket (botslides collaboration layer) now carries:
 *   laser              — pointer position (normalised x/y over the canvas)
 *   question           — an attendee asks a question
 *   question_upvote    — +1 on a question
 *   question_answered  — presenter marks a question resolved
 *   presenter_take / presenter_assign / presenter_release — presenter role
 *   remote_nav         — presenter/co-presenter drives everyone's slide
 *
 * Public API (window.SlidesPresenter):
 *   laser(msg), remoteNav(msg), question(msg), presenter(msg)
 *   toggleLaser(), toggleQa(), takePresenter(), releasePresenter()
 *   submitQuestion(text), upvote(id), markAnswered(id)
 */
(function (window) {
  var CSS_ID = "gb-slides-presenter-css";
  var laserActive = false;
  var laserDot = null;
  var laserTimer = null;
  var laserFrame = null;
  var panel = null;
  var listEl = null;
  var state = { presentationId: "current", questions: [], isPresenter: false, presenterName: null };

  function presId() {
    return (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current";
  }
  function send(type, data) {
    if (window.GBCollab && window.GBCollab.send) window.GBCollab.send(type, data);
  }
  function announce(m) {
    if (window.GBCollabA11y && window.GBCollabA11y.announce) window.GBCollabA11y.announce(m);
  }
  function esc(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }
  function canvasHost() {
    return document.getElementById("slides-content");
  }
  function canvas() {
    var host = canvasHost();
    return host ? host.querySelector(".sl-canvas") : null;
  }

  function ensureCss() {
    if (document.getElementById(CSS_ID)) return;
    var css = [
      ".sl-laser-dot{position:fixed;width:14px;height:14px;margin:-7px 0 0 -7px;border-radius:50%;",
      "background:rgba(239,68,68,.25);border:2px solid #ef4444;pointer-events:none;z-index:100001;",
      "box-shadow:0 0 12px 3px rgba(239,68,68,.55);transition:opacity .4s ease;opacity:0;}",
      ".sl-laser-dot.on{opacity:1;}",
      "#gb-qa-panel{position:fixed;top:0;right:0;bottom:0;width:380px;max-width:94vw;",
      "background:#0f172a;border-left:1px solid #334155;z-index:100000;display:flex;flex-direction:column;",
      "box-shadow:-8px 0 24px rgba(0,0,0,.4);transform:translateX(100%);transition:transform .2s ease;",
      "font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;}",
      "#gb-qa-panel.gqa-open{transform:translateX(0);}",
      "#gb-qa-panel .gqa-header{display:flex;align-items:center;gap:8px;padding:12px 14px;",
      "border-bottom:1px solid #334155;background:#1e293b;}",
      "#gb-qa-panel .gqa-title{flex:1;color:#f8fafc;font-size:14px;font-weight:600;}",
      "#gb-qa-panel .gqa-close{background:none;border:none;color:#94a3b8;font-size:20px;",
      "line-height:1;cursor:pointer;padding:0 4px;}",
      "#gb-qa-panel .gqa-close:hover{color:#f8fafc;}",
      "#gb-qa-panel .gqa-list{flex:1;overflow-y:auto;padding:12px 14px;display:flex;",
      "flex-direction:column;gap:10px;}",
      "#gb-qa-panel .gqa-empty{color:#94a3b8;font-size:13px;text-align:center;padding:24px 8px;}",
      "#gb-qa-panel .gqa-item{background:#1e293b;border:1px solid #334155;border-radius:8px;padding:10px 12px;}",
      "#gb-qa-panel .gqa-item.answered{opacity:.55;}",
      "#gb-qa-panel .gqa-meta{display:flex;align-items:center;gap:8px;margin-bottom:6px;}",
      "#gb-qa-panel .gqa-author{color:#f8fafc;font-weight:600;font-size:12.5px;flex:1;",
      "overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-qa-panel .gqa-badge{font-size:10.5px;padding:2px 8px;border-radius:999px;font-weight:600;}",
      "#gb-qa-panel .gqa-badge.answered{background:#064e3b;color:#6ee7b7;}",
      "#gb-qa-panel .gqa-badge.open{background:#7c2d12;color:#fdba74;}",
      "#gb-qa-panel .gqa-text{color:#cbd5e1;font-size:13px;line-height:1.5;word-break:break-word;}",
      "#gb-qa-panel .gqa-actions{display:flex;gap:6px;margin-top:8px;}",
      "#gb-qa-panel .gqa-actions button{background:#0f172a;border:1px solid #334155;color:#93c5fd;",
      "border-radius:6px;padding:4px 10px;font-size:12px;cursor:pointer;}",
      "#gb-qa-panel .gqa-actions button:hover{background:#334155;}",
      "#gb-qa-panel .gqa-compose{display:flex;gap:8px;padding:12px 14px;border-top:1px solid #334155;}",
      "#gb-qa-panel .gqa-input{flex:1;background:#1e293b;border:1px solid #334155;border-radius:6px;",
      "color:#f8fafc;padding:8px 10px;font-size:13px;resize:none;min-height:36px;}",
      "#gb-qa-panel .gqa-send{background:#3b82f6;border:none;color:#fff;border-radius:6px;",
      "padding:0 14px;font-size:13px;cursor:pointer;}",
      "#gb-qa-panel .gqa-send:hover{background:#2563eb;}",
      ".sl-presenter-badge{font-size:11px;font-weight:600;padding:2px 8px;border-radius:999px;",
      "background:#7c2d12;color:#fdba74;display:none;}",
      ".slides-toolbar .btn-icon.active{background:#3b82f6;color:#fff;border-color:#3b82f6;}"
    ].join("");
    var style = document.createElement("style");
    style.id = CSS_ID;
    style.textContent = css;
    document.head.appendChild(style);
  }

  /* ---- Laser pointer ---- */
  function ensureLaserDot() {
    if (laserDot && laserDot.parentNode) return laserDot;
    laserDot = document.createElement("div");
    laserDot.className = "sl-laser-dot";
    document.body.appendChild(laserDot);
    return laserDot;
  }

  function showLaser(msg) {
    var cv = canvas();
    if (!cv || !msg || !msg.data) return;
    var x = typeof msg.data.x === "number" ? msg.data.x : null;
    var y = typeof msg.data.y === "number" ? msg.data.y : null;
    if (x === null || y === null) { hideLaser(); return; }
    var rect = cv.getBoundingClientRect();
    var dot = ensureLaserDot();
    dot.style.left = (rect.left + x * rect.width) + "px";
    dot.style.top = (rect.top + y * rect.height) + "px";
    dot.classList.add("on");
    clearTimeout(laserTimer);
    laserTimer = setTimeout(hideLaser, 900);
  }

  function hideLaser() {
    if (laserDot) laserDot.classList.remove("on");
  }

  function onCanvasMove(e) {
    if (!laserActive) return;
    var cv = canvas();
    if (!cv) return;
    var rect = cv.getBoundingClientRect();
    if (laserFrame) return;
    laserFrame = window.requestAnimationFrame(function () {
      laserFrame = null;
      send("laser", {
        slide_index: parseInt((cv.dataset.slideId) || "0", 10) || 0,
        x: Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width)),
        y: Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height))
      });
    });
  }

  function toggleLaser() {
    laserActive = !laserActive;
    var btn = document.getElementById("laserBtn");
    if (btn) btn.classList.toggle("active", laserActive);
    var host = canvasHost();
    if (host) {
      if (laserActive) host.addEventListener("mousemove", onCanvasMove);
      else host.removeEventListener("mousemove", onCanvasMove);
    }
    announce(laserActive ? "Laser pointer on" : "Laser pointer off");
  }

  /* ---- Q&A ---- */
  function ensurePanel() {
    ensureCss();
    if (panel && panel.parentNode) return panel;
    panel = document.createElement("div");
    panel.id = "gb-qa-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-label", "Questions");
    panel.innerHTML =
      '<div class="gqa-header">' +
      '<span class="gqa-title">Questions &amp; answers</span>' +
      '<button class="gqa-close" title="Close" aria-label="Close">×</button>' +
      '</div>' +
      '<div class="gqa-list"></div>' +
      '<div class="gqa-compose">' +
      '<textarea class="gqa-input" placeholder="Ask a question…" aria-label="Question"></textarea>' +
      '<button class="gqa-send">Ask</button>' +
      '</div>';
    document.body.appendChild(panel);
    listEl = panel.querySelector(".gqa-list");
    panel.querySelector(".gqa-close").addEventListener("click", closeQa);
    var input = panel.querySelector(".gqa-input");
    var sendBtn = panel.querySelector(".gqa-send");
    function doSend() {
      var text = input.value.trim();
      if (!text) return;
      input.value = "";
      send("question", { text: text });
      state.questions.unshift({
        id: "local-" + Date.now(), author_name: "You", text: text,
        upvotes: 0, answered: false
      });
      renderQuestions();
      announce("Question asked");
    }
    sendBtn.addEventListener("click", doSend);
    input.addEventListener("keydown", function (e) {
      if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); doSend(); }
    });
    return panel;
  }

  function fetchQuestions() {
    return fetch("/api/slides/" + encodeURIComponent(presId()) + "/questions")
      .then(function (r) { return r.ok ? r.json() : { questions: [] }; })
      .catch(function () { return { questions: [] }; })
      .then(function (d) { return d.questions || []; });
  }

  function renderQuestions() {
    if (!listEl) return;
    if (!state.questions.length) {
      listEl.innerHTML = '<div class="gqa-empty">No questions yet — be the first to ask.</div>';
      return;
    }
    listEl.innerHTML = state.questions.map(function (q) {
      var answered = !!q.answered;
      return '<div class="gqa-item' + (answered ? " answered" : "") + '">' +
        '<div class="gqa-meta">' +
        '<span class="gqa-author">' + esc(q.author_name || q.author_id || "Anonymous") + '</span>' +
        '<span class="gqa-badge ' + (answered ? "answered" : "open") + '">' +
        (answered ? "answered" : "open") + '</span>' +
        '</div>' +
        '<div class="gqa-text">' + esc(q.text) + '</div>' +
        '<div class="gqa-actions">' +
        '<button data-q="' + esc(q.id) + '" data-act="upvote">▲ ' + (q.upvotes || 0) + '</button>' +
        '<button data-q="' + esc(q.id) + '" data-act="answered">' + (answered ? "Reopen" : "Mark answered") + '</button>' +
        '</div>' +
        '</div>';
    }).join("");
  }

  function bindPanelActions() {
    if (!panel) return;
    panel.querySelectorAll("[data-q][data-act]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        var id = btn.dataset.q;
        var act = btn.dataset.act;
        if (act === "upvote") {
          send("question_upvote", { question_id: id });
          state.questions.forEach(function (q) { if (q.id === id) q.upvotes = (q.upvotes || 0) + 1; });
        } else {
          send("question_answered", { question_id: id });
          state.questions.forEach(function (q) { if (q.id === id) q.answered = !q.answered; });
        }
        renderQuestions();
      });
    });
  }

  function openQa() {
    ensurePanel();
    panel.classList.add("gqa-open");
    fetchQuestions().then(function (list) {
      state.questions = list.slice();
      renderQuestions();
      bindPanelActions();
    });
    announce("Questions panel opened");
  }

  function closeQa() {
    if (panel) panel.classList.remove("gqa-open");
  }

  function toggleQa() {
    if (panel && panel.classList.contains("gqa-open")) closeQa();
    else openQa();
  }

  function question(msg) {
    if (!msg || !msg.data || !msg.data.text) return;
    state.questions.unshift({
      id: msg.data.question_id || ("q-" + Date.now()),
      author_name: msg.user_name || "Attendee",
      text: msg.data.text,
      upvotes: 0,
      answered: false
    });
    if (panel && panel.classList.contains("gqa-open")) { renderQuestions(); bindPanelActions(); }
    announce((msg.user_name || "An attendee") + " asked a question");
  }

  /* ---- Presenter / remote control ---- */
  function presenterBadge() {
    var b = document.getElementById("presenterBadge");
    if (!b) return;
    b.style.display = state.isPresenter ? "inline-flex" : "none";
  }

  function takePresenter() {
    send("presenter_take");
    state.isPresenter = true;
    presenterBadge();
    announce("You are now presenting");
  }

  function releasePresenter() {
    send("presenter_release");
    state.isPresenter = false;
    presenterBadge();
    announce("You stopped presenting");
  }

  function presenter(msg) {
    // presenter_take / assign / release broadcast so every peer can render the
    // presenter badge. The message's user is the actor, not necessarily self.
    if (!msg) return;
    if (msg.msg_type === "presenter_release") {
      state.isPresenter = false;
      state.presenterName = null;
    } else {
      state.presenterName = msg.user_name || null;
    }
    presenterBadge();
  }

  function remoteNav(msg) {
    if (!msg || msg.slide_index === undefined || msg.slide_index === null) return;
    var idx = Number(msg.slide_index);
    var thumbs = document.querySelectorAll(".sl-thumb");
    if (thumbs[idx]) thumbs[idx].click();
    announce((msg.user_name || "Presenter") + " moved to slide " + (idx + 1));
  }

  function refreshQa() {
    if (!panel || !panel.classList.contains("gqa-open")) return;
    fetchQuestions().then(function (list) {
      state.questions = list.slice();
      renderQuestions();
      bindPanelActions();
    });
  }

  function refreshPresenterStatus() {
    fetch("/api/slides/" + encodeURIComponent(presId()) + "/presenter-control")
      .then(function (r) { return r.ok ? r.json() : {}; })
      .catch(function () { return {}; })
      .then(function (d) {
        if (d.presenter) {
          state.isPresenter = d.presenter.presenter_id === (window.GBCollab && window.GBCollab.getUser().id);
          state.presenterName = d.presenter.presenter_name;
          presenterBadge();
        }
      });
  }

  window.SlidesPresenter = {
    laser: showLaser,
    remoteNav: remoteNav,
    question: question,
    presenter: presenter,
    toggleLaser: toggleLaser,
    toggleQa: toggleQa,
    openQa: openQa,
    closeQa: closeQa,
    takePresenter: takePresenter,
    releasePresenter: releasePresenter,
    togglePresenter: function () {
      if (state.isPresenter) releasePresenter();
      else takePresenter();
    },
    isPresenter: function () { return state.isPresenter; },
    submitQuestion: function (text) {
      if (!text) return;
      send("question", { text: text });
    },
    upvote: function (id) { send("question_upvote", { question_id: id }); },
    markAnswered: function (id) { send("question_answered", { question_id: id }); },
    refreshQa: refreshQa,
    refreshPresenterStatus: refreshPresenterStatus
  };
})(window);
