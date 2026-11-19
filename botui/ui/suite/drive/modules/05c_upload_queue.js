/* Drive Module — 05c Interrupted upload queue (#2510)
 *
 * A phone switches from Wi-Fi to cellular, or the user backgrounds the tab, and
 * an in-flight upload dies. Before this queue the bytes existed only in the
 * page, so the write was lost without a trace: the file never appeared in the
 * folder and nothing said why.
 *
 * Every upload is written to IndexedDB first. Success deletes the record; a
 * reload that still finds records offers them back with Retry, so the user can
 * finish the transfer instead of hunting for the photo again.
 */
"use strict";

var GBC_QUEUE_DB = "gb-drive-uploads";
var GBC_QUEUE_STORE = "pending";

function gbcQueueOpen() {
    return new Promise(function (resolve, reject) {
        if (!window.indexedDB) {
            reject(new Error("indexedDB unavailable"));
            return;
        }
        var request = window.indexedDB.open(GBC_QUEUE_DB, 1);
        request.onupgradeneeded = function () {
            var db = request.result;
            if (!db.objectStoreNames.contains(GBC_QUEUE_STORE)) {
                db.createObjectStore(GBC_QUEUE_STORE, { keyPath: "id" });
            }
        };
        request.onsuccess = function () { resolve(request.result); };
        request.onerror = function () { reject(request.error || new Error("indexedDB open failed")); };
    });
}

function gbcQueueTx(mode, run) {
    return gbcQueueOpen().then(function (db) {
        return new Promise(function (resolve, reject) {
            var tx = db.transaction(GBC_QUEUE_STORE, mode);
            var store = tx.objectStore(GBC_QUEUE_STORE);
            var request = run(store);
            tx.oncomplete = function () {
                db.close();
                resolve(request ? request.result : undefined);
            };
            tx.onerror = function () {
                db.close();
                reject(tx.error || new Error("indexedDB transaction failed"));
            };
        });
    });
}

// Persisting is best-effort: a browser with storage disabled must still upload.
async function gbcQueueRemember(record) {
    try {
        await gbcQueueTx("readwrite", function (store) { return store.put(record); });
        return true;
    } catch (err) {
        if (window.gbLogError) gbLogError(err, "gbcQueueRemember");
        else console.error("gbcQueueRemember", err);
        return false;
    }
}

async function gbcQueueForget(id) {
    try {
        await gbcQueueTx("readwrite", function (store) { return store.delete(id); });
    } catch (err) {
        if (window.gbLogError) gbLogError(err, "gbcQueueForget");
        else console.error("gbcQueueForget", err);
    }
}

async function gbcQueuePending() {
    try {
        var rows = await gbcQueueTx("readonly", function (store) { return store.getAll(); });
        return Array.isArray(rows) ? rows : [];
    } catch (err) {
        if (window.gbLogError) gbLogError(err, "gbcQueuePending");
        else console.error("gbcQueuePending", err);
        return [];
    }
}

async function gbcQueueRetry(rows) {
    var done = [];
    for (var i = 0; i < rows.length; i++) {
        var row = rows[i];
        try {
            await apiRequest("/write", {
                method: "POST",
                body: JSON.stringify({
                    bucket: row.bucket,
                    path: row.path,
                    content: row.content,
                    scope: row.scope,
                }),
            });
            await gbcQueueForget(row.id);
            done.push(row.id);
        } catch (err) {
            if (window.gbLogError) gbLogError(err, "gbcQueueRetry");
            else console.error("gbcQueueRetry", err);
        }
    }
    return done;
}

function gbcQueuePanel(rows) {
    var existing = document.getElementById("gbc-upload-queue-panel");
    if (existing) existing.remove();
    if (!rows.length) return;
    var t = function (key, fallback) {
        return window.gbT ? window.gbT(key, fallback) : fallback;
    };
    var panel = document.createElement("div");
    panel.id = "gbc-upload-queue-panel";
    panel.setAttribute("role", "status");
    panel.setAttribute("aria-live", "polite");
    panel.style.cssText = "position:fixed;bottom:calc(16px + env(safe-area-inset-bottom,0px));left:16px;right:16px;" +
        "max-width:420px;z-index:var(--z-toast);background:var(--card,#1e293b);color:var(--card-foreground,#f8fafc);" +
        "border:1px solid var(--border,#334155);border-radius:12px;padding:12px 14px;display:flex;" +
        "flex-direction:column;gap:8px;box-shadow:0 8px 24px rgba(0,0,0,.35);font-size:13px;";
    var body = document.createElement("span");
    body.textContent = t("drive-upload-queue-body", "{count} upload(s) did not finish. Retry?")
        .replace("{count}", String(rows.length));
    var actions = document.createElement("div");
    actions.style.cssText = "display:flex;gap:8px;justify-content:flex-end;";
    function button(key, fallback, primary, onClick) {
        var btn = document.createElement("button");
        btn.type = "button";
        btn.textContent = t(key, fallback);
        btn.style.cssText = "font:inherit;padding:6px 12px;border-radius:8px;cursor:pointer;" +
            (primary
                ? "background:var(--primary,#3b82f6);color:var(--primary-foreground,#fff);border:0;"
                : "background:transparent;color:inherit;border:1px solid var(--border,#334155);");
        btn.addEventListener("click", onClick);
        return btn;
    }
    actions.appendChild(button("drive-upload-queue-discard", "Discard", false, async function () {
        for (var i = 0; i < rows.length; i++) await gbcQueueForget(rows[i].id);
        panel.remove();
    }));
    actions.appendChild(button("drive-upload-queue-retry", "Retry", true, async function () {
        panel.remove();
        var recovered = await gbcQueueRetry(rows);
        if (recovered.length && typeof showNotification === "function") {
            showNotification(t("drive-upload-queue-recovered", "Uploaded {count} recovered file(s)")
                .replace("{count}", String(recovered.length)), "success");
            if (typeof loadFiles === "function") loadFiles(currentPath, currentBucket);
        }
        var left = rows.filter(function (row) { return recovered.indexOf(row.id) === -1; });
        gbcQueuePanel(left);
    }));
    panel.appendChild(body);
    panel.appendChild(actions);
    document.body.appendChild(panel);
}

// Called once after Drive boots: offer back whatever a killed tab left behind.
async function gbcQueueOfferRecovery() {
    var rows = await gbcQueuePending();
    if (rows.length) gbcQueuePanel(rows);
}
