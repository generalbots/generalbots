/* Plan app — module 2/4: Kanban, Gantt and list renderers (split from plan/plan.js for #1731) */

(function (app) {
  "use strict";
  const { STATUSES, PRIORITIES, $, el, today, fmtDate, daysBetween, clamp, Store } = app;

  // Linked lazily — provided by the editor/actions module, resolved on first call.
  const promptNewTask = (...args) => app.promptNewTask(...args);
  const openTaskEditor = (...args) => app.openTaskEditor(...args);

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
        el("div", { class: "plan-card-title" }, [
          t.title,
          el("button", {
            class: "plan-card-comment-btn",
            title: "Comments",
            onclick: function (e) { e.stopPropagation(); openPlanTaskComments(t.id, t.title); }
          }, "💬")
        ]),
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

  Object.assign(app, { renderKanban, renderKanbanCard, renderGantt, renderList });
})(window.__gbPlan);
