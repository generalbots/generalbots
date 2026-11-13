/* Drive Module v2.0 — 08 Modals: share links and sharing */
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
        '<div id="share-person-section" style="border-top:1px solid #334155; padding-top:14px;">' +
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

    // #1863 — desktop-local is a single-user install: "share with a person"
    // has no recipient directory to resolve, so the section is not offered.
    var product = window.productConfig || {};
    var desktopLocal = product.shell_mode === true ||
        product.desktop_local === true ||
        product.is_desktop === true ||
        product.no_security === true;
    if (desktopLocal) {
        var personSection = document.getElementById("share-person-section");
        if (personSection) personSection.style.display = "none";
    }

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

function shareLinkReach(url) {
    try {
        var host = new URL(url, window.location.href).hostname.toLowerCase();
        if (host === "localhost" || host === "127.0.0.1" || host === "::1" ||
            host === "[::1]" || host.endsWith(".localhost")) {
            return "Only this machine can open this link.";
        }
        return "Anyone who can reach " + host + " can open this link.";
    } catch (e) {
        return "";
    }
}

function showShareLink(result) {
    var status = document.getElementById("share-status");
    var row = document.getElementById("share-link-row");
    var input = document.getElementById("share-link-input");
    if (!row || !input) return;
    // The server returns an absolute URL when PUBLIC_BASE_URL is configured
    // (#1863); a relative URL is kept working on the machine that minted it.
    var rawUrl = String(result.url || "");
    input.value = /^https?:\/\//i.test(rawUrl) ? rawUrl : window.location.origin + rawUrl;
    var reach = shareLinkReach(input.value);
    row.style.display = "flex";
    row.dataset.token = result.token || "";
    if (status) {
        if (!result.expires_at) {
            status.textContent = "Anyone with this link can download the file until it is revoked (no expiry). " + reach;
        } else if (isExpired(result.expires_at)) {
            status.textContent = "This link expired on " + gbFormatDateTime(new Date(result.expires_at)) + ".";
        } else {
            status.textContent = "Anyone with this link can download the file until " +
                gbFormatDateTime(new Date(result.expires_at)) + ". Revoke it at any time. " + reach;
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
                // drive_shares keys recipients by the same identity /shared
                // filters on (uid = email); the picker's UUID never matched.
                recipient_id: email || recipientId,
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

