"use strict";
(function () {
  var A = window.GBApprovals;
  var api = A.api;
  var t = A.t;

  function value(id) {
    var el = document.getElementById(id);
    return el ? el.value.trim() : "";
  }

  function refresh() {
    return api.listApprovals().then(function (payload) {
      A.state.records = payload.approvals || [];
      var keep = A.state.selectedId;
      var found = keep && A.state.records.some(function (r) { return r.id === keep; });
      var target = found ? keep : (A.state.records.length ? A.state.records[0].id : null);
      if (!target) {
        A.state.selectedId = null;
        A.state.record = null;
        A.render.renderAll();
        return null;
      }
      return api.getApproval(target).then(function (detail) {
        A.state.selectedId = target;
        A.state.record = detail.approval;
        A.render.renderAll();
        return detail.approval;
      });
    }).catch(function (err) {
      A.setStatus(err.message, true);
      return null;
    });
  }

  function select(id) {
    return api.getApproval(id).then(function (payload) {
      A.state.selectedId = id;
      A.state.record = payload.approval;
      A.render.renderAll();
    }).catch(function (err) { A.setStatus(err.message, true); });
  }

  function buildRoute() {
    var reviewers = value("apprReviewers").split(",").map(function (s) { return s.trim(); })
      .filter(function (s) { return s.length > 0; });
    var sla = parseInt(value("apprSla"), 10);
    if (!isFinite(sla) || sla < 1) sla = 3600;
    return [
      {
        id: "reviewers", title: t("approvals-step-reviewers", "Parallel reviewers"),
        mode: "parallel", kind: "review", approvers: reviewers, groups: [],
        sla_secs: sla, escalate_to: "chief-of-staff", quorum: 0
      },
      {
        id: "approver", title: t("approvals-step-approver", "Authorized approver"),
        mode: "sequential", kind: "approval", approvers: [value("apprCfo")],
        groups: [], sla_secs: 0, escalate_to: null, quorum: 0
      },
      {
        id: "esign", title: t("approvals-step-signature", "Electronic signature"),
        mode: "sequential", kind: "signature", approvers: [value("apprSigner")],
        groups: [], sla_secs: 0, escalate_to: null, quorum: 0
      }
    ];
  }

  function create(event) {
    event.preventDefault();
    var input = {
      tenant: "default",
      doc_id: value("apprDocId"),
      doc_class: value("apprDocClass") || "contract",
      title: value("apprDocTitle"),
      owner: value("apprOwner"),
      content: value("apprContent")
    };
    if (!input.doc_id || !input.owner || !input.content) {
      A.setStatus(A.format("approvals-status-decision-error", {
        error: t("approvals-error-required", "Required fields are missing")
      }), true);
      return Promise.resolve(null);
    }
    return api.createApproval(input).then(function (payload) {
      var id = payload.approval.id;
      return api.routeApproval(id, {
        steps: buildRoute(),
        message: value("apprDocTitle"),
        due_at: null
      }).then(function (routed) {
        A.state.selectedId = id;
        A.state.record = routed.approval;
        A.setStatus(A.format("approvals-status-routed", { count: routed.approval.route.length }));
        return refresh();
      });
    }).catch(function (err) {
      A.setStatus(A.format("approvals-status-decision-error", { error: err.message }), true);
      return null;
    });
  }

  function decide(kind) {
    var record = A.state.record;
    if (!record) return Promise.resolve(null);
    return api.decide(record.id, {
      actor: value("apprActor"),
      decision: kind,
      reason: value("apprReason"),
      comment: value("apprComment"),
      version_hash: value("apprHash")
    }).then(function () {
      A.setStatus(t("approvals-status-decision-ok"));
      return refresh();
    }).catch(function (err) {
      A.setStatus(A.format("approvals-status-decision-error", { error: err.message }), true);
      return null;
    });
  }

  function sign() {
    var record = A.state.record;
    if (!record) return Promise.resolve(null);
    return api.sign(record.id, { actor: value("apprActor"), comment: value("apprComment") })
      .then(function () { A.setStatus(t("approvals-msg-signed")); return refresh(); })
      .catch(function (err) {
        A.setStatus(A.format("approvals-status-decision-error", { error: err.message }), true);
        return null;
      });
  }

  function publish() {
    var record = A.state.record;
    if (!record) return Promise.resolve(null);
    return api.publish(record.id).then(function (payload) {
      A.setStatus(A.format("approvals-msg-published", { entry: payload.artifact.records_entry }));
      return refresh();
    }).catch(function (err) {
      A.setStatus(A.format("approvals-status-decision-error", { error: err.message }), true);
      return null;
    });
  }

  function verify() {
    var record = A.state.record;
    if (!record) return Promise.resolve(null);
    return api.verify(record.id).then(function (payload) {
      A.setStatus(A.format("approvals-status-verified", { value: payload.verified }));
    }).catch(function (err) { A.setStatus(err.message, true); });
  }

  function futureTime() {
    return new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString();
  }

  function breach() {
    var record = A.state.record;
    if (!record) return Promise.resolve(null);
    return api.checkSla(record.id, { at: futureTime() }).then(function (payload) {
      A.setStatus(A.format("approvals-status-breach", { count: payload.events.length }));
    }).catch(function (err) { A.setStatus(err.message, true); });
  }

  function sweep() {
    return api.slaSweep({ at: futureTime() }).then(function (payload) {
      A.setStatus(A.format("approvals-status-breach", { count: payload.events.length }));
      return refresh();
    }).catch(function (err) { A.setStatus(err.message, true); });
  }

  function edit() {
    var record = A.state.record;
    if (!record) return Promise.resolve(null);
    return api.updateContent(record.id, {
      content: value("apprContent") || record.version_hash + "-edited"
    }).then(function (payload) {
      if (payload.approval.status === "invalidated") A.setStatus(t("approvals-msg-invalidated"));
      return refresh();
    }).catch(function (err) {
      A.setStatus(A.format("approvals-status-decision-error", { error: err.message }), true);
      return null;
    });
  }

  function onClick(event) {
    var target = event.target.closest ? event.target.closest("[data-action]") : null;
    if (!target) return;
    var action = target.getAttribute("data-action");
    var id = target.getAttribute("data-id");
    if (action === "view") return select(id);
    if (action === "approve") return decide("approve");
    if (action === "reject") return decide("reject");
    if (action === "changes") return decide("request_changes");
    if (action === "sign") return sign();
    if (action === "publish") return publish();
    if (action === "verify") return verify();
    if (action === "breach") return breach();
    if (action === "edit") return edit();
    return null;
  }

  function bind() {
    document.addEventListener("click", onClick);
    var form = document.getElementById("apprForm");
    if (form) form.addEventListener("submit", create);
    var refreshBtn = document.getElementById("apprRefresh");
    if (refreshBtn) refreshBtn.addEventListener("click", refresh);
    var sweepBtn = document.getElementById("apprSlaSweep");
    if (sweepBtn) sweepBtn.addEventListener("click", sweep);
  }

  A.events = { bind: bind, refresh: refresh, select: select };
})();
