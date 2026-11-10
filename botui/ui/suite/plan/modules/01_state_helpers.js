/* Plan app — module 1/4: constants, helpers and store (split from plan/plan.js for #1731) */

if (window.GBAppLifecycle) GBAppLifecycle.begin("plan");

/* plan shell — Kanban + Gantt + List hybrid project planner with real-time collab */

(function (app) {
  "use strict";
  const APP = "plan";
  const DEFAULT_PLAN_ID = "default";
  const STORAGE_KEY = "gb-plan-data";
  /* Plan ids are minted by the server (#2136); `DEFAULT_PLAN_ID` only marks a
     local draft that has no server plan yet. */
  const SERVER_ID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  function isServerPlanId(id) { return SERVER_ID_RE.test(String(id || "")); }
  function authToken() {
    try {
      return localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    } catch (_) { return ""; }
  }
  /* Every plan route requires a verified credential: the same token the rest
     of the suite sends. */
  function authHeaders(extra) {
    const headers = Object.assign({ "Content-Type": "application/json" }, extra || {});
    const token = authToken();
    if (token) headers.Authorization = "Bearer " + token;
    return headers;
  }
  const STATUSES = [
    { id: "todo", label: "A Fazer", color: "#64748b" },
    { id: "inprogress", label: "Em Andamento", color: "#3b82f6" },
    { id: "review", label: "Em Revisão", color: "#a855f7" },
    { id: "done", label: "Concluído", color: "#22c55e" }
  ];
  const PRIORITIES = [
    { id: "low", label: "Baixa", color: "#94a3b8" },
    { id: "medium", label: "Média", color: "#f59e0b" },
    { id: "high", label: "Alta", color: "#f97316" },
    { id: "urgent", label: "Urgente", color: "#ef4444" }
  ];

  function $(s, r) { return (r || document).querySelector(s); }
  function $$(s, r) { return Array.from((r || document).querySelectorAll(s)); }
  function el(tag, attrs, children) {
    const n = document.createElement(tag);
    if (attrs) for (const k in attrs) {
      if (k === "style" && typeof attrs[k] === "object") Object.assign(n.style, attrs[k]);
      else if (k === "class") n.className = attrs[k];
      else if (k === "data" && typeof attrs[k] === "object") for (const dk in attrs[k]) n.dataset[dk] = attrs[k][dk];
      else if (k.startsWith("on") && typeof attrs[k] === "function") n.addEventListener(k.slice(2), attrs[k]);
      else n.setAttribute(k, attrs[k]);
    }
    if (children) {
      const list = Array.isArray(children) ? children : [children];
      list.forEach(function (c) { if (c != null) n.appendChild(typeof c === "string" ? document.createTextNode(c) : c); });
    }
    return n;
  }
  function uid() { return "t_" + Math.random().toString(36).slice(2, 10); }
  function today() { const d = new Date(); d.setHours(0, 0, 0, 0); return d.getTime(); }
  function fmtDate(ts) { if (!ts) return "—"; const d = new Date(ts); return d.toLocaleDateString("pt-BR", { day: "2-digit", month: "short" }); }
  function daysBetween(a, b) { return Math.round((b - a) / 86400000); }
  function clamp(n, lo, hi) { return Math.max(lo, Math.min(hi, n)); }

  const Store = {
    data: { id: DEFAULT_PLAN_ID, title: "Meu Plano", tasks: [], members: [], createdAt: Date.now() },
    listeners: [],
    load: function () {
      try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) this.data = JSON.parse(raw);
      } catch (_) {}
      if (!this.data.tasks || !this.data.tasks.length) this.seed();
    },
    save: function () {
      try { localStorage.setItem(STORAGE_KEY, JSON.stringify(this.data)); } catch (_) {}
    },
    seed: function () {
      const now = today();
      this.data.tasks = [
        { id: uid(), title: "Planejamento inicial", status: "done", priority: "high", start: now - 14, end: now - 7, progress: 100, assignee: "Ana", tags: ["planejamento"], depends: [] },
        { id: uid(), title: "Setup do servidor", status: "done", priority: "high", start: now - 7, end: now - 2, progress: 100, assignee: "Bruno", tags: ["infra"], depends: [] },
        { id: uid(), title: "Backend Rust", status: "inprogress", priority: "high", start: now - 2, end: now + 10, progress: 45, assignee: "Carla", tags: ["backend", "rust"], depends: [] },
        { id: uid(), title: "Frontend HTMX", status: "inprogress", priority: "medium", start: now, end: now + 8, progress: 30, assignee: "Diego", tags: ["frontend"], depends: [] },
        { id: uid(), title: "Integração Zitadel", status: "todo", priority: "high", start: now + 3, end: now + 12, progress: 0, assignee: "Ana", tags: ["auth"], depends: [] },
        { id: uid(), title: "Testes E2E", status: "todo", priority: "medium", start: now + 10, end: now + 18, progress: 0, assignee: "Bruno", tags: ["qa"], depends: [] },
        { id: uid(), title: "Deploy produção", status: "review", priority: "urgent", start: now + 16, end: now + 20, progress: 80, assignee: "Carla", tags: ["deploy"], depends: [] }
      ];
      this.save();
    },
    subscribe: function (fn) { this.listeners.push(fn); return function () { this.listeners = this.listeners.filter(function (l) { return l !== fn; }); }.bind(this); },
    notify: function () { this.listeners.forEach(function (l) { l(this.data); }.bind(this)); this.save(); },
    addTask: function (task) { this.data.tasks.push(task); this.notify(); },
    updateTask: function (id, patch) {
      const t = this.data.tasks.find(function (t) { return t.id === id; });
      if (t) { Object.assign(t, patch); this.notify(); }
    },
    removeTask: function (id) {
      this.data.tasks = this.data.tasks.filter(function (t) { return t.id !== id; });
      this.notify();
    }
  };

  const View = { current: "kanban" };

  Object.assign(app, { APP, DEFAULT_PLAN_ID, isServerPlanId, authToken, authHeaders, STATUSES, PRIORITIES, $, $$, el, uid, today, fmtDate, daysBetween, clamp, Store, View });
})(window.__gbPlan = window.__gbPlan || {});
