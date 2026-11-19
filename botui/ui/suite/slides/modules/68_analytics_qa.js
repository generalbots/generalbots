"use strict";
/* Slides audience Q&A (#2489) — layer 3, DOM-free moderation model. Questions
 * live in the deck-scoped local state; an explicit policy decides whether the
 * submitter is attributable or anonymous. Nothing leaves the browser. */
(function (window) {
  var A = window.GBSlidesAnalytics = window.GBSlidesAnalytics || {};
  var QA = A.qa = A.qa || {};

  QA.POLICIES = ["attributable", "anonymous"];
  QA.STATUSES = ["pending", "approved", "rejected", "answered"];

  QA.submit = function (state, input) {
    var values = input || {};
    var text = String(values.text || "").trim();
    if (!text) return null;
    var anonymous = values.policy === "anonymous";
    var question = {
      id: A.uid("qa"),
      text: text,
      ts: A.num(values.ts, Date.now()),
      votes: 0,
      voters: [],
      status: "pending",
      answer: "",
      anonymous: anonymous,
      submittedBy: anonymous ? null : {
        viewerId: String(values.viewerId || ""),
        name: String(values.name || ""),
        email: String(values.email || "")
      }
    };
    state.questions = Array.isArray(state.questions) ? state.questions : [];
    state.questions.push(question);
    return question;
  };

  QA.get = function (state, id) {
    var rows = Array.isArray(state.questions) ? state.questions : [];
    for (var i = 0; i < rows.length; i += 1) {
      if (rows[i].id === id) return rows[i];
    }
    return null;
  };

  QA.upvote = function (state, id, viewerId) {
    var question = QA.get(state, id);
    if (!question || question.status === "rejected") return false;
    var voter = A.pseudonym(viewerId);
    if (!Array.isArray(question.voters)) question.voters = [];
    if (question.voters.indexOf(voter) >= 0) return false;
    question.voters.push(voter);
    question.votes = A.num(question.votes, 0) + 1;
    return true;
  };

  QA.moderate = function (state, id, decision) {
    var question = QA.get(state, id);
    if (!question || (decision !== "approve" && decision !== "reject")) return false;
    question.status = decision === "approve" ? "approved" : "rejected";
    return true;
  };

  QA.answer = function (state, id, answer) {
    var question = QA.get(state, id);
    if (!question || question.status === "rejected") return false;
    question.answer = String(answer || "");
    question.status = "answered";
    return true;
  };

  QA.filter = function (state, status) {
    var rows = Array.isArray(state.questions) ? state.questions : [];
    return rows.filter(function (question) { return question.status === status; });
  };

  QA.queue = function (state) { return QA.filter(state, "pending"); };
  QA.approved = function (state) { return QA.filter(state, "approved"); };
  QA.archive = function (state) { return QA.filter(state, "answered"); };

  QA.count = function (state) {
    var rows = Array.isArray(state.questions) ? state.questions : [];
    return rows.filter(function (question) { return question.status !== "rejected"; }).length;
  };

  QA.attribution = function (question) {
    if (!question || question.anonymous || !question.submittedBy) {
      return A.tr("slides-analytics-qa-anonymous-label", "Anonymous attendee");
    }
    var by = question.submittedBy;
    return by.name || by.email || by.viewerId ||
      A.tr("slides-analytics-qa-anonymous-label", "Anonymous attendee");
  };
})(typeof window !== "undefined" ? window : globalThis);
