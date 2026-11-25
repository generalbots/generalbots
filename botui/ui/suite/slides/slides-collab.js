
/* Slides collab comments — open threaded comments for the active deck. */
function openSlidesComments() {
    if (!window.GBCollabComments) return;
    var title = (document.getElementById("presTitle") && document.getElementById("presTitle").value) || "Presentation";
    window.GBCollabComments.open({
        resourceType: "slides",
        resourceId: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "",
        title: "Comments on " + title,
        includeChildren: true,
    });
}
/* Slides activity log — audit timeline for the active deck. */
function openSlidesActivity() {
    if (!window.GBCollabActivity) return;
    var title = (document.getElementById("presTitle") && document.getElementById("presTitle").value) || "Presentation";
    window.GBCollabActivity.open({
        resourceType: "slides",
        resourceId: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "",
        title: "Activity on " + title,
    });
}
/* Slides sharing — real access-control dialog (replaces the old stub). */
function openSlidesShare() {
    if (!window.GBCollabShare) return;
    var title = (document.getElementById("presTitle") && document.getElementById("presTitle").value) || "Presentation";
    window.GBCollabShare.open({
        resourceType: "slides",
        resourceId: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "",
        title: "Share " + title,
    });
}
/* Slides version history — snapshot/restore the active deck. */
function openSlidesVersions() {
    if (!window.GBCollabVersions) return;
    var title = (document.getElementById("presTitle") && document.getElementById("presTitle").value) || "Presentation";
    window.GBCollabVersions.open({
        resourceType: "slides",
        resourceId: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "",
        title: "Version history — " + title,
        onRestore: function (content) {
            var host = document.getElementById("slides-content");
            if (!host) return;
            host.innerHTML = content;
            var els = host.querySelectorAll(".sl-element");
            if (window.SlidesCanvas && window.SlidesCanvas.bindElement) {
                for (var i = 0; i < els.length; i++) window.SlidesCanvas.bindElement(els[i]);
            }
            if (window.recordSlidesEdit) window.recordSlidesEdit();
        }
    });
}
var _slidesSnapshotTimer = null;
function recordSlidesSnapshot() {
    if (!window.GBCollabVersions) return;
    clearTimeout(_slidesSnapshotTimer);
    _slidesSnapshotTimer = setTimeout(function () {
        var host = document.getElementById("slides-content");
        if (!host) return;
        var sid = (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "";
        window.GBCollabVersions.snapshot({
            resourceType: "slides",
            resourceId: sid,
            content: host.innerHTML
        }).then(function () {
            if (window.GBOfflineDraft) window.GBOfflineDraft.clear("slides:" + sid);
        }).catch(function () {
            if (window.GBOfflineDraft) window.GBOfflineDraft.save("slides:" + sid, host.innerHTML);
        });
    }, 1500);
}
var _slidesEditTimer = null;
function slidesDraftKey() {
    return "slides:" + ((window.getSlidesPresentationId && window.getSlidesPresentationId()) || "");
}
function applySlidesContent(content) {
    var host = document.getElementById("slides-content");
    if (!host) return;
    host.innerHTML = content;
    var els = host.querySelectorAll(".sl-element");
    if (window.SlidesCanvas && window.SlidesCanvas.bindElement) {
        for (var i = 0; i < els.length; i++) window.SlidesCanvas.bindElement(els[i]);
    }
    if (window.recordSlidesEdit) window.recordSlidesEdit();
}
function restoreSlidesDraft() {
    if (!window.GBOfflineDraft) return;
    var key = slidesDraftKey();
    var draft = window.GBOfflineDraft.load(key);
    if (!draft) return;
    if (window.GBOfflineDraft.isOnline() && window.GBCollabVersions) {
        window.GBCollabVersions.snapshot({ resourceType: "slides", resourceId: key.slice(7), content: draft.c })
            .then(function () {
                window.GBOfflineDraft.clear(key);
                window.GBOfflineDraft.hideBanner();
            })
            .catch(function () {
                window.gbLog && window.gbLog.warn ? window.gbLog.warn("restoreSlidesDraft: snapshot sync failed") : console.warn("restoreSlidesDraft: snapshot sync failed");
            });
    } else {
        applySlidesContent(draft.c);
    }
}
function checkSlidesDraftOnLoad() {
    if (!window.GBOfflineDraft || !window.GBOfflineDraft.has(slidesDraftKey())) return;
    window.GBOfflineDraft.showBanner(
        "You have unsaved changes from a previous session",
        { sticky: true, actionLabel: "Restore", onAction: restoreSlidesDraft }
    );
    window.GBOfflineDraft.onReconnect(restoreSlidesDraft);
}
function recordSlidesEdit() {
    if (!window.GBCollabActivity) return;
    clearTimeout(_slidesEditTimer);
    _slidesEditTimer = setTimeout(function () {
        window.GBCollabActivity.record({
            resourceType: "slides",
            resourceId: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "",
            action: "edit",
            payload: {}
        }).catch(function () {
            window.gbLog && window.gbLog.warn ? window.gbLog.warn("recordSlidesEdit: activity record failed") : console.warn("recordSlidesEdit: activity record failed");
        });
    }, 1500);
}
function bindSlidesBadge() {
    if (!window.GBCollabComments || !window.GBCollabComments.bindBadge) return;
    var btn = document.getElementById("commentsBtn");
    if (!btn) return;
    window.GBCollabComments.bindBadge(btn, {
        resourceType: "slides",
        resourceId: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "",
        includeChildren: true,
    });
}
/* Slides conflict watch — poll the op-log for concurrent/LLM edits. */
function startSlidesConflictWatch() {
    if (!window.GBCollabConflict) return;
    window.GBCollabConflict.start({
        resourceType: "slides",
        resourceId: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || ""
    });
}
if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", function () { bindSlidesBadge(); startSlidesConflictWatch(); });
else { bindSlidesBadge(); startSlidesConflictWatch(); }
