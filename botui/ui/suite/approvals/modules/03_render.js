"use strict";
(function () {
  var A = window.GBApprovals;
  var esc = A.esc;
  var t = A.t;

  function shortHash(hash) {
    if (!hash) return "";
    return String(hash).substring(0, 12);
  }

  var DECISION_KEYS = {
    approve: "approvals-decision-approve",
    reject: "approvals-decision-reject",
    request_changes: "approvals-decision-request-changes"
  };

  function renderList(records) {
    var host = document.getElementById("apprList");
    if (!host) return;
    if (!records.length) {
      host.innerHTML = '<p class="gb-appr-empty">' + esc(t("approvals-empty-list")) + "</p>";
      return;
    }
    var rows = records.map(function (record) {
      var step = record.route[record.current_step];
      var stepTitle = step ? esc(step.id) : "—";
      return '<tr data-id="' + esc(record.id) + '">' +
        '<td>' + esc(record.title || record.doc_id) + '<br><span class="gb-appr-mono">' + esc(record.doc_id) + '</span></td>' +
        '<td><span class="gb-appr-badge ' + esc(record.status) + '">' + esc(A.statusLabel(record.status)) + '</span></td>' +
        '<td>' + stepTitle + '</td>' +
        '<td class="gb-appr-mono">' + esc(shortHash(record.version_hash)) + '</td>' +
        '<td><button type="button" class="gb-appr-btn" data-action="view" data-id="' + esc(record.id) + '">' + esc(t("approvals-action-view")) + '</button></td>' +
        '</tr>';
    }).join("");
    host.innerHTML = '<table class="gb-appr-table"><thead><tr>' +
      '<th>' + esc(t("approvals-col-doc")) + '</th>' +
      '<th>' + esc(t("approvals-col-status")) + '</th>' +
      '<th>' + esc(t("approvals-col-step")) + '</th>' +
      '<th>' + esc(t("approvals-col-version")) + '</th>' +
      '<th>' + esc(t("approvals-col-actions")) + '</th>' +
      '</tr></thead><tbody>' + rows + '</tbody></table>';
  }

  function renderDetail(record) {
    var host = document.getElementById("apprDetail");
    if (!host) return;
    if (!record) {
      host.innerHTML = '<p class="gb-appr-empty">' + esc(t("approvals-no-selection")) + "</p>";
      return;
    }
    var step = record.route[record.current_step];
    var published = record.published;
    var html = '<div><span class="gb-appr-badge ' + esc(record.status) + '">' +
      esc(A.statusLabel(record.status)) + '</span></div>' +
      '<div><strong>' + esc(t("approvals-label-version")) + ':</strong> <span class="gb-appr-mono">' + esc(record.version_hash) + '</span></div>' +
      '<div><strong>' + esc(t("approvals-label-step")) + ':</strong> ' + esc(step ? step.id : "—") + '</div>' +
      '<div><strong>' + esc(t("approvals-label-due")) + ':</strong> ' + esc(record.due_at || "—") + '</div>';
    if (published) {
      html += '<div><strong>' + esc(t("approvals-label-records")) + ':</strong> ' + esc(published.records_entry) + '</div>' +
        '<div><strong>' + esc(t("approvals-label-signed-hash")) + ':</strong> <span class="gb-appr-mono">' + esc(published.signed_hash) + '</span></div>';
    }
    host.innerHTML = html;
  }

  function renderTrail(record) {
    var host = document.getElementById("apprTrail");
    if (!host) return;
    if (!record || !record.decisions.length) {
      host.innerHTML = '<p class="gb-appr-empty">' + esc(t("approvals-empty-trail")) + "</p>";
      return;
    }
    host.innerHTML = record.decisions.map(function (decision) {
      var key = DECISION_KEYS[decision.decision];
      var label = key ? t(key) : String(decision.decision);
      return '<div class="gb-appr-trail-row">' +
        '<div><strong>' + esc(decision.actor) + '</strong> ' + esc(label) +
        ' · ' + esc(decision.step_id) + '</div>' +
        (decision.reason ? '<div>' + esc(decision.reason) + '</div>' : "") +
        (decision.comment ? '<div>' + esc(decision.comment) + '</div>' : "") +
        '<div class="gb-appr-mono">' + esc(t("approvals-label-version")) + ': ' + esc(shortHash(decision.version_hash)) +
        ' · ' + esc(t("approvals-label-signature")) + ': ' + esc(decision.signature) + '</div>' +
        '</div>';
    }).join("");
  }

  function renderActions(record) {
    var host = document.getElementById("apprActions");
    if (!host) return;
    if (!record) { host.innerHTML = ""; return; }
    var step = record.route[record.current_step];
    var actor = step && step.approvers.length ? step.approvers[0] : record.owner;
    var disabled = record.status === "published" || record.status === "invalidated";
    var block = function (disabledFlag) { return disabled || disabledFlag ? " disabled" : ""; };
    host.innerHTML =
      '<div class="gb-appr-actions-row">' +
      '<label for="apprActor">' + esc(t("approvals-field-actor")) + '</label>' +
      '<input id="apprActor" type="text" value="' + esc(actor) + '" />' +
      '<label for="apprReason">' + esc(t("approvals-field-reason")) + '</label>' +
      '<input id="apprReason" type="text" />' +
      '<label for="apprComment">' + esc(t("approvals-field-comment")) + '</label>' +
      '<input id="apprComment" type="text" />' +
      '<label for="apprHash">' + esc(t("approvals-field-version-hash")) + '</label>' +
      '<input id="apprHash" type="text" value="' + esc(record.version_hash) + '" />' +
      '</div>' +
      '<div class="gb-appr-actions-row">' +
      '<button type="button" class="gb-appr-btn gb-appr-btn-primary" data-action="approve"' + block() + '>' + esc(t("approvals-action-approve")) + '</button>' +
      '<button type="button" class="gb-appr-btn gb-appr-btn-danger" data-action="reject"' + block() + '>' + esc(t("approvals-action-reject")) + '</button>' +
      '<button type="button" class="gb-appr-btn" data-action="changes"' + block() + '>' + esc(t("approvals-action-request-changes")) + '</button>' +
      '<button type="button" class="gb-appr-btn" data-action="sign"' + block() + '>' + esc(t("approvals-action-sign")) + '</button>' +
      '<button type="button" class="gb-appr-btn" data-action="publish"' + block(record.status !== "signed") + '>' + esc(t("approvals-action-publish")) + '</button>' +
      '<button type="button" class="gb-appr-btn" data-action="verify"' + block() + '>' + esc(t("approvals-action-verify")) + '</button>' +
      '<button type="button" class="gb-appr-btn" data-action="breach"' + block() + '>' + esc(t("approvals-action-simulate-breach")) + '</button>' +
      '<button type="button" class="gb-appr-btn" data-action="edit"' + block() + '>' + esc(t("approvals-action-edit-content")) + '</button>' +
      '</div>';
  }

  function renderAll() {
    renderList(A.state.records);
    renderDetail(A.state.record);
    renderTrail(A.state.record);
    renderActions(A.state.record);
  }

  A.render = {
    renderList: renderList,
    renderDetail: renderDetail,
    renderTrail: renderTrail,
    renderActions: renderActions,
    renderAll: renderAll
  };
})();
