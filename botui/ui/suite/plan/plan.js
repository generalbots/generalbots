"use strict";
/* plan shell — Kanban + Gantt + List hybrid project planner with real-time collab */

(function () {
  const APP = "plan";
  const DEFAULT_PLAN_ID = "default";
  const STORAGE_KEY = "gb-plan-data";
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

  function renderKanban() {
    const root = $("#plan-content");
    if (!root) return;
    root.innerHTML = "";
    const wrap = el("div", { class: "plan-kanban" });
    STATUSES.forEach(function (s) {
      const col = el("div", { class: "plan-kanban-col" });
      const tasks = Store.data.tasks.filter(function (t) { return t.status === s.id; });
      col.appendChild(el("div", { class: "plan-kanban-col-header" }, [
        el("span", { class: "plan-kanban-col-title" }, [
          el("span", { class: "plan-kanban-col-dot", style: { background: s.color } }),
          s.label
        ]),
        el("span", { class: "plan-kanban-col-count" }, String(tasks.length))
      ]));
      const list = el("div", { class: "plan-kanban-col-list" });
      list.dataset.status = s.id;
      tasks.forEach(function (t) { list.appendChild(renderKanbanCard(t)); });
      list.addEventListener("dragover", function (e) { e.preventDefault(); list.style.background = "#1e293b"; });
      list.addEventListener("dragleave", function () { list.style.background = ""; });
      list.addEventListener("drop", function (e) {
        e.preventDefault();
        list.style.background = "";
        const id = e.dataTransfer.getData("text/plain");
        if (id) {
          Store.updateTask(id, { status: s.id });
          if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
            window.GBCollab.send("plan_update", { content: JSON.stringify({ type: "status", id: id, status: s.id }) });
          }
        }
      });
      col.appendChild(list);
      col.appendChild(el("button", {
        class: "plan-kanban-add",
        onclick: function () { promptNewTask(s.id); }
      }, "+ Adicionar"));
      wrap.appendChild(col);
    });
    root.appendChild(wrap);
  }

  function renderKanbanCard(t) {
    const prio = PRIORITIES.find(function (p) { return p.id === t.priority; }) || PRIORITIES[1];
    const card = el("div", {
      class: "plan-card",
      draggable: "true",
      data: { id: t.id, type: "task" }
    }, [
      el("div", { class: "plan-card-priority-bar", style: { background: prio.color } }),
      el("div", { class: "plan-card-body" }, [
        el("div", { class: "plan-card-title" }, t.title),
        el("div", { class: "plan-card-meta" }, [
          t.assignee ? el("span", { class: "plan-card-avatar", title: t.assignee }, t.assignee.charAt(0).toUpperCase()) : null,
          t.end ? el("span", { class: "plan-card-date" }, "📅 " + fmtDate(t.end)) : null,
          t.tags && t.tags.length ? el("span", { class: "plan-card-tags" }, t.tags.map(function (tg) { return el("span", { class: "plan-tag" }, tg); })) : null
        ].filter(Boolean)),
        t.progress != null ? el("div", { class: "plan-card-progress" }, [
          el("div", { class: "plan-card-progress-bar" }, [el("div", { class: "plan-card-progress-fill", style: { width: (t.progress || 0) + "%" } })]),
          el("span", { class: "plan-card-progress-text" }, (t.progress || 0) + "%")
        ]) : null
      ].filter(Boolean))
    ]);
    card.addEventListener("dragstart", function (e) { e.dataTransfer.setData("text/plain", t.id); });
    card.addEventListener("click", function () { openTaskEditor(t); });
    return card;
  }

  function renderGantt() {
    const root = $("#plan-content");
    if (!root) return;
    root.innerHTML = "";
    const tasks = Store.data.tasks.slice().sort(function (a, b) { return (a.start || 0) - (b.start || 0); });
    if (!tasks.length) { root.appendChild(el("p", { class: "plan-empty" }, "Sem tarefas.")); return; }
    const t0 = today();
    const minS = Math.min.apply(null, tasks.map(function (t) { return t.start || t0; }).concat([t0]));
    const maxE = Math.max.apply(null, tasks.map(function (t) { return t.end || t0 + 1; }).concat([t0 + 30]));
    const totalDays = Math.max(7, daysBetween(minS, maxE) + 4);
    const dayW = 28;
    const headerH = 32;
    const rowH = 36;
    const leftW = 220;

    const scroll = el("div", { class: "plan-gantt-scroll" });
    const totalW = leftW + totalDays * dayW;
    const wrap = el("div", { class: "plan-gantt-wrap", style: { width: totalW + "px" } });

    const header = el("div", { class: "plan-gantt-header", style: { height: headerH + "px" }});
    header.appendChild(el("div", { class: "plan-gantt-corner", style: { width: leftW + "px" } }));
    for (let i = 0; i < totalDays; i++) {
      const d = new Date(minS + i * 86400000);
      const isWeekend = d.getDay() === 0 || d.getDay() === 6;
      const isToday = Math.abs(d.getTime() - t0) < 86400000;
      header.appendChild(el("div", {
        class: "plan-gantt-day" + (isWeekend ? " weekend" : "") + (isToday ? " today" : ""),
        style: { width: dayW + "px" }
      }, [
        el("span", { class: "plan-gantt-day-num" }, d.getDate()),
        el("span", { class: "plan-gantt-day-name" }, ["D", "S", "T", "Q", "Q", "S", "S"][d.getDay()])
      ]));
    }
    wrap.appendChild(header);

    const todayOffset = daysBetween(minS, t0);
    const todayLine = el("div", { class: "plan-gantt-today-line", style: { left: (leftW + todayOffset * dayW) + "px", top: headerH + "px", height: (tasks.length * rowH) + "px" } });
    wrap.appendChild(todayLine);

    tasks.forEach(function (t, idx) {
      const s = t.start || minS;
      const e = t.end || (s + 86400000);
      const sOff = clamp(daysBetween(minS, s), 0, totalDays - 1);
      const len = clamp(daysBetween(s, e) + 1, 1, totalDays - sOff);
      const st = STATUSES.find(function (x) { return x.id === t.status; }) || STATUSES[0];
      const prio = PRIORITIES.find(function (p) { return p.id === t.priority; }) || PRIORITIES[1];
      const row = el("div", { class: "plan-gantt-row", style: { height: rowH + "px" } });
      row.appendChild(el("div", { class: "plan-gantt-label", style: { width: leftW + "px" } }, [
        el("div", { class: "plan-gantt-label-title" }, t.title),
        el("div", { class: "plan-gantt-label-meta" }, [
          t.assignee ? el("span", null, t.assignee) : null,
          t.progress != null ? el("span", null, (t.progress || 0) + "%") : null
        ].filter(Boolean))
      ]));
      const bar = el("div", {
        class: "plan-gantt-bar",
        style: {
          left: (leftW + sOff * dayW + 2) + "px",
          width: (len * dayW - 4) + "px",
          top: ((idx * rowH) + (rowH - 22) / 2 + headerH) + "px",
          background: st.color,
          borderLeft: "3px solid " + prio.color
        },
        data: { id: t.id }
      }, [
        el("div", { class: "plan-gantt-bar-progress", style: { width: (t.progress || 0) + "%" } }),
        el("span", { class: "plan-gantt-bar-title" }, t.title)
      ]);
      bar.addEventListener("click", function () { openTaskEditor(t); });
      row.appendChild(bar);
      wrap.appendChild(row);
    });

    scroll.appendChild(wrap);
    root.appendChild(scroll);
  }

  function renderList() {
    const root = $("#plan-content");
    if (!root) return;
    root.innerHTML = "";
    const table = el("table", { class: "plan-list-table" }, [
      el("thead", null, el("tr", null, [
        el("th", null, "Tarefa"),
        el("th", null, "Status"),
        el("th", null, "Prioridade"),
        el("th", null, "Responsável"),
        el("th", null, "Início"),
        el("th", null, "Prazo"),
        el("th", null, "Progresso"),
        el("th", null, "")
      ])),
      el("tbody", null, Store.data.tasks.map(function (t) {
        const st = STATUSES.find(function (x) { return x.id === t.status; }) || STATUSES[0];
        const pr = PRIORITIES.find(function (x) { return x.id === t.priority; }) || PRIORITIES[1];
        return el("tr", { data: { id: t.id }, onclick: function () { openTaskEditor(t); } }, [
          el("td", null, t.title),
          el("td", null, [el("span", { class: "plan-status-pill", style: { background: st.color } }, st.label)]),
          el("td", null, [el("span", { class: "plan-prio-pill", style: { background: pr.color } }, pr.label)]),
          el("td", null, t.assignee || "—"),
          el("td", null, fmtDate(t.start)),
          el("td", null, fmtDate(t.end)),
          el("td", null, [
            el("div", { class: "plan-list-progress" }, [el("div", { class: "plan-list-progress-fill", style: { width: (t.progress || 0) + "%" } })])
          ]),
          el("td", null, [
            el("button", {
              class: "plan-row-del",
              onclick: function (ev) { ev.stopPropagation(); if (confirm("Excluir tarefa?")) Store.removeTask(t.id); }
            }, "×")
          ])
        ]);
      }))
    ]);
    root.appendChild(table);
  }

  function openTaskEditor(task) {
    const modal = $("#modal-container");
    if (!modal) return;
    modal.innerHTML = "";
    const prioOpts = PRIORITIES.map(function (p) { return '<option value="' + p.id + '"' + (task.priority === p.id ? " selected" : "") + '>' + p.label + '</option>'; }).join("");
    const stOpts = STATUSES.map(function (s) { return '<option value="' + s.id + '"' + (task.status === s.id ? " selected" : "") + '>' + s.label + '</option>'; }).join("");
    const toDateInput = function (ts) { if (!ts) return ""; const d = new Date(ts); return d.toISOString().slice(0, 10); };
    modal.innerHTML = '<div class="plan-modal-backdrop" id="plan-modal-backdrop"></div>' +
      '<div class="plan-modal">' +
        '<div class="plan-modal-header"><h3>Editar Tarefa</h3><button class="plan-modal-close" id="plan-modal-close">×</button></div>' +
        '<div class="plan-modal-body">' +
          '<label>Título<input id="f-title" value="' + (task.title || "").replace(/"/g, "&quot;") + '" /></label>' +
          '<label>Status<select id="f-status">' + stOpts + '</select></label>' +
          '<label>Prioridade<select id="f-priority">' + prioOpts + '</select></label>' +
          '<label>Responsável<input id="f-assignee" value="' + (task.assignee || "").replace(/"/g, "&quot;") + '" /></label>' +
          '<label>Início<input type="date" id="f-start" value="' + toDateInput(task.start) + '" /></label>' +
          '<label>Prazo<input type="date" id="f-end" value="' + toDateInput(task.end) + '" /></label>' +
          '<label>Progresso (%)<input type="number" min="0" max="100" id="f-progress" value="' + (task.progress || 0) + '" /></label>' +
          '<label>Tags (vírgula)<input id="f-tags" value="' + (task.tags || []).join(", ").replace(/"/g, "&quot;") + '" /></label>' +
        '</div>' +
        '<div class="plan-modal-footer">' +
          '<button class="plan-btn plan-btn-secondary" id="plan-cancel">Cancelar</button>' +
          '<button class="plan-btn plan-btn-primary" id="plan-save">Salvar</button>' +
        '</div>' +
      '</div>';
    function close() { modal.innerHTML = ""; }
    $("#plan-modal-close").onclick = close;
    $("#plan-modal-backdrop").onclick = close;
    $("#plan-cancel").onclick = close;
    $("#plan-save").onclick = function () {
      const patch = {
        title: $("#f-title").value.trim() || task.title,
        status: $("#f-status").value,
        priority: $("#f-priority").value,
        assignee: $("#f-assignee").value.trim(),
        start: $("#f-start").value ? new Date($("#f-start").value).getTime() : task.start,
        end: $("#f-end").value ? new Date($("#f-end").value).getTime() : task.end,
        progress: clamp(parseInt($("#f-progress").value, 10) || 0, 0, 100),
        tags: $("#f-tags").value.split(",").map(function (s) { return s.trim(); }).filter(Boolean)
      };
      Store.updateTask(task.id, patch);
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        window.GBCollab.send("plan_update", { content: JSON.stringify(Object.assign({ type: "task", id: task.id }, patch)) });
      }
      close();
    };
  }

  function promptNewTask(status) {
    const id = uid();
    const t = { id: id, title: "Nova tarefa", status: status || "todo", priority: "medium", start: today(), end: today() + 7, progress: 0, assignee: "", tags: [] };
    Store.addTask(t);
    openTaskEditor(t);
  }

  function render() {
    if (View.current === "kanban") renderKanban();
    else if (View.current === "gantt") renderGantt();
    else renderList();
  }

  function initViewTabs() {
    $$(".plan-view-tab").forEach(function (b) {
      b.addEventListener("click", function () {
        View.current = b.dataset.view;
        $$(".plan-view-tab").forEach(function (x) { x.classList.toggle("active", x === b); });
        render();
      });
    });
  }

  function initCollab() {
    if (!window.GBCollab) return;
    const connStatus = $("#gb-conn-status");
    window.GBCollab.connect({
      app: APP,
      docId: DEFAULT_PLAN_ID,
      collaboratorsEl: $("#collaborators"),
      onConnect: function () {
        if (connStatus) { connStatus.className = "gb-connection-status online"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "online"; }
      },
      onDisconnect: function () {
        if (connStatus) { connStatus.className = "gb-connection-status offline"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "offline"; }
      },
      onEdit: function (msg) {
        if (!msg || !msg.content) return;
        try {
          const u = JSON.parse(msg.content);
          if (u.type === "task") Store.updateTask(u.id, u);
          else if (u.type === "status") Store.updateTask(u.id, { status: u.status });
        } catch (_) {}
      }
    });
  }

  function initAuth() {
    if (window.GBAuthGuard) GBAuthGuard.injectLoginButton($("#gb-auth-button"));
  }

  window.addEventListener("DOMContentLoaded", function () {
    Store.load();
    Store.subscribe(render);
    initViewTabs();
    initAuth();
    initCollab();
    render();
    window.PlanStore = Store;
    window.PlanView = View;
  });
})();
