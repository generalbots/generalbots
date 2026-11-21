"use strict";
/**
 * 02b_requests.js - Requests that can never hold the connection pool.
 *
 * The window has a handful of connections to the server. A background poll
 * that hangs keeps one of them, so polls here always carry a deadline, skip
 * a tick while the previous request is in flight, share at most two
 * connections between them, back off after a failure and sleep while the
 * window is hidden. The conversation list and messages live here too: they
 * carry a deadline and report failure instead of an empty answer.
 */
(function () {
    var A = window.GBAssistant;
    var POLL_TIMEOUT = 8000, POLL_SLOTS = 2, POLL_BACKOFF = 30000;
    var LIST_TIMEOUT = 15000, PAGE = 50, PAGE_CAP = 200;
    var VM_STATES = ["stopped", "starting", "unavailable"];

    /** JSON request with a deadline: resolves {ok, status, data}, rejects on
        network error or timeout (err.timeout). The body is read inside the
        deadline, so a stalled body cannot hold the connection either. */
    A.fetchJSON = function (path, init, ms) {
        var ctrl = new AbortController();
        var timer = setTimeout(function () { ctrl.abort(); }, ms || LIST_TIMEOUT);
        return A.apiFetch(path, Object.assign({}, init || {}, { signal: ctrl.signal }))
            .then(function (r) {
                return r.text().then(function (text) {
                    var data = null;
                    try { data = text ? JSON.parse(text) : null; } catch (e) { data = null; }
                    return { ok: r.ok, status: r.status, data: data };
                });
            })
            .catch(function (err) {
                if (!ctrl.signal.aborted) { throw err; }
                var late = new Error("timeout");
                late.timeout = true;
                throw late;
            })
            .finally(function () { clearTimeout(timer); });
    };

    /** The agent workspace state a quick answer carries: a 503, or a 200
        whose body is {"status": "stopped" | "starting" | "unavailable"}. */
    A.vmState = function (res) {
        if (!res || res.skipped) { return null; }
        if (res.status === 503) { return "unavailable"; }
        var s = res.data && typeof res.data.status === "string" ? res.data.status.toLowerCase() : "";
        return VM_STATES.indexOf(s) >= 0 ? s : null;
    };

    var inflight = {}, busy = 0;
    /** Background request. Never rejects: resolves {ok, status, data,
        failed, timeout}. A call while the same request is in flight gets
        that request's answer; when both poll connections are taken it
        resolves {skipped: true} at once instead of queueing. */
    A.pollJSON = function (url, opts) {
        opts = opts || {};
        var key = opts.inflightKey || url;
        if (inflight[key]) { return inflight[key]; }
        if (busy >= POLL_SLOTS) { return Promise.resolve({ skipped: true }); }
        busy++;
        var p = A.fetchJSON(url, opts.init, opts.timeout || POLL_TIMEOUT)
            .then(function (res) { res.failed = !res.ok; return res; },
                function (err) { return { ok: false, status: 0, failed: true, timeout: !!err.timeout }; })
            .then(function (res) { busy--; delete inflight[key]; return res; });
        inflight[key] = p;
        return p;
    };

    /** Chained-timeout poller. `run` returns (or resolves to) the delay
        before the next tick in ms, or nothing for `every`; a rejection waits
        `backoff`. It never runs twice at once, never runs while the window is
        hidden or `when()` is false, and resumes on its own when they change
        (call wake() after changing what `when` reads). */
    A.poller = function (run, opts) {
        var every = opts.every, backoff = opts.backoff || POLL_BACKOFF;
        var on = false, going = false, timer = 0, nextAt = 0;
        function due() { return on && !document.hidden && (!opts.when || opts.when()); }
        function plan(ms) {
            clearTimeout(timer);
            timer = 0;
            nextAt = Date.now() + ms;
            if (due()) { timer = setTimeout(tick, ms); }
        }
        function tick() {
            timer = 0;
            if (!due() || going) { return; }
            going = true;
            Promise.resolve().then(run).then(function (ms) {
                return typeof ms === "number" ? ms : every;
            }, function () { return backoff; }).then(function (ms) {
                going = false;
                if (on) { plan(ms); }
            });
        }
        function wake() {
            if (!due()) { clearTimeout(timer); timer = 0; return; }
            if (!timer && !going) { timer = setTimeout(tick, Math.max(0, nextAt - Date.now())); }
        }
        document.addEventListener("visibilitychange", wake);
        return {
            start: function (delay) { if (on) { wake(); return; } on = true; plan(delay || 0); },
            stop: function () { on = false; clearTimeout(timer); timer = 0; },
            now: function () { if (on && !going) { plan(0); } },
            wake: wake,
            running: function () { return on; }
        };
    };

    /* ---- Conversation list (paged) ---- */

    var listSeq = 0, offsetPaging = false, moreBusy = false;
    A.sessionsMore = false;
    A.sessionsTotal = null;

    function listUrl(offset, limit) {
        return "/api/chat/history/sessions?limit=" + limit + (offset ? "&offset=" + offset : "") +
            "&bot_name=" + encodeURIComponent(A.botName);
    }
    function mapSession(s) {
        /* Server returns session_id, not id */
        return { id: s.session_id || s.id, title: s.title, updated_at: s.updated_at || s.created_at };
    }
    /** What the page says about the rest: total / has_more when the server
        pages by offset, otherwise a full page means there may be more. */
    function noteMore(data, got, asked) {
        offsetPaging = typeof data.has_more === "boolean" || typeof data.total === "number";
        A.sessionsTotal = typeof data.total === "number" ? data.total : null;
        if (typeof data.has_more === "boolean") { A.sessionsMore = data.has_more; }
        else if (A.sessionsTotal !== null) { A.sessionsMore = A.sessions.length < A.sessionsTotal; }
        else { A.sessionsMore = got >= asked && asked < PAGE_CAP; }
    }

    /** Reload the list from the top, keeping as many rows as were shown. */
    A.loadSessions = async function () {
        var seq = ++listSeq, old = A.sessions || [];
        var limit = Math.min(PAGE_CAP, Math.max(PAGE, old.length));
        try {
            var res = await A.fetchJSON(listUrl(0, limit), null, LIST_TIMEOUT);
            if (seq !== listSeq || !res.ok || !res.data) { return; }
            var fresh = (res.data.sessions || []).map(mapSession), seen = {};
            fresh.forEach(function (s) { seen[s.id] = true; });
            var got = fresh.length;
            if (old.length > limit) {
                fresh = fresh.concat(old.slice(limit).filter(function (s) { return !seen[s.id]; }));
            }
            A.sessions = fresh;
            noteMore(res.data, got, limit);
        } catch (e) {
            console.warn("[assistant] sessions fetch failed:", e);
        }
    };

    /** Next page of the list; resolves true when rows were added. */
    A.loadMoreSessions = async function () {
        if (moreBusy || !A.sessionsMore) { return false; }
        moreBusy = true;
        var seq = ++listSeq, have = (A.sessions || []).length;
        var limit = offsetPaging ? PAGE : Math.min(PAGE_CAP, have + PAGE);
        try {
            var res = await A.fetchJSON(listUrl(offsetPaging ? have : 0, limit), null, LIST_TIMEOUT);
            if (seq !== listSeq || !res.ok || !res.data) { return false; }
            var page = (res.data.sessions || []).map(mapSession);
            if (offsetPaging) {
                var seen = {};
                A.sessions.forEach(function (s) { seen[s.id] = true; });
                A.sessions = A.sessions.concat(page.filter(function (s) { return !seen[s.id]; }));
            } else {
                A.sessions = page;
            }
            noteMore(res.data, page.length, limit);
            return A.sessions.length > have;
        } catch (e) {
            console.warn("[assistant] more sessions failed:", e);
            return false;
        } finally {
            moreBusy = false;
        }
    };

    /** Messages of one conversation. Rejects on a non-OK answer, a network
        error or after 15 s, so the caller can offer to try again. The step
        cards are kept only while that conversation is still the open one. */
    A.loadSessionMessages = async function (sessId) {
        var res = await A.fetchJSON("/api/chat/history/sessions/" + encodeURIComponent(sessId) +
            "/messages?limit=500&offset=0&include_activities=1", null, LIST_TIMEOUT);
        if (!res.ok) { throw new Error("HTTP " + res.status); }
        var data = res.data || {};
        if (sessId === A.currentSessionId) { A._lastHistoryActivities = data.activities || []; }
        return data.messages || [];
    };
})();
