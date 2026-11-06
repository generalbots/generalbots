/* Docs collab comments — open threaded comments for the active document. */
function openDocComments() {
    if (!window.GBCollabComments) return;
    var article = document.querySelector("article[contenteditable]");
    var docId = (article && article.dataset && article.dataset.docId) || "current";
    var title = (document.getElementById("docTitle") && document.getElementById("docTitle").value) || "Document";
    window.GBCollabComments.open({
        resourceType: "docs",
        resourceId: String(docId),
        title: "Comments on " + title,
        includeChildren: true,
    });
}
function bindDocsBadge() {
    if (!window.GBCollabComments || !window.GBCollabComments.bindBadge) return;
    var btn = document.getElementById("commentsBtn");
    var article = document.querySelector("article[contenteditable]");
    if (!btn || !article) return;
    var docId = (article.dataset && article.dataset.docId) || "current";
    window.GBCollabComments.bindBadge(btn, {
        resourceType: "docs",
        resourceId: String(docId),
        includeChildren: true,
    });
}
/* Docs version history — snapshot/restore the active document. */
function openDocVersions() {
    if (!window.GBCollabVersions) return;
    var article = document.querySelector("article[contenteditable]");
    var docId = (article && article.dataset && article.dataset.docId) || "current";
    var title = (document.getElementById("docTitle") && document.getElementById("docTitle").value) || "Document";
    window.GBCollabVersions.open({
        resourceType: "docs",
        resourceId: String(docId),
        title: "Version history — " + title,
        onRestore: function (content) {
            var a = document.querySelector("article[contenteditable]");
            if (a) { a.innerHTML = content; scheduleSave(a.dataset.docId, content); }
        }
    });
}
/* Docs sharing — real access-control dialog (replaces the old stub). */
function openDocShare() {
    if (!window.GBCollabShare) return;
    var article = document.querySelector("article[contenteditable]");
    var docId = (article && article.dataset && article.dataset.docId) || "current";
    var title = (document.getElementById("docTitle") && document.getElementById("docTitle").value) || "Document";
    window.GBCollabShare.open({
        resourceType: "docs",
        resourceId: String(docId),
        title: "Share " + title,
    });
}
/* Docs activity log — audit timeline for the active document. */
function openDocActivity() {
    if (!window.GBCollabActivity) return;
    var article = document.querySelector("article[contenteditable]");
    var docId = (article && article.dataset && article.dataset.docId) || "current";
    var title = (document.getElementById("docTitle") && document.getElementById("docTitle").value) || "Document";
    window.GBCollabActivity.open({
        resourceType: "docs",
        resourceId: String(docId),
        title: "Activity on " + title,
    });
}
/* Docs conflict watch — poll the op-log for concurrent/LLM edits. */
function startDocConflictWatch() {
    if (!window.GBCollabConflict) return;
    var article = document.querySelector("article[contenteditable]");
    var docId = (article && article.dataset && article.dataset.docId) || "current";
    window.GBCollabConflict.start({
        resourceType: "docs",
        resourceId: String(docId)
    });
}
document.addEventListener("htmx:afterSwap", function (e) {
    if (e.target.id === "docs-content") {
        setTimeout(bindDocsBadge, 0);
        setTimeout(function () { if (typeof checkDocDraftOnLoad === "function") checkDocDraftOnLoad(); }, 0);
        setTimeout(startDocConflictWatch, 0);
    }
    if (e.target.id === "docs-panel-body") {
        // Toolbar buttons (TOC, track changes) render into the sidebar panel
        // host; reveal the Painéis tab so the result is visible.
        var panelsTab = document.querySelector('[data-sidebar-tab="panels"]');
        if (panelsTab) panelsTab.click();
    }
});
if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", function () { setTimeout(bindDocsBadge, 50); setTimeout(startDocConflictWatch, 50); });
else setTimeout(bindDocsBadge, 50);

/* #1711 — the print modal is injected HTML; a plain <a href> cannot carry the
   bearer token the export API requires (401), and a Drive-opened document has
   no stored doc to export (404). Fetch with the suite token and save a blob;
   fall back to the live editor content: print for PDF, download HTML else. */
function gbDocsSaveBlob(blob, name) {
    var link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = name;
    document.body.appendChild(link);
    link.click();
    link.remove();
    setTimeout(function () { URL.revokeObjectURL(link.href); }, 4000);
}
function GBDocsExport(format, id) {
    var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
        window.localStorage.getItem("gb-access-token") ||
        window.sessionStorage.getItem("gb-access-token") || "";
    var headers = token ? { Authorization: "Bearer " + token } : {};
    fetch("/api/docs/export/" + encodeURIComponent(format) + "?id=" + encodeURIComponent(id), { headers: headers })
        .then(function (response) {
            if (!response.ok) throw new Error("HTTP " + response.status);
            return response.blob();
        })
        .then(function (blob) {
            var title = (document.getElementById("docTitle") && document.getElementById("docTitle").value) ||
                "document";
            var slug = title.replace(/[^\w\-]+/g, "_").replace(/_+/g, "_").replace(/^_|_$/g, "") || "document";
            gbDocsSaveBlob(blob, slug + "." + format);
        })
        .catch(function () {
            var article = document.querySelector("[id^=window-body-docs] #doc-content") ||
                document.querySelector("#doc-content");
            var html = article ? article.innerHTML : "";
            if (!html) {
                if (window.GBToasts) GBToasts.show("Export", "Nothing to export yet.", "error");
                return;
            }
            if (format === "pdf") {
                var printWindow = window.open("", "_blank");
                if (!printWindow) return;
                printWindow.document.write(
                    "<!doctype html><html><head><meta charset='utf-8'><title>Print</title></head><body>" +
                    html + "</body></html>"
                );
                printWindow.document.close();
                printWindow.focus();
                printWindow.print();
                return;
            }
            // The stored-document export 404s for a file opened from Drive:
            // there is no saved copy to convert. Save the live editor content
            // as HTML and say so, instead of naming an HTML file ".docx".
            var fallbackTitle = (document.getElementById("docTitle") && document.getElementById("docTitle").value) ||
                "document";
            var fallbackSlug = fallbackTitle.replace(/[^\w\-]+/g, "_").replace(/_+/g, "_").replace(/^_|_$/g, "") || "document";
            if (window.GBToasts) {
                GBToasts.show("Export",
                    "This document has no stored copy yet, so the editor content was saved as HTML.",
                    "info");
            }
            gbDocsSaveBlob(new Blob([html], { type: "text/html" }), fallbackSlug + ".html");
        });
}
window.GBDocsExport = GBDocsExport;

/* The template endpoints return a Document JSON; persist it as a real doc and
   load it into the editor instead of navigating to the JSON response. */
function GBDocsApplyTemplateByName(name, id) {
    var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
        window.localStorage.getItem("gb-access-token") ||
        window.sessionStorage.getItem("gb-access-token") || "";
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    fetch("/api/docs/template/" + encodeURIComponent(name), { headers: headers })
        .then(function (response) {
            if (!response.ok) throw new Error("HTTP " + response.status);
            return response.json();
        })
        .then(function (doc) {
            return fetch("/api/docs/save", {
                method: "POST",
                headers: headers,
                body: JSON.stringify({ id: doc.id, title: doc.title, content: doc.content }),
            }).then(function (response) {
                if (!response.ok) throw new Error("HTTP " + response.status);
                return doc;
            });
        })
        .then(function (doc) {
            var scope = document.querySelector("[id^=window-body-docs]") || document;
            var article = scope.querySelector("#doc-content");
            var title = scope.querySelector("#docTitle");
            if (title) title.value = doc.title || "Untitled Document";
            if (article) {
                article.innerHTML = doc.content || "";
                if (article.dataset) article.dataset.docId = doc.id;
            }
            var modal = scope.querySelector("#template-modal") ||
                document.getElementById("template-modal");
            if (modal) modal.remove();
        })
        .catch(function (error) {
            if (window.GBToasts) GBToasts.show("Template", "Failed: " + error.message, "error");
        });
}

function GBDocsApplyTemplate(event, id) {
    event.preventDefault();
    var form = event.target;
    var select = form.querySelector('[name="template"]');
    var name = (select && select.value) || (form.dataset && form.dataset.template) || "blank";
    GBDocsApplyTemplateByName(name, id);
    return false;
}
window.GBDocsApplyTemplateByName = GBDocsApplyTemplateByName;
window.GBDocsApplyTemplate = GBDocsApplyTemplate;
/* Find & replace in the live document. The old form posted to the GET-only
   /api/docs/load (405); replacing text nodes keeps markup and attributes
   intact, and the autosave persists the result. */
function GBDocsFindReplace(event) {
    event.preventDefault();
    var form = event.target;
    var find = form.querySelector('[name="find"]');
    var replace = form.querySelector('[name="replace"]');
    var needle = find ? find.value : "";
    if (!needle) return false;
    var replacement = replace ? replace.value : "";
    var scope = form.closest ? (form.closest("[id^=window-body-docs]") || document) : document;
    var article = scope.querySelector("#doc-content");
    if (!article) return false;
    var count = 0;
    var walker = document.createTreeWalker(article, NodeFilter.SHOW_TEXT);
    var nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    nodes.forEach(function (node) {
        if (node.nodeValue && node.nodeValue.indexOf(needle) !== -1) {
            count += node.nodeValue.split(needle).length - 1;
            node.nodeValue = node.nodeValue.split(needle).join(replacement);
        }
    });
    var output = form.querySelector("#find-replace-results");
    if (output) output.textContent = count + " replacement(s) applied.";
    if (typeof scheduleSave === "function" && article.dataset && article.dataset.docId) {
        scheduleSave(article.dataset.docId, article.innerHTML);
    }
    return false;
}
window.GBDocsFindReplace = GBDocsFindReplace;
/* The print form is injected into #docs-modal-container, not a .dc-modal. */
function GBDocsCloseModal(element) {
    var modal = element.closest ? element.closest(".dc-modal") : null;
    if (modal) {
        modal.remove();
        return;
    }
    var container = document.getElementById("docs-modal-container");
    if (container) container.innerHTML = "";
}
window.GBDocsCloseModal = GBDocsCloseModal;

/* Compare returns a DocumentComparison JSON; render a readable diff instead
   of swapping the object into the dialog. */
function gbDocsEscape(value) {
    return String(value == null ? "" : value)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}
function GBDocsCompare(event) {
    event.preventDefault();
    var form = event.target;
    var original = form.dataset ? form.dataset.originalDocId : "";
    var input = form.querySelector('[name="modified_doc_id"]');
    var modified = input ? input.value.trim() : "";
    var output = form.querySelector("#compare-result");
    if (!output) return false;
    if (!original || !modified) {
        output.textContent = "Both document ids are required.";
        return false;
    }
    var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
        window.localStorage.getItem("gb-access-token") ||
        window.sessionStorage.getItem("gb-access-token") || "";
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    output.textContent = "Comparing…";
    fetch("/api/docs/compare", {
        method: "POST",
        headers: headers,
        body: JSON.stringify({ original_doc_id: original, modified_doc_id: modified }),
    })
        .then(function (response) {
            if (!response.ok) throw new Error("HTTP " + response.status);
            return response.json();
        })
        .then(function (data) {
            var comparison = data.comparison || {};
            var summary = comparison.summary || {};
            var differences = comparison.differences || [];
            var html = '<div style="padding:6px 0;color:#f8fafc;font-weight:600;">' +
                (summary.total_changes || 0) + " change(s) · +" +
                (summary.insertions || 0) + " / −" + (summary.deletions || 0) +
                " / ~" + (summary.modifications || 0) + "</div>";
            if (!differences.length) {
                html += '<div style="padding:8px;color:#94a3b8;">No differences found.</div>';
            } else {
                html += '<div style="max-height:40vh;overflow:auto;">';
                differences.slice(0, 200).forEach(function (diff) {
                    var before = diff.original_text || "";
                    var after = diff.modified_text || "";
                    html += '<div style="padding:6px;border-bottom:1px solid #334155;">' +
                        '<div style="color:#94a3b8;font-size:11px;">' +
                        gbDocsEscape(diff.diff_type) + " @ " + (diff.position || 0) + "</div>" +
                        (before ? '<div style="color:#fca5a5;text-decoration:line-through;">' + gbDocsEscape(before) + "</div>" : "") +
                        (after ? '<div style="color:#86efac;">' + gbDocsEscape(after) + "</div>" : "") +
                        "</div>";
                });
                html += "</div>";
            }
            output.innerHTML = html;
        })
        .catch(function (error) {
            output.textContent = "Compare failed: " + error.message;
        });
    return false;
}
window.GBDocsCompare = GBDocsCompare;

/* Re-fetch a sidebar panel after a mutation whose API returns JSON. */
function GBDocsRefreshTrackChanges() {
    var view = document.querySelector("#doc-view[data-id]");
    var article = document.querySelector("article[data-doc-id]");
    var id = (view && view.getAttribute("data-id")) ||
        (article && article.dataset ? article.dataset.docId : "") || "";
    if (!id) return;
    var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
        window.localStorage.getItem("gb-access-token") ||
        window.sessionStorage.getItem("gb-access-token") || "";
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    fetch("/suite/docs/fragments/track-changes", {
        method: "POST",
        headers: headers,
        body: JSON.stringify({ id: id }),
    })
        .then(function (response) { return response.text(); })
        .then(function (html) {
            var panel = document.getElementById("docs-panel-body");
            if (panel) panel.innerHTML = html;
        })
        .catch(function () {});
}
window.GBDocsRefreshTrackChanges = GBDocsRefreshTrackChanges;

/* TOC/outline entries carry the server's character offset; scroll to the
   closest heading in the live document instead of a non-existent #pos-N id. */
document.addEventListener("click", function (event) {
    var link = event.target.closest ? event.target.closest(".dc-toc-entry a, .dc-outline-item a") : null;
    if (!link) return;
    var href = link.getAttribute("href") || "";
    if (href.indexOf("#pos-") !== 0) return;
    event.preventDefault();
    var target = parseInt(href.slice(5), 10) || 0;
    var article = document.querySelector("#doc-content");
    if (!article) return;
    var headings = article.querySelectorAll("h1,h2,h3,h4,h5,h6");
    var regex = /<h[1-6][^>]*>/gi;
    var html = article.innerHTML;
    var indexes = [];
    var match;
    while ((match = regex.exec(html))) indexes.push(match.index);
    var best = 0;
    var bestDistance = Infinity;
    indexes.forEach(function (index, position) {
        var distance = Math.abs(index - target);
        if (distance < bestDistance) {
            bestDistance = distance;
            best = position;
        }
    });
    if (headings[best]) headings[best].scrollIntoView({ behavior: "smooth", block: "start" });
});

