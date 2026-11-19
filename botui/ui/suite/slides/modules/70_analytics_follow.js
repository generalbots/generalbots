"use strict";
/* Slides follow-along (#2489) — layer 5, DOM-free simulation of audience
 * devices tracking a presenter slide, including a 30 s disconnect/reconnect.
 * There is no WebSocket, no real audience device and no server session here. */
(function (window) {
  var A = window.GBSlidesAnalytics = window.GBSlidesAnalytics || {};
  var F = A.follow = A.follow || {};

  F.addDevice = function (state, input) {
    var values = input || {};
    state.follow = state.follow || { presenterSlide: 0, devices: [] };
    if (!Array.isArray(state.follow.devices)) state.follow.devices = [];
    var device = {
      id: String(values.id || A.uid("device")),
      label: String(values.label || A.tr("slides-analytics-follow-device", "Device")),
      connected: values.connected !== false,
      slide: A.num(values.slide, A.num(state.follow.presenterSlide, 0)),
      disconnectedAt: 0,
      disconnectMs: 0,
      gapBefore: 0,
      lastSyncedAt: A.num(values.ts, Date.now()),
      resyncLatencyMs: 0,
      slideDelta: 0,
      withinBudget: true,
      withinOneSlide: true
    };
    state.follow.devices.push(device);
    return device;
  };

  F.get = function (state, id) {
    var rows = state.follow && Array.isArray(state.follow.devices) ? state.follow.devices : [];
    for (var i = 0; i < rows.length; i += 1) {
      if (rows[i].id === id) return rows[i];
    }
    return null;
  };

  F.setPresenterSlide = function (state, index, ts) {
    var slide = Math.max(0, A.num(index, 0));
    state.follow = state.follow || { presenterSlide: 0, devices: [] };
    state.follow.presenterSlide = slide;
    var now = A.num(ts, Date.now());
    (state.follow.devices || []).forEach(function (device) {
      if (!device.connected) return;
      device.slide = slide;
      device.lastSyncedAt = now;
      device.slideDelta = 0;
      device.withinOneSlide = true;
    });
    return slide;
  };

  F.advance = function (state, ts) {
    return F.setPresenterSlide(state, A.num(state.follow && state.follow.presenterSlide, 0) + 1, ts);
  };

  F.disconnect = function (state, id, ts) {
    var device = F.get(state, id);
    if (!device || !device.connected) return false;
    device.connected = false;
    device.disconnectedAt = A.num(ts, Date.now());
    device.disconnectMs = 0;
    return true;
  };

  F.reconnect = function (state, id, ts, resyncLatencyMs) {
    var device = F.get(state, id);
    if (!device || device.connected) return false;
    var now = A.num(ts, Date.now());
    var presenterSlide = A.num(state.follow && state.follow.presenterSlide, 0);
    device.gapBefore = Math.abs(A.num(device.slide, 0) - presenterSlide);
    device.disconnectMs = Math.max(0, now - A.num(device.disconnectedAt, now));
    device.connected = true;
    device.slide = presenterSlide;
    device.lastSyncedAt = now;
    device.resyncLatencyMs = Math.max(0, A.num(resyncLatencyMs, 0));
    device.slideDelta = Math.abs(A.num(device.slide, 0) - presenterSlide);
    device.withinOneSlide = device.slideDelta <= A.MAX_SLIDE_DELTA;
    device.withinBudget = device.resyncLatencyMs <= A.RESYNC_BUDGET_MS;
    return true;
  };

  F.simulateThirtySecondDrop = function (state, deviceId, startTs) {
    var start = A.num(startTs, A.BASE_TS);
    if (!F.disconnect(state, deviceId, start)) return null;
    F.advance(state, start + 10000);
    F.advance(state, start + 20000);
    F.reconnect(state, deviceId, start + 30000);
    return F.get(state, deviceId);
  };

  F.devices = function (state) {
    var rows = state.follow && Array.isArray(state.follow.devices) ? state.follow.devices : [];
    return rows.slice();
  };

  F.summary = function (state) {
    var rows = F.devices(state);
    var connected = rows.filter(function (device) { return !!device.connected; }).length;
    var maxLatency = rows.reduce(function (value, device) {
      return Math.max(value, A.num(device.resyncLatencyMs, 0));
    }, 0);
    return {
      presenterSlide: A.num(state.follow && state.follow.presenterSlide, 0),
      total: rows.length,
      connected: connected,
      offline: rows.length - connected,
      maxResyncLatencyMs: maxLatency,
      allWithinOneSlide: rows.every(function (device) {
        return !device.connected || device.withinOneSlide !== false;
      }),
      allWithinBudget: maxLatency <= A.RESYNC_BUDGET_MS
    };
  };
})(typeof window !== "undefined" ? window : globalThis);
