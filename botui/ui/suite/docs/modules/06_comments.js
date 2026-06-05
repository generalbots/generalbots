"use strict";

/**
 * Module 06: Comments for Word Processor.
 * Provides: comment threads, replies, resolve/reopen, mentions, anchors.
 */

function commentId() {
  return "cmt-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 8);
}

function createComment(state, anchor, author, text) {
  if (!state.comments) state.comments = [];
  const cmt = {
    id: commentId(),
    anchor: anchor || { start: 0, end: 0 },
    author: author || "Unknown",
    text: String(text || ""),
    timestamp: new Date().toISOString(),
    resolved: false,
    resolvedBy: null,
    resolvedAt: null,
    replies: [],
  };
  state.comments.push(cmt);
  return cmt;
}

function replyToComment(state, commentIdOrRef, author, text) {
  const cmt = findComment(state, commentIdOrRef);
  if (!cmt) return null;
  const reply = {
    id: commentId(),
    author: author || "Unknown",
    text: String(text || ""),
    timestamp: new Date().toISOString(),
  };
  cmt.replies.push(reply);
  return reply;
}

function findComment(state, ref) {
  if (!state.comments) return null;
  if (typeof ref === "string") return state.comments.find((c) => c.id === ref) || null;
  return ref || null;
}

function resolveComment(state, ref, resolver) {
  const cmt = findComment(state, ref);
  if (!cmt) return false;
  cmt.resolved = true;
  cmt.resolvedBy = resolver || "Unknown";
  cmt.resolvedAt = new Date().toISOString();
  return true;
}

function reopenComment(state, ref) {
  const cmt = findComment(state, ref);
  if (!cmt) return false;
  cmt.resolved = false;
  cmt.resolvedBy = null;
  cmt.resolvedAt = null;
  return true;
}

function deleteComment(state, ref) {
  if (!state.comments) return false;
  const idx = state.comments.findIndex((c) => c.id === ref);
  if (idx === -1) return false;
  state.comments.splice(idx, 1);
  return true;
}

function updateComment(state, ref, newText) {
  const cmt = findComment(state, ref);
  if (!cmt) return false;
  cmt.text = String(newText || "");
  cmt.editedAt = new Date().toISOString();
  return true;
}

function commentsForRange(state, start, end) {
  return (state.comments || []).filter((c) => c.anchor && c.anchor.end >= start && c.anchor.start <= end);
}

function unresolvedComments(state) {
  return (state.comments || []).filter((c) => !c.resolved);
}

function resolvedComments(state) {
  return (state.comments || []).filter((c) => c.resolved);
}

function mentionUsers(text) {
  if (!text) return [];
  const matches = String(text).match(/@(\w+)/g) || [];
  return matches.map((m) => m.slice(1));
}

function threadCount(state, ref) {
  const cmt = findComment(state, ref);
  if (!cmt) return 0;
  return 1 + (cmt.replies || []).length;
}

function commentSummary(state) {
  const list = state.comments || [];
  return {
    total: list.length,
    resolved: list.filter((c) => c.resolved).length,
    unresolved: list.filter((c) => !c.resolved).length,
    totalReplies: list.reduce((acc, c) => acc + (c.replies ? c.replies.length : 0), 0),
  };
}

window.DocsComments = {
  createComment,
  replyToComment,
  findComment,
  resolveComment,
  reopenComment,
  deleteComment,
  updateComment,
  commentsForRange,
  unresolvedComments,
  resolvedComments,
  mentionUsers,
  threadCount,
  commentSummary,
};
