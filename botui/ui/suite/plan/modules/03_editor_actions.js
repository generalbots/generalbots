/* Plan app — module 3/4: task editor, toolbar actions and collab (split from plan/plan.js for #1731) */

(function (app) {
  "use strict";
  const {
    APP, DEFAULT_PLAN_ID, STATUSES, PRIORITIES, $, $$, uid, today, clamp,
    Store, View, renderKanban, renderGantt, renderList,
  } = app;

  function openTaskEditor(task) {
    const modal = $("#plan-modal-container");
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
          '<button class="plan-btn plan-btn-secondary" id="plan-comment" onclick="openPlanTaskComments(\'' + task.id + '\', \'' + (task.title || "").replace(/'/g, "\\'") + '\')">💬</button>' +
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
      syncTaskToBackend(Object.assign({ id: task.id }, patch), "update");
      close();
    };
  }

  function promptNewTask(status) {
    const id = uid();
    const t = { id: id, title: "Nova tarefa", status: status || "todo", priority: "medium", start: today(), end: today() + 7, progress: 0, assignee: "", tags: [] };
    Store.addTask(t);
    syncTaskToBackend(t, "create");
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

  function syncTaskToBackend(task, mode) {
    const payload = {
      plan_id: Store.data.id,
      task_id: task.id,
      title: task.title,
      status: task.status,
      priority: task.priority,
      assignee: task.assignee || null,
      start: task.start || null,
      end: task.end || null,
      tags: task.tags || []
    };
    fetch(mode === "create" ? "/api/plan/task" : "/api/plan/task/update", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload)
    }).catch(function (e) { console.error("plan task sync failed", e); });
  }

  async function onAiAssist() {
    const instruction = window.prompt("Descreva o que a IA deve planejar (ex: adicionar etapas de QA):");
    if (!instruction) return;
    const aiBtn = $("#aiAssistBtn");
    if (aiBtn) { aiBtn.disabled = true; aiBtn.textContent = "🤖 ..."; }
    const context = JSON.stringify({ title: Store.data.title, tasks: Store.data.tasks });
    try {
      const res = await fetch("/api/plan/ai/assist", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ instruction: instruction, context: context })
      });
      const data = await res.json();
      if (data.ok && Array.isArray(data.suggestions) && data.suggestions.length) {
        data.suggestions.forEach(function (s) {
          const id = uid();
          const t = {
            id: id,
            title: s.title || "Nova tarefa",
            status: ["todo", "inprogress", "review", "done"].indexOf(s.status) >= 0 ? s.status : "todo",
            priority: ["low", "medium", "high", "urgent"].indexOf(s.priority) >= 0 ? s.priority : "medium",
            assignee: s.assignee || "",
            start: today(),
            end: today() + 7,
            progress: 0,
            tags: []
          };
          Store.addTask(t);
          syncTaskToBackend(t, "create");
        });
        window.alert(data.suggestions.length + " tarefa(s) sugerida(s) adicionada(s).");
      } else {
        window.alert("IA: " + (data.error || "sem sugestões"));
      }
    } catch (e) {
      console.error(e);
      window.alert("Falha ao chamar a IA.");
    } finally {
      if (aiBtn) { aiBtn.disabled = false; aiBtn.textContent = "🤖 IA Assist"; }
    }
  }

  function onExport() {
    const data = JSON.stringify(Store.data, null, 2);
    const blob = new Blob([data], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (Store.data.title || "plan").replace(/\s+/g, "_") + ".json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  async function onImportFile(e) {
    const file = e.target.files && e.target.files[0];
    if (!file) return;
    try {
      const text = await file.text();
      const parsed = JSON.parse(text);
      if (parsed && parsed.tasks) {
        Store.data = parsed;
        Store.data.id = Store.data.id || DEFAULT_PLAN_ID;
        Store.notify();
        Store.data.tasks.forEach(function (t) { syncTaskToBackend(t, "create"); });
        window.alert("Plano importado com " + Store.data.tasks.length + " tarefa(s).");
      } else {
        window.alert("Arquivo de plano inválido.");
      }
    } catch (err) {
      console.error(err);
      window.alert("Arquivo inválido.");
    }
    e.target.value = "";
  }

  function wireToolbar() {
    const newBtn = $("#newTaskBtn");
    if (newBtn) newBtn.addEventListener("click", function () { promptNewTask("todo"); });

    const aiBtn = $("#aiAssistBtn");
    if (aiBtn) aiBtn.addEventListener("click", function () { onAiAssist(); });

    const expBtn = $("#exportBtn");
    if (expBtn) expBtn.addEventListener("click", function () { onExport(); });

    const impBtn = $("#importBtn");
    if (impBtn) {
      let fileInput = document.getElementById("plan-import-input");
      if (!fileInput) {
        fileInput = document.createElement("input");
        fileInput.type = "file";
        fileInput.id = "plan-import-input";
        fileInput.accept = "application/json";
        fileInput.style.display = "none";
        document.body.appendChild(fileInput);
        fileInput.addEventListener("change", function (ev) { onImportFile(ev); });
      }
      impBtn.addEventListener("click", function () { fileInput.click(); });
    }
  }

  function bindPlanCommentsBadge() {
    const btn = document.getElementById("commentsBtn");
    if (btn && window.GBCollabComments) {
      window.GBCollabComments.bindBadge(btn, {
        resourceType: "plan",
        resourceId: String(Store.data.id),
        includeChildren: true
      });
    }
  }

  Object.assign(app, {
    openTaskEditor,
    promptNewTask,
    render,
    initViewTabs,
    initCollab,
    initAuth,
    syncTaskToBackend,
    onAiAssist,
    onExport,
    onImportFile,
    wireToolbar,
    bindPlanCommentsBadge,
  });
})(window.__gbPlan);
