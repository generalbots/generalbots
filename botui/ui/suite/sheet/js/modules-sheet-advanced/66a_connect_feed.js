"use strict";
/* Sheets data connectivity (#2492) - deterministic local feed and refresh model.
 * window.SheetConnectFeed. Everything here is simulated: the "source" is a
 * fixed in-memory table, refreshes produce deterministic local snapshots, and
 * no credential value is ever accepted, fetched or persisted. */

(function () {
  var C = window.SheetConnectCore;
  var t = C ? C.t : function (key, fallback) { return fallback; };
  var SOURCE_COLUMNS = ["date", "region", "amount", "category"];
  var SOURCE_ROWS = [
    { date: "2026-03-01", region: "north", amount: 120.5, category: "supplies" },
    { date: "2026-03-02", region: "south", amount: 80, category: "travel" },
    { date: "2026-03-03", region: "north", amount: 250.75, category: "services" },
    { date: "2026-03-04", region: "east", amount: 45.2, category: "supplies" },
    { date: "2026-03-05", region: "south", amount: 310, category: "services" },
    { date: "2026-03-06", region: "east", amount: 60.4, category: "travel" },
  ];
  var BASE_TIME = Date.UTC(2026, 2, 1, 6, 0, 0);
  var SECRET_FIELD = /(?:password|passwd|secret|token|api[_-]?key|access[_-]?key|credential)(?:"\s*:\s*"|\s*=\s*")([^"]{4,})/gi;

  function hostOf(url) {
    var raw = String(url == null ? "" : url).trim();
    var match = raw.match(/^[a-z][a-z0-9+.-]*:\/\/([^/?#]+)/i);
    var authority = match ? match[1] : raw.split("/")[0];
    return String(authority || "").split("@").pop().split(":")[0].toLowerCase();
  }

  function activeConnector(target) {
    if (!target.activeConnectorId) return null;
    return (target.connectors || []).filter(function (item) {
      return item.id === target.activeConnectorId;
    })[0] || null;
  }

  function simulatedFeed(connector) {
    var label = connector ? connector.name || connector.type : "local";
    return {
      columns: SOURCE_COLUMNS.slice(),
      rows: SOURCE_ROWS.map(function (row) {
        return SOURCE_COLUMNS.map(function (column) { return row[column]; });
      }),
      lineage: SOURCE_ROWS.map(function () {
        return SOURCE_COLUMNS.map(function (column) {
          return { stepId: "source", stepType: "source", sourceField: column };
        });
      }),
      sourceLabel: label,
    };
  }

  function paramValue(parameters, key) {
    var found = (parameters || []).filter(function (p) { return p.key === key; })[0];
    return found ? found.value : "";
  }

  function applyParameters(value, parameters) {
    return String(value == null ? "" : value).replace(/\{param:([a-zA-Z0-9_-]+)\}/g, function (all, key) {
      var resolved = paramValue(parameters, key);
      return resolved === "" ? all : resolved;
    });
  }

  function sanitizeConfig(config) {
    var result = {};
    Object.keys(config || {}).forEach(function (key) {
      var value = config[key];
      if (/(?:password|passwd|secret|token|key|credential)/i.test(key)) {
        if (typeof value === "string" && value.indexOf("vault://") === 0) result[key] = value;
        return;
      }
      if (typeof value === "string" && value.length <= 500) result[key] = value;
    });
    return result;
  }

  function definition(target) {
    var connector = activeConnector(target);
    var schedule = target.schedule || {};
    return {
      version: 1,
      kind: "sheet-connect-local-preview",
      localOnly: true,
      connector: connector ? {
        id: connector.id, name: connector.name, type: connector.type,
        config: sanitizeConfig(connector.config),
      } : null,
      parameters: (target.parameters || []).map(function (p) {
        return { key: p.key, kind: p.kind, labelKey: p.labelKey, defaultValue: p.defaultValue, value: p.value };
      }),
      steps: (target.steps || []).map(function (step) {
        return { id: step.id, type: step.type, enabled: step.enabled, config: sanitizeConfig(step.config) };
      }),
      schedule: {
        frequency: schedule.frequency, hour: schedule.hour,
        timezone: schedule.timezone, notifyOnFailure: Boolean(schedule.notifyOnFailure),
      },
      note: "Local only - simulated feed; no network, Vault or server calls.",
    };
  }

  function mockExport(target) {
    return {
      definition: definition(target),
      snapshot: target.snapshot || null,
      audit: (target.audit || []).map(function (entry) {
        return { time: entry.time, status: entry.status, rows: entry.rows, error: entry.error };
      }),
    };
  }

  function findSecrets(label, value) {
    var findings = [];
    var source = typeof value === "string" ? value : JSON.stringify(value == null ? null : value);
    SECRET_FIELD.lastIndex = 0;
    var match = SECRET_FIELD.exec(source);
    while (match) {
      if (match[1].indexOf("vault://") !== 0) findings.push(label + ": " + match[1].slice(0, 40));
      match = SECRET_FIELD.exec(source);
    }
    return findings;
  }

  function noCredentialScan(target) {
    var definitionText = JSON.stringify(definition(target));
    var exportText = JSON.stringify(mockExport(target));
    var workbookText = JSON.stringify(target);
    var findings = findSecrets("workbook", workbookText)
      .concat(findSecrets("definition", definitionText))
      .concat(findSecrets("export", exportText));
    return { pass: findings.length === 0, findings: findings };
  }

  function nextRefresh(iso, schedule) {
    var from = new Date(iso);
    if (Number.isNaN(from.getTime())) return null;
    var next = new Date(from.getTime());
    var hour = Number.isFinite(Number(schedule.hour)) ? Number(schedule.hour) : 6;
    next.setUTCHours(hour, 0, 0, 0);
    var frequency = C ? C.oneOf(schedule.frequency, C.FREQUENCIES, "daily") : "daily";
    if (frequency === "daily") {
      if (next.getTime() <= from.getTime()) next.setUTCDate(next.getUTCDate() + 1);
    } else if (frequency === "weekly") {
      next.setUTCDate(next.getUTCDate() + 7);
    } else {
      next.setUTCMonth(next.getUTCMonth() + 1);
    }
    return next.toISOString();
  }

  function stamp(target) {
    return new Date(BASE_TIME + (target.audit || []).length * 86400000).toISOString();
  }

  function refreshNow(target) {
    var connector = activeConnector(target);
    var schedule = target.schedule;
    var time = stamp(target);
    if (!connector) {
      return {
        ok: false,
        errorKey: "sheet-connect-refresh-no-connector",
        errorFallback: "Select an active connector before refreshing.",
      };
    }
    if (target.simulateFailure) {
      var error = t("sheet-connect-refresh-sim-error", "Simulated feed error (local).");
      schedule.lastStatus = "failed";
      schedule.lastError = error;
      schedule.lastRefreshAt = time;
      schedule.nextRefreshAt = nextRefresh(time, schedule);
      target.audit.push({
        id: "audit-" + target.audit.length, time: time, status: "failed",
        rows: 0, error: error, connectorId: connector.id,
      });
      return { ok: false, error: error, time: time };
    }
    var result = window.SheetConnectSteps
      ? window.SheetConnectSteps.applyAll(target, connector)
      : { table: simulatedFeed(connector), applied: 0 };
    var table = result.table;
    target.snapshot = {
      createdAt: time,
      connectorId: connector.id,
      sourceLabel: table.sourceLabel,
      columns: table.columns.slice(),
      rows: table.rows.map(function (row) { return row.slice(); }),
      lineage: table.lineage.map(function (line) { return line.map(function (cell) { return Object.assign({}, cell); }); }),
      rowCount: table.rows.length,
      stepsApplied: result.applied,
      stale: false,
    };
    schedule.lastStatus = "success";
    schedule.lastError = "";
    schedule.lastRefreshAt = time;
    schedule.nextRefreshAt = nextRefresh(time, schedule);
    target.audit.push({
      id: "audit-" + target.audit.length, time: time, status: "success",
      rows: table.rows.length, error: "", connectorId: connector.id,
    });
    return { ok: true, rows: table.rows.length, time: time };
  }

  window.SheetConnectFeed = {
    SOURCE_COLUMNS: SOURCE_COLUMNS,
    hostOf: hostOf,
    activeConnector: activeConnector,
    simulatedFeed: simulatedFeed,
    paramValue: paramValue,
    applyParameters: applyParameters,
    sanitizeConfig: sanitizeConfig,
    definition: definition,
    mockExport: mockExport,
    noCredentialScan: noCredentialScan,
    nextRefresh: nextRefresh,
    stamp: stamp,
    refreshNow: refreshNow,
  };
})();
