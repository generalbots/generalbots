"use strict";
/* Sheets data connectivity (#2492) - connector gallery and host allowlist.
 * window.SheetConnectConnectors. Credential fields accept ONLY a vault://
 * reference; no secret value is ever read into state or shown in the UI. */

(function () {
  var C = window.SheetConnectCore;
  var F = window.SheetConnectFeed;
  var U = window.SheetConnectUi;
  var t = C.t;

  var TYPES = [
    { value: "csv", key: "sheet-connect-type-csv", fallback: "CSV / TSV URL" },
    { value: "rest", key: "sheet-connect-type-rest", fallback: "REST / JSON (paginated)" },
    { value: "postgres", key: "sheet-connect-type-postgres", fallback: "PostgreSQL (Vault credential)" },
    { value: "mysql", key: "sheet-connect-type-mysql", fallback: "MySQL (Vault credential)" },
    { value: "drive", key: "sheet-connect-type-drive", fallback: "Drive file" },
    { value: "gsheet", key: "sheet-connect-type-gsheet", fallback: "Google Sheets import" },
  ];
  var VAULT_TYPES = ["postgres", "mysql"];

  var draft = newDraft();
  var editingId = null;

  function newDraft(type) {
    return {
      name: "", type: type || "csv", url: "", param: "page", path: "",
      sheet: "", credentialRef: "", host: "",
    };
  }

  function labelOf(type) {
    var found = TYPES.filter(function (item) { return item.value === type; })[0];
    return found ? t(found.key, found.fallback) : type;
  }

  function validateConnector(fields, target) {
    var errors = [];
    var value = fields || {};
    var type = value.type;
    if (!String(value.name || "").trim()) {
      errors.push({ key: "sheet-connect-validation-name", fallback: "Enter a connector name." });
    }
    if (C.CONNECTOR_TYPES.indexOf(type) < 0) {
      errors.push({ key: "sheet-connect-validation-type", fallback: "Choose a connector type." });
    }
    if (type === "csv" || type === "rest") {
      var url = String(value.url || "").trim();
      var host = F.hostOf(url);
      if (!/^https?:\/\//i.test(url) || !host) {
        errors.push({ key: "sheet-connect-validation-url", fallback: "Enter a valid http(s) URL." });
      } else if (type === "rest" && (target.allowlist || []).indexOf(host) < 0) {
        errors.push({
          key: "sheet-connect-allowlist-required", fallback: "Unapproved host: { $host }. Approve it first.",
          params: { host: host }, host: host,
        });
        errors.push({
          key: "sheet-connect-host-blocked", fallback: "An unapproved host cannot be added as a connector.",
        });
      }
    }
    if (VAULT_TYPES.indexOf(type) >= 0) {
      var reference = String(value.credentialRef || "").trim();
      if (reference.indexOf("vault://") !== 0 || reference.length <= 8) {
        errors.push({
          key: "sheet-connect-validation-vault",
          fallback: "Enter a vault:// credential reference; a secret value is not allowed.",
        });
      }
      Object.keys(value).forEach(function (key) {
        var secret = /(?:password|passwd|secret|token|key|credential)/i.test(key);
        var content = String(value[key] || "");
        if (secret && content && content.indexOf("vault://") !== 0) {
          errors.push({
            key: "sheet-connect-validation-secret-value",
            fallback: "Credential values are not allowed; use a vault:// reference only.",
          });
        }
      });
    }
    if (type === "drive" || type === "gsheet") {
      if (!String(value.path || "").trim()) {
        errors.push({ key: "sheet-connect-validation-path", fallback: "Enter a file path or identifier." });
      }
    }
    return { ok: errors.length === 0, errors: errors };
  }

  function configOf(fields) {
    if (fields.type === "csv" || fields.type === "rest") {
      return { url: String(fields.url || "").trim(), param: String(fields.param || "page").trim(), host: F.hostOf(fields.url) };
    }
    if (VAULT_TYPES.indexOf(fields.type) >= 0) {
      return { credentialRef: String(fields.credentialRef || "").trim() };
    }
    return { path: String(fields.path || "").trim(), sheet: String(fields.sheet || "").trim() };
  }

  function makeId() {
    return "conn-" + Date.now().toString(36) + "-" + Math.floor(Math.random() * 1e4).toString(36);
  }

  function create(fields, editingIdValue, ctx) {
    var valid = validateConnector(fields, ctx.state());
    if (!valid.ok) {
      if (ctx) ctx.setStatus(t(valid.errors[0].key, valid.errors[0].fallback, valid.errors[0].params), true);
      return valid;
    }
    var connector = {
      id: editingIdValue || makeId(), name: String(fields.name).trim().slice(0, 80),
      type: fields.type, config: configOf(fields), createdAt: new Date().toISOString(),
    };
    C.mutate(function (state) {
      if (editingIdValue) {
        state.connectors = state.connectors.map(function (item) { return item.id === editingIdValue ? connector : item; });
      } else {
        state.connectors.push(connector);
      }
      if (!state.activeConnectorId) state.activeConnectorId = connector.id;
    });
    if (ctx) {
      ctx.setStatus(t("sheet-connect-connector-saved", "Connector saved locally: { $name }.", { name: connector.name }));
      ctx.rerender();
    }
    return { ok: true, connector: connector };
  }

  function save(ctx) {
    var result = create(draft, editingId, ctx);
    if (result.ok) {
      editingId = null;
      draft = newDraft(draft.type);
    }
    return result;
  }

  function remove(id, ctx) {
    C.mutate(function (state) {
      state.connectors = state.connectors.filter(function (item) { return item.id !== id; });
      if (state.activeConnectorId === id) {
        state.activeConnectorId = state.connectors.length ? state.connectors[0].id : null;
      }
    });
    if (editingId === id) { editingId = null; draft = newDraft(); }
    if (ctx) {
      ctx.setStatus(t("sheet-connect-connector-deleted", "Connector removed."));
      ctx.rerender();
    }
  }

  function activate(id, ctx) {
    C.mutate(function (state) { state.activeConnectorId = id; });
    if (ctx) {
      ctx.setStatus(t("sheet-connect-connector-active", "Active connector changed."));
      ctx.rerender();
    }
  }

  function approve(rawHost, ctx) {
    var host = F.hostOf(rawHost);
    if (!host) {
      if (ctx) ctx.setStatus(t("sheet-connect-validation-host", "Enter a host name."), true);
      return false;
    }
    var added = false;
    C.mutate(function (state) {
      if (state.allowlist.indexOf(host) < 0) { state.allowlist.push(host); added = true; }
    });
    if (ctx) {
      ctx.setStatus(t(added ? "sheet-connect-allowlist-added" : "sheet-connect-allowlist-exists",
        added ? "Host approved: { $host }." : "Host is already approved.", { host: host }));
      ctx.rerender();
    }
    return added;
  }

  function approveHost(ctx) {
    var added = approve(draft.host, ctx);
    draft.host = "";
    return added;
  }

  function unapproveHost(host, ctx) {
    C.mutate(function (state) {
      state.allowlist = state.allowlist.filter(function (item) { return item !== host; });
    });
    if (ctx) {
      ctx.setStatus(t("sheet-connect-allowlist-removed", "Host removed from the allowlist."));
      ctx.rerender();
    }
  }

  function renderTypeButtons(container, ctx) {
    container.appendChild(U.title("sheet-connect-connector-gallery", "Connector gallery"));
    var row = U.el("div", "ss-connect-type-row");
    TYPES.forEach(function (item) {
      var button = U.button(item.key, item.fallback, function () {
        draft = newDraft(item.value);
        editingId = null;
        ctx.rerender();
      }, "ss-connect-type-btn" + (draft.type === item.value ? " is-active" : ""));
      row.appendChild(button);
    });
    container.appendChild(row);
  }

  function renderForm(container, ctx) {
    container.appendChild(U.title("sheet-connect-form-title", "New connector"));
    var form = U.el("div", "ss-connect-form");
    var name = U.field("sheet-connect-connector-name", "Connector name", "text", draft.name);
    name.input.addEventListener("input", function () { draft.name = name.input.value; });
    form.appendChild(name);
    var type = U.select("sheet-connect-connector-type", "Connector type", TYPES, draft.type);
    type.input.addEventListener("change", function () {
      draft = newDraft(type.input.value);
      editingId = null;
      ctx.rerender();
    });
    form.appendChild(type);

    if (draft.type === "csv" || draft.type === "rest") {
      var url = U.field("sheet-connect-field-url", "Feed URL", "url", draft.url,
        "sheet-connect-placeholder-url", "https://data.example.com/feed");
      url.input.addEventListener("input", function () { draft.url = url.input.value; });
      form.appendChild(url);
      if (draft.type === "rest") {
        var param = U.field("sheet-connect-field-pagination", "Pagination parameter", "text", draft.param);
        param.input.addEventListener("input", function () { draft.param = param.input.value; });
        form.appendChild(param);
      }
    } else if (VAULT_TYPES.indexOf(draft.type) >= 0) {
      var reference = U.field("sheet-connect-field-vault-ref", "Vault credential reference", "text",
        draft.credentialRef, "sheet-connect-placeholder-vault", "vault://secret/gbo/...");
      reference.input.addEventListener("input", function () { draft.credentialRef = reference.input.value; });
      form.appendChild(reference);
      form.appendChild(U.note("sheet-connect-vault-note",
        "Reference only - never paste a password or key. Stored as a vault:// reference."));
    } else {
      var path = U.field("sheet-connect-field-path", "File path or identifier", "text", draft.path);
      path.input.addEventListener("input", function () { draft.path = path.input.value; });
      form.appendChild(path);
      if (draft.type === "gsheet") {
        var sheet = U.field("sheet-connect-field-sheet", "Sheet name or range", "text", draft.sheet);
        sheet.input.addEventListener("input", function () { draft.sheet = sheet.input.value; });
        form.appendChild(sheet);
      }
    }
    var actions = U.el("div", "ss-connect-actions");
    actions.appendChild(U.button("sheet-connect-save", "Save connector", function () { save(ctx); }));
    if (editingId) {
      actions.appendChild(U.button("sheet-connect-cancel", "Cancel", function () {
        editingId = null; draft = newDraft(); ctx.rerender();
      }));
    }
    form.appendChild(actions);
    container.appendChild(form);
  }

  function renderAllowlist(container, ctx, target) {
    container.appendChild(U.title("sheet-connect-allowlist", "Host allowlist"));
    container.appendChild(U.note("sheet-connect-allowlist-note",
      "Only approved hosts may be used by REST connectors."));
    var list = U.el("div", "ss-connect-host-list");
    if (!target.allowlist.length) list.appendChild(U.note("sheet-connect-allowlist-empty", "No approved hosts yet."));
    target.allowlist.forEach(function (host) {
      var row = U.el("div", "ss-connect-host-row");
      row.appendChild(U.el("span", "ss-connect-host-name", host));
      row.appendChild(U.iconButton("sheet-connect-allowlist-remove", "Remove host", "\u00d7", function () {
        unapproveHost(host, ctx);
      }));
      list.appendChild(row);
    });
    container.appendChild(list);
    var add = U.el("div", "ss-connect-inline");
    var input = U.field("sheet-connect-field-host", "Host", "text", draft.host);
    input.input.addEventListener("input", function () { draft.host = input.input.value; });
    input.input.addEventListener("keydown", function (event) {
      if (event.key === "Enter") { event.preventDefault(); approveHost(ctx); }
    });
    add.appendChild(input);
    add.appendChild(U.button("sheet-connect-allowlist-add", "Approve host", function () { approveHost(ctx); }));
    container.appendChild(add);
  }

  function renderSaved(container, ctx, target) {
    container.appendChild(U.title("sheet-connect-connector-list", "Saved connectors"));
    if (!target.connectors.length) {
      container.appendChild(U.note("sheet-connect-connector-empty", "No connectors yet. Choose a type above to add one."));
      return;
    }
    var list = U.el("div", "ss-connect-connector-list");
    target.connectors.forEach(function (connector) {
      var row = U.el("div", "ss-connect-connector-row");
      var info = U.el("div", "ss-connect-connector-info");
      info.appendChild(U.el("strong", "", connector.name));
      info.appendChild(U.el("span", "ss-connect-connector-sub", labelOf(connector.type)));
      row.appendChild(info);
      if (target.activeConnectorId === connector.id) {
        row.appendChild(U.el("span", "ss-connect-badge", t("sheet-connect-active", "Active")));
      } else {
        row.appendChild(U.button("sheet-connect-select", "Use for query", function () { activate(connector.id, ctx); }));
      }
      row.appendChild(U.button("sheet-connect-edit", "Edit", function () {
        editingId = connector.id;
        draft = {
          name: connector.name, type: connector.type, url: connector.config.url || "",
          param: connector.config.param || "page", path: connector.config.path || "",
          sheet: connector.config.sheet || "", credentialRef: connector.config.credentialRef || "", host: "",
        };
        ctx.rerender();
      }));
      row.appendChild(U.button("sheet-connect-delete", "Delete", function () { remove(connector.id, ctx); }));
      list.appendChild(row);
    });
    container.appendChild(list);
  }

  function render(container, ctx) {
    var target = ctx.state();
    renderTypeButtons(container, ctx);
    renderForm(container, ctx);
    renderAllowlist(container, ctx, target);
    renderSaved(container, ctx, target);
  }

  window.SheetConnectConnectors = {
    TYPES: TYPES, VAULT_TYPES: VAULT_TYPES,
    validateConnector: validateConnector, configOf: configOf,
    create: create, remove: remove, activate: activate,
    approve: approve, unapprove: unapproveHost,
    render: render, newDraft: newDraft,
    reset: function () { editingId = null; draft = newDraft(); },
  };
})();
