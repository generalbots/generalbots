/* Drive Module v2.0 — 08 Modals: preview, editor, share, new-bot */
"use strict";

/**
 * Sanitize HTML to prevent XSS — keeps only safe tags and attributes.
 */
function sanitizeHtml(html) {
  // Allowed tags (lowercase)
  var SAFE_TAGS = ["p","h1","h2","h3","h4","h5","h6","ul","ol","li","a",
    "strong","em","b","i","code","pre","blockquote","table","tr","td","th",
    "thead","tbody","img","br","hr","div","span","sup","sub","dl","dt","dd"];
  // Allowed attributes per tag (all lowercase)
  var SAFE_ATTRS = {
    a: ["href","title","target","rel"],
    img: ["src","alt","width","height"],
    td: ["colspan","rowspan"],
    th: ["colspan","rowspan"],
  };

  // 1. Remove <script> tags and their content
  html = html.replace(/<script[\s\S]*?<\/script\s*>/gi, "");
  // 2. Remove <style> tags and their content
  html = html.replace(/<style[\s\S]*?<\/style\s*>/gi, "");
  // 3. Remove dangerous tags (keep content but strip the tags themselves)
  var dangerousTags = ["iframe","object","embed","form","input","textarea","link","meta","base"];
  dangerousTags.forEach(function (tag) {
    html = html.replace(new RegExp("<\\/?" + tag + "\\b[^>]*>", "gi"), "");
  });

  // 4. Remove all on* event handler attributes
  html = html.replace(/\s+on\w+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)/gi, "");
  // 5. Remove javascript: URLs in any attribute value
  html = html.replace(/(href|src|action)\s*=\s*(?:"javascript:[^"]*"|'javascript:[^']*')/gi, "$1=\"\"");
  html = html.replace(/(href|src|action)\s*=\s*javascript:[^\s>]*/gi, "$1=\"\"");

  // 6. Strip tags not in the safe list (keep their content)
  html = html.replace(/<\/?([a-zA-Z][a-zA-Z0-9]*)\b[^>]*>/g, function (match, tagName) {
    var lower = tagName.toLowerCase();
    var isClosing = match.charAt(1) === "/";
    if (SAFE_TAGS.indexOf(lower) === -1) return "";
    if (isClosing) return "</" + lower + ">";
    // Rebuild tag with only allowed attributes
    var allowed = SAFE_ATTRS[lower] || [];
    var attrs = "";
    var attrRe = /\s+([a-zA-Z_][\w-]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|(\S+))/g;
    var am;
    while ((am = attrRe.exec(match)) !== null) {
      var attrName = am[1].toLowerCase();
      var attrVal = am[2] !== undefined ? am[2] : (am[3] !== undefined ? am[3] : am[4]);
      if (allowed.indexOf(attrName) === -1) continue;
      // Block javascript: in href/src
      if ((attrName === "href" || attrName === "src") && /^\s*javascript\s*:/i.test(attrVal)) continue;
      attrs += " " + attrName + '="' + attrVal.replace(/"/g, "&quot;") + '"';
    }
    // Self-closing tags
    if (["br","hr","img"].indexOf(lower) !== -1) return "<" + lower + attrs + " />";
    return "<" + lower + attrs + ">";
  });

  return html;
}

function showPreviewModal(fileName, ext, blob) {
    var modal = document.getElementById("preview-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "preview-modal";
    modal.style.cssText = "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(15, 23, 42, 0.85); display: flex; align-items: center; justify-content: center; z-index: 9999; backdrop-filter: blur(8px);";
    const container = document.createElement("div");
    container.style.cssText = "background: #1e293b; border: 1px solid #334155; border-radius: 12px; width: 80%; max-width: 900px; max-height: 90vh; display: flex; flex-direction: column; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5); overflow: hidden;";
    const header = document.createElement("div");
    header.style.cssText = "display: flex; align-items: center; justify-content: space-between; padding: 16px 24px; border-bottom: 1px solid #334155;";
    header.innerHTML = '<h3 style="margin:0; color:#f8fafc; font-size:1.125rem;">Preview: ' + escapeHtml(fileName) + '</h3>';
    const closeBtn = document.createElement("button");
    closeBtn.textContent = "\u00D7";
    closeBtn.style.cssText = "background: none; border: none; color: #94a3b8; font-size: 28px; cursor: pointer; line-height: 1; padding: 0; margin: 0;";
    var objectUrl;
    closeBtn.onclick = function() { modal.remove(); if (objectUrl) URL.revokeObjectURL(objectUrl); };
    header.appendChild(closeBtn);
    const body = document.createElement("div");
    body.style.cssText = "padding: 24px; overflow-y: auto; flex-grow: 1; display: flex; align-items: center; justify-content: center; background: #0f172a;";
    objectUrl = URL.createObjectURL(blob);
    var previewEl;
    if (["png", "jpg", "jpeg", "gif", "webp", "bmp", "avif", "svg"].indexOf(ext) !== -1) {
        previewEl = document.createElement("img");
        previewEl.src = objectUrl;
        previewEl.style.cssText = "max-width:100%; max-height:70vh; object-fit:contain; border-radius:4px;";
    } else if (ext === "pdf") {
        previewEl = document.createElement("iframe");
        previewEl.src = objectUrl;
        previewEl.style.cssText = "width:100%; height:70vh; border:none; border-radius:4px;";
    } else if (["mp4", "webm", "mov", "m4v", "mkv", "avi", "ogv"].indexOf(ext) !== -1) {
        previewEl = document.createElement("video");
        previewEl.src = objectUrl;
        previewEl.controls = true;
        previewEl.autoplay = true;
        previewEl.style.cssText = "max-width:100%; max-height:70vh; border-radius:4px;";
    } else if (["mp3", "wav", "ogg", "oga", "m4a", "flac"].indexOf(ext) !== -1) {
        previewEl = document.createElement("audio");
        previewEl.src = objectUrl;
        previewEl.controls = true;
        previewEl.autoplay = true;
        previewEl.style.cssText = "width:100%; max-width:500px;";
    } else if (ext === "md" && typeof marked !== "undefined") {
        previewEl = document.createElement("div");
        previewEl.style.cssText = "color:#f8fafc; font-size:14px; width:100%; max-height:70vh; overflow-y:auto; padding:16px; line-height:1.6;";
        blob.text().then(function(t) { previewEl.innerHTML = sanitizeHtml(marked.parse(t.substring(0, 50000))); }).catch(function() { previewEl.textContent = "Preview not available."; });
    } else if (ext === "csv") {
        previewEl = document.createElement("div");
        previewEl.style.cssText = "color:#f8fafc; width:100%; max-height:70vh; overflow:auto;";
        blob.text().then(function(t) {
            var rows = t.substring(0, 30000).split("\n").filter(function(r) { return r.trim(); });
            var html = '<table style="border-collapse:collapse;width:100%;font-size:13px;">';
            rows.forEach(function(row, i) {
                var cells = row.split(",");
                var tag = i === 0 ? "th" : "td";
                html += "<tr>" + cells.map(function(c) { return "<" + tag + ' style="border:1px solid #334155;padding:6px 10px;text-align:left;">' + escapeHtml(c.trim()) + "</" + tag + ">"; }).join("") + "</tr>";
            });
            html += "</table>";
            previewEl.innerHTML = html;
        }).catch(function() { previewEl.textContent = "Preview not available."; });
    } else if (ext === "json") {
        previewEl = document.createElement("pre");
        previewEl.style.cssText = "color:#f8fafc; font-family:monospace; font-size:13px; white-space:pre-wrap; width:100%; max-height:70vh; overflow-y:auto; margin:0;";
        blob.text().then(function(t) { try { previewEl.textContent = JSON.stringify(JSON.parse(t), null, 2); } catch (e) { previewEl.textContent = t.substring(0, 10000); } }).catch(function() { previewEl.textContent = "Preview not available."; });
    } else {
        previewEl = document.createElement("pre");
        previewEl.style.cssText = "color:#f8fafc; font-family:monospace; font-size:14px; white-space:pre-wrap; width:100%; max-height:70vh; overflow-y:auto; margin:0;";
        blob.text().then(function(t) { previewEl.textContent = t.substring(0, 10000); }).catch(function() { previewEl.textContent = "Preview not available for this file type."; });
    }
    body.appendChild(previewEl);
    container.appendChild(header);
    container.appendChild(body);
    modal.appendChild(container);
    document.body.appendChild(modal);
}

// ── Public share link (real, token-based, revocable) ────────────────────
// Creates a link via POST /api/files/share-link (or reuses the existing one
// from GET /api/files/share-links) and shows the real URL with Copy + Revoke.

function shareFile(path) {
    var fileName = path.split("/").pop() || "file";
    var modal = document.getElementById("share-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "share-modal";
    modal.style.cssText = "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(15, 23, 42, 0.85); display: flex; align-items: center; justify-content: center; z-index: 9999; backdrop-filter: blur(8px);";
    const container = document.createElement("div");
    container.style.cssText = "background: #1e293b; border: 1px solid #334155; border-radius: 12px; width: 90%; max-width: 560px; padding: 24px; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);";
    container.innerHTML =
        '<div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:16px;">' +
        '<h3 style="margin:0; color:#f8fafc; font-size:1.25rem;">Share "' + escapeHtml(fileName) + '"</h3>' +
        '<button onclick="document.getElementById(\'share-modal\').remove()" style="background:none; border:none; color:#94a3b8; font-size:24px; cursor:pointer;">\u00D7</button></div>' +
        // #1639 — the expiry is chosen here; the old modal never sent one, so
        // the server default ("never") was the only behaviour.
        '<div style="display:flex; gap:10px; align-items:center; margin-bottom:12px;">' +
        '<label for="share-expiry" style="color:#94a3b8; font-size:13px;">Link expires</label>' +
        '<select id="share-expiry" style="background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:8px 10px; font-size:13px;">' +
        '<option value="7" selected>In 7 days</option>' +
        '<option value="1">In 1 day</option>' +
        '<option value="30">In 30 days</option>' +
        '<option value="never">Never (not recommended)</option>' +
        '</select>' +
        '<button id="share-create-btn" onclick="DriveModule.createShareLinkNow(\'' + escapeJs(path) + '\')" style="margin-left:auto; background:#3b82f6; border:none; border-radius:6px; color:#fff; padding:8px 16px; cursor:pointer; font-weight:500; font-size:13px;">Create link</button>' +
        '</div>' +
        '<p id="share-status" style="color:#94a3b8; font-size:13px; margin:0 0 12px;"></p>' +
        '<div id="share-link-row" style="display:none; gap:8px; margin-bottom:16px;">' +
        '<input type="text" id="share-link-input" readonly value="" style="flex-grow:1; background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:10px 14px; font-size:13px;" />' +
        '<button onclick="DriveModule.copyShareLink()" style="background:#3b82f6; border:none; border-radius:6px; color:#fff; padding:0 16px; cursor:pointer; font-weight:500; font-size:13px;">Copy</button>' +
        '<button onclick="DriveModule.revokeShareLinkFromModal(\'' + escapeJs(path) + '\')" style="background:#dc2626; border:none; border-radius:6px; color:#fff; padding:0 16px; cursor:pointer; font-weight:500; font-size:13px;">Revoke</button>' +
        '</div>' +
        '<div style="border-top:1px solid #334155; padding-top:14px;">' +
        '<h4 style="margin:0 0 8px; color:#f8fafc; font-size:14px;">Share with a person</h4>' +
        '<div style="display:flex; gap:8px;">' +
        '<input id="share-recipient-input" type="search" placeholder="Email or username" autocomplete="off" style="flex-grow:1; background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:9px 12px; font-size:13px;" />' +
        '<select id="share-role" style="background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:9px 10px; font-size:13px;">' +
        '<option value="read">Can view</option>' +
        '<option value="write">Can edit</option>' +
        '</select>' +
        '</div>' +
        '<div id="share-recipient-results" style="margin-top:8px;"></div>' +
        '<p id="share-recipient-status" style="color:#94a3b8; font-size:12px; margin:8px 0 0;">Type at least two characters to find an account.</p>' +
        '</div>' +
        '<div style="border-top:1px solid #334155; padding-top:14px; margin-top:14px;">' +
        '<h4 style="margin:0 0 8px; color:#f8fafc; font-size:14px;">Send the link by email</h4>' +
        '<div style="display:flex; gap:8px; margin-bottom:8px;">' +
        '<select id="share-email-sender" style="background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:9px 10px; font-size:13px; max-width:220px;">' +
        '<option value="">Loading senders…</option>' +
        '</select>' +
        '<input id="share-email-to" type="text" placeholder="name@example.com, other@example.com" autocomplete="off" style="flex-grow:1; background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:9px 12px; font-size:13px;" />' +
        '</div>' +
        '<textarea id="share-email-message" rows="2" placeholder="Optional message" style="width:100%; box-sizing:border-box; background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:9px 12px; font-size:13px; resize:vertical;"></textarea>' +
        '<div style="display:flex; align-items:center; gap:10px; margin-top:8px;">' +
        '<button id="share-email-send" onclick="DriveModule.sendShareEmail(\'' + escapeJs(path) + '\')" style="background:#3b82f6; border:none; border-radius:6px; color:#fff; padding:9px 16px; cursor:pointer; font-weight:500; font-size:13px;">Send link</button>' +
        '<span id="share-email-status" style="color:#94a3b8; font-size:12px;"></span>' +
        '</div>' +
        '</div>' +
        '<div style="display:flex; justify-content:flex-end; margin-top:16px;">' +
        '<button onclick="document.getElementById(\'share-modal\').remove()" style="background:#334155; border:none; border-radius:6px; color:#f8fafc; padding:10px 20px; cursor:pointer; font-weight:500; font-size:13px;">Close</button>' +
        '</div>';
    modal.appendChild(container);
    document.body.appendChild(modal);

    bindShareRecipientSearch(path);
    loadExistingShareLink(path);
    loadShareEmailSenders();
}

function shareExpiryIso() {
    var select = document.getElementById("share-expiry");
    var value = select ? select.value : "7";
    if (value === "never") return null;
    var days = parseInt(value, 10);
    if (!isFinite(days) || days <= 0) return null;
    return new Date(Date.now() + days * 24 * 60 * 60 * 1000).toISOString();
}

async function loadExistingShareLink(path) {
    var status = document.getElementById("share-status");
    try {
        var existing = await apiRequest("/share-links?path=" + encodeURIComponent(path) + "&bucket=" + encodeURIComponent(getEffectiveBucket()) + "&scope=" + currentScope);
        var live = (existing || []).find(function (l) { return !l.revoked && !isExpired(l.expires_at); });
        if (!live) {
            if (status) status.textContent = "No live link yet. Choose an expiry and create one.";
            return;
        }
        showShareLink(live);
    } catch (e) {
        if (status) status.textContent = "Choose an expiry and create a link.";
    }
}

function isExpired(value) {
    if (!value) return false;
    var when = Date.parse(value);
    return isFinite(when) && when <= Date.now();
}

function showShareLink(result) {
    var status = document.getElementById("share-status");
    var row = document.getElementById("share-link-row");
    var input = document.getElementById("share-link-input");
    if (!row || !input) return;
    input.value = window.location.origin + result.url;
    row.style.display = "flex";
    row.dataset.token = result.token || "";
    if (status) {
        if (!result.expires_at) {
            status.textContent = "Anyone with this link can download the file until it is revoked (no expiry).";
        } else if (isExpired(result.expires_at)) {
            status.textContent = "This link expired on " + new Date(result.expires_at).toLocaleString() + ".";
        } else {
            status.textContent = "Anyone with this link can download the file until " +
                new Date(result.expires_at).toLocaleString() + ". Revoke it at any time.";
        }
    }
    var createBtn = document.getElementById("share-create-btn");
    if (createBtn) createBtn.style.display = "none";
}

async function createShareLinkNow(path) {
    var status = document.getElementById("share-status");
    if (status) status.textContent = "Creating secure link...";
    try {
        var payload = { bucket: getEffectiveBucket(), path: path, scope: currentScope };
        var expires = shareExpiryIso();
        if (expires) payload.expires_at = expires;
        var result = await apiRequest("/share-link", {
            method: "POST",
            body: JSON.stringify(payload),
        });
        showShareLink(result);
    } catch (err) {
        if (status) status.textContent = "Failed to create link: " + err.message;
    }
}

function bindShareRecipientSearch(path) {
    var input = document.getElementById("share-recipient-input");
    var results = document.getElementById("share-recipient-results");
    var status = document.getElementById("share-recipient-status");
    if (!input || !results) return;
    var timer;
    input.addEventListener("input", function () {
        clearTimeout(timer);
        var query = input.value.trim();
        results.innerHTML = "";
        if (query.length < 2) {
            if (status) status.textContent = "Type at least two characters to find an account.";
            return;
        }
        timer = setTimeout(async function () {
            try {
                var people = await apiRequest("/share/recipients?query=" + encodeURIComponent(query));
                if (!people || !people.length) {
                    if (status) status.textContent = "No account matches that email or username.";
                    return;
                }
                if (status) status.textContent = "Choose an account to share with:";
                results.innerHTML = people.map(function (person) {
                    return '<button type="button" class="btn-primary" data-recipient="' + escapeHtml(person.id) +
                        '" data-email="' + escapeHtml(person.email) +
                        '" style="display:block; width:100%; text-align:left; margin:4px 0; padding:8px 10px; background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; cursor:pointer; font-size:13px;">' +
                        escapeHtml(person.email) + ' <span style="color:#64748b;">' + escapeHtml(person.username || "") + '</span></button>';
                }).join("");
                Array.prototype.forEach.call(results.querySelectorAll("[data-recipient]"), function (btn) {
                    btn.addEventListener("click", function () {
                        shareWithRecipient(path, btn.getAttribute("data-recipient"), btn.getAttribute("data-email"));
                    });
                });
            } catch (err) {
                if (status) status.textContent = "Could not search accounts: " + err.message;
            }
        }, 300);
    });
}

async function shareWithRecipient(path, recipientId, email) {
    var role = document.getElementById("share-role");
    var status = document.getElementById("share-recipient-status");
    try {
        await apiRequest("/share", {
            method: "POST",
            body: JSON.stringify({
                bucket: getEffectiveBucket(),
                path: path,
                recipient_id: recipientId,
                permissions: role ? role.value : "read",
                scope: currentScope,
            }),
        });
        if (status) status.textContent = "Shared with " + (email || "the selected account") + ". It now appears in their Shared tab.";
        showNotification("Shared with " + (email || "the selected account"), "success");
    } catch (err) {
        if (status) status.textContent = "Share failed: " + err.message;
    }
}

async function loadShareEmailSenders() {
    var select = document.getElementById("share-email-sender");
    var status = document.getElementById("share-email-status");
    if (!select) return;
    try {
        var payload = await apiRequest("/share/email/accounts");
        var options = [];
        if (payload && payload.native && payload.native.configured) {
            options.push('<option value="">Server email' +
                (payload.native.from ? " (" + escapeHtml(payload.native.from) + ")" : "") +
                "</option>");
        }
        (payload && payload.accounts ? payload.accounts : []).forEach(function (account) {
            var label = account.email || account.display_name || "Connected account";
            options.push('<option value="' + escapeHtml(account.id) + '">' + escapeHtml(label) + "</option>");
        });
        var sendBtn = document.getElementById("share-email-send");
        var toInput = document.getElementById("share-email-to");
        if (!options.length) {
            select.innerHTML = '<option value="">No email sender configured</option>';
            select.disabled = true;
            if (sendBtn) sendBtn.disabled = true;
            if (toInput) toInput.disabled = true;
            if (status) status.textContent = "Connect Gmail/Outlook in Integrations or configure the server mailbox.";
        } else {
            select.disabled = false;
            if (sendBtn) sendBtn.disabled = false;
            if (toInput) toInput.disabled = false;
            select.innerHTML = options.join("");
        }
    } catch (error) {
        select.innerHTML = '<option value="">Sender lookup failed</option>';
        select.disabled = true;
        if (status) status.textContent = error.message || "Could not load email senders.";
    }
}

async function sendShareEmail(path) {
    var to = document.getElementById("share-email-to");
    var sender = document.getElementById("share-email-sender");
    var message = document.getElementById("share-email-message");
    var status = document.getElementById("share-email-status");
    var button = document.getElementById("share-email-send");
    if (!to || !to.value.trim()) {
        if (status) status.textContent = "Type at least one recipient email.";
        return;
    }
    if (button) button.disabled = true;
    if (status) status.textContent = "Sending…";
    try {
        var payload = await apiRequest("/share/email", {
            method: "POST",
            body: JSON.stringify({
                bucket: getEffectiveBucket(),
                path: path,
                scope: currentScope,
                origin: window.location.origin,
                recipients: [to.value],
                message: message ? message.value : "",
                expires_at: shareExpiryIso(),
                account_id: sender && !sender.disabled ? sender.value : "",
            }),
        });
        if (status) {
            status.textContent = "Sent to " + (payload.recipients || []).join(", ") +
                (payload.failed ? " (" + payload.failed + " failed)" : "") + ".";
        }
        if (window.GBToasts) GBToasts.show("Email sent", "The share link was emailed.", "success");
    } catch (error) {
        if (status) status.textContent = error.message || "The email could not be sent.";
    } finally {
        if (button) button.disabled = false;
    }
}

async function revokeShareLink(token, path) {
    if (!token) return;
    if (!confirm("Revoke this link? Anyone holding it will lose access immediately.")) return;
    try {
        await apiRequest("/revoke-link", {
            method: "POST",
            body: JSON.stringify({
                token: token,
                bucket: getEffectiveBucket(),
                path: path,
                scope: currentScope,
            }),
        });
        var row = document.getElementById("share-link-row");
        if (row) {
            row.style.display = "none";
            row.dataset.token = "";
        }
        var createBtn = document.getElementById("share-create-btn");
        if (createBtn) createBtn.style.display = "inline-block";
        var status = document.getElementById("share-status");
        if (status) status.textContent = "Link revoked. Create a new one if you still need to share.";
        showNotification("Link revoked", "success");
    } catch (error) {
        showNotification("Revoke failed: " + (error.message || "unknown error"), "error");
    }
}

async function revokeShareLinkFromModal(path) {
    var row = document.getElementById("share-link-row");
    var token = row ? row.dataset.token : "";
    if (!token) return;
    await revokeShareLink(token, path);
}

function closeModal(modalId) {
    var el = document.getElementById(modalId);
    if (el) el.remove();
}

function copyShareLink() {
    const input = document.getElementById("share-link-input");
    if (input) { input.select(); document.execCommand("copy"); showNotification("Share link copied to clipboard!", "success"); }
}

function showEditorModal(path, fileName, content) {
    var modal = document.getElementById("editor-modal");
    if (modal) modal.remove();
    const ext = (fileName.split(".").pop() || "txt").toLowerCase();
    modal = document.createElement("div");
    modal.id = "editor-modal";
    modal.className = "modal-overlay";
    var headerHtml = '<div class="editor-header"><div class="editor-title"><span class="editor-icon">\uD83D\uDCDD</span><span class="editor-filename">' + escapeHtml(fileName) + '</span><span class="editor-status" id="editor-status"></span></div><div class="editor-actions"><button class="btn-secondary" onclick="DriveModule.closeEditor()">Cancel</button><button class="btn-primary" onclick="DriveModule.saveEditorContent()"><span>\uD83D\uDCBE</span> Save</button></div></div>';
    var bodyHtml = '<div class="editor-body"><textarea id="editor-textarea" class="editor-textarea" spellcheck="false" data-path="' + escapeHtml(path) + '" data-ext="' + ext + '"></textarea></div>';
    var footerHtml = '<div class="editor-footer"><span class="editor-info">Line: <span id="editor-line">1</span>, Col: <span id="editor-col">1</span></span><span class="editor-info">' + ext.toUpperCase() + '</span></div>';
    modal.innerHTML = '<div class="modal-content editor-modal-content">' + headerHtml + bodyHtml + footerHtml + '</div>';
    document.body.appendChild(modal);
    const textarea = document.getElementById("editor-textarea");
    if (textarea) { textarea.value = content || ""; textarea.focus(); }
    textarea.addEventListener("input", function() { document.getElementById("editor-status").textContent = "\u25CF Modified"; });
    textarea.addEventListener("click", updateEditorCursor);
    textarea.addEventListener("keyup", updateEditorCursor);
    textarea.addEventListener("keydown", function(e) {
        if (e.key === "s" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); saveEditorContent(); }
        if (e.key === "Escape") { closeEditor(); }
        if (e.key === "Tab") {
            e.preventDefault();
            var start = textarea.selectionStart;
            var end = textarea.selectionEnd;
            textarea.value = textarea.value.substring(0, start) + "  " + textarea.value.substring(end);
            textarea.selectionStart = textarea.selectionEnd = start + 2;
        }
    });
    modal.addEventListener("click", function(e) { if (e.target === modal) closeEditor(); });
}

function updateEditorCursor() {
    const textarea = document.getElementById("editor-textarea");
    if (!textarea) return;
    const text = textarea.value.substring(0, textarea.selectionStart);
    const lines = text.split("\n");
    document.getElementById("editor-line").textContent = lines.length;
    document.getElementById("editor-col").textContent = lines[lines.length - 1].length + 1;
}

async function saveEditorContent() {
    const textarea = document.getElementById("editor-textarea");
    if (!textarea) return;
    const path = textarea.dataset.path;
    const content = textarea.value;
    const statusEl = document.getElementById("editor-status");
    statusEl.textContent = "Saving...";
    try {
        await apiRequest("/write", {
            method: "POST",
            body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, content: content, scope: currentScope }),
        });
        statusEl.textContent = "\u2713 Saved";
        if (typeof saveFileVersion === "function") saveFileVersion(path, content);
        showNotification("File saved successfully", "success");
        setTimeout(function() { if (statusEl) statusEl.textContent = ""; }, 2000);
    } catch (err) {
        statusEl.textContent = "\u2717 Save failed";
        showNotification("Failed to save: " + err.message, "error");
    }
}

function closeEditor() {
    const modal = document.getElementById("editor-modal");
    const statusEl = document.getElementById("editor-status");
    if (statusEl && statusEl.textContent.indexOf("Modified") !== -1) {
        if (!confirm("You have unsaved changes. Close anyway?")) return;
    }
    if (modal) modal.remove();
}

function openNewBotModal() {
    var modal = document.getElementById("new-bot-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "new-bot-modal";
    modal.className = "modal-overlay";
    modal.innerHTML = '<div class="modal-content" style="max-width:480px">'
        + '<div class="editor-header"><div class="editor-title"><span class="editor-icon">\uD83E\uDD16</span><span class="editor-filename">Create New Bot</span></div><button class="btn-icon" onclick="closeNewBotModal()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">\u00D7</button></div>'
        + '<div style="padding:24px">'
        + '<label style="display:block;margin-bottom:8px;color:var(--text-secondary);font-size:14px">Bot Name</label>'
        + '<input type="text" id="new-bot-name" placeholder="my-new-bot" style="width:100%;padding:10px 14px;background:#0f172a;border:1px solid var(--border);border-radius:6px;color:#f8fafc;font-size:14px;box-sizing:border-box;margin-bottom:12px" oninput="document.getElementById(\'new-bot-preview\').textContent=this.value + \'.gbai\'" />'
        + '<p style="color:var(--text-secondary);font-size:12px;margin-bottom:16px">Lowercase letters, numbers, and hyphens only (3-50 chars). Creates bucket: <code id="new-bot-preview" style="color:var(--primary)">my-new-bot.gbai</code></p>'
        + '<div style="display:flex;gap:8px;justify-content:flex-end">'
        + '<button class="btn-secondary" onclick="closeNewBotModal()">Cancel</button>'
        + '<button class="btn-primary" id="confirm-new-bot-btn" onclick="confirmNewBot()">Create Bot</button>'
        + '</div></div></div>';
    document.body.appendChild(modal);
    document.getElementById("new-bot-name").focus();
}

function closeNewBotModal() {
    var modal = document.getElementById("new-bot-modal");
    if (modal) modal.remove();
}

async function confirmNewBot() {
    var name = document.getElementById("new-bot-name").value.trim();
    if (!name) { showNotification("Enter a bot name", "error"); return; }
    var btn = document.getElementById("confirm-new-bot-btn");
    btn.disabled = true;
    btn.textContent = "Creating...";
    await createNewBot(name);
    btn.disabled = false;
    btn.textContent = "Create Bot";
}
