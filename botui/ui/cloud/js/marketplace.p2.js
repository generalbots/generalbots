/* Skills Marketplace — part 2: detail drawer, install flow, wiring */
"use strict";

function mktLoginUrl() {
    return window.GB_LOGIN_URL || "http://localhost:5000";
}

async function openSkillDetail(slug) {
    const body = document.getElementById("mkt-detail-body");
    MktState.selectedSlug = slug;
    try {
        MktState.detail = await mktApi("/api/marketplace/skills/" + encodeURIComponent(slug));
    } catch (err) {
        if (body) body.innerHTML = '<p class="mkt-state">Failed to load details: ' + mktEsc(err.message) + "</p>";
    }
    if (!MktState.detail) return;
    renderSkillDetail(MktState.detail);
    document.getElementById("mkt-modal").classList.add("open");
    loadBotsForInstall();
}

function mktManifestOf(detail) {
    return detail.manifest || (detail.latest_version_manifest) || {};
}

function mktPermissionsOf(detail) {
    const manifest = mktManifestOf(detail);
    let perms = manifest.permissions || detail.permissions || [];
    if (!Array.isArray(perms)) perms = [];
    return perms;
}

function mktScreenshotsOf(detail) {
    const manifest = mktManifestOf(detail);
    let shots = manifest.screenshots || [];
    if (!Array.isArray(shots)) shots = [];
    return shots;
}

const MKT_PERM_ICONS = {
    network: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>',
    storage: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>',
    llm: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>',
    default: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>'
};

function mktPermIcon(perm) {
    const key = String(perm).toLowerCase();
    for (const candidate of Object.keys(MKT_PERM_ICONS)) {
        if (key.includes(candidate)) return MKT_PERM_ICONS[candidate];
    }
    return MKT_PERM_ICONS.default;
}

function renderSkillDetail(detail) {
    const body = document.getElementById("mkt-detail-body");
    if (!body) return;
    const glyph = detail.icon_glyph || (detail.name || "?").charAt(0).toUpperCase();
    const perms = mktPermissionsOf(detail);
    const shots = mktScreenshotsOf(detail);
    let html = '<div class="mkt-detail-head">';
    html += '<span class="mkt-avatar lg" aria-hidden="true">' + mktEsc(glyph) + "</span>";
    html += "<div><h3 style=\"margin:0;font-size:1.05rem;color:var(--text)\">" + mktEsc(detail.name) + "</h3>";
    html += '<div class="mkt-publisher">by ' + mktEsc(detail.publisher_name || "Community") +
        " · v" + mktEsc(detail.latest_version || "—") +
        " · ↓ " + mktEsc(mktFmtDownloads(detail.downloads)) + "</div></div></div>";
    html += '<p class="mkt-desc" style="-webkit-line-clamp:unset">' + mktEsc(detail.description || "") + "</p>";
    if (perms.length) {
        html += '<div class="mkt-section-title">Requested permissions</div>';
        html += perms.map(function (perm) {
            const label = typeof perm === "string" ? perm : (perm.name || JSON.stringify(perm));
            return '<div class="mkt-perm-row">' + mktPermIcon(label) + "<span>" + mktEsc(label) + "</span></div>";
        }).join("");
    } else {
        html += '<div class="mkt-section-title">Requested permissions</div><div class="mkt-perm-row"><span>No special permissions requested.</span></div>';
    }
    if (shots.length) {
        html += '<div class="mkt-section-title">Screenshots</div>' + shots.map(function () {
            return '<div class="mkt-shot">Screenshot preview unavailable</div>';
        }).join("");
    }
    if (detail.changelog) {
        html += '<div class="mkt-section-title">Changelog</div><div style="font-size:.76rem;color:var(--muted);white-space:pre-wrap">' +
            mktEsc(detail.changelog) + "</div>";
    }
    body.innerHTML = html;
}

async function loadBotsForInstall() {
    const select = document.getElementById("mkt-bot-select");
    if (!select) return;
    select.innerHTML = '<option value="">Loading bots…</option>';
    try {
        const data = await mktApi("/api/cloud/bots");
        const bots = data.items || data.bots || data;
        if (!Array.isArray(bots) || !bots.length) {
            select.innerHTML = '<option value="">No bots available — create one first.</option>';
            return;
        }
        select.innerHTML = bots.map(function (bot) {
            const id = bot.id || bot.bot_id || bot.uuid;
            const name = bot.name || bot.bot_name || id;
            return '<option value="' + mktEsc(id) + '">' + mktEsc(name) + "</option>";
        }).join("");
    } catch (err) {
        select.innerHTML = '<option value="">Failed to load bots (' + mktEsc(err.message) + ")</option>";
    }
}

function requireAuthOrRedirect() {
    const authed = typeof isAuthenticated === "function"
        ? isAuthenticated()
        : !!localStorage.getItem("management_token");
    if (!authed) {
        window.location.href = mktLoginUrl() + "/login?next=" + encodeURIComponent("/marketplace");
        return false;
    }
    return true;
}

async function installSelectedSkill() {
    if (!requireAuthOrRedirect()) return;
    const botId = document.getElementById("mkt-bot-select").value;
    const btn = document.getElementById("mkt-install-btn");
    if (!botId) {
        if (typeof showToast === "function") showToast("Select a target bot to install.", "error");
        return;
    }
    btn.disabled = true;
    try {
        await mktApi("/api/marketplace/skills/" + encodeURIComponent(MktState.selectedSlug) + "/install", {
            method: "POST",
            body: JSON.stringify({ bot_id: botId })
        });
        const botName = document.getElementById("mkt-bot-select").selectedOptions[0].textContent;
        closeMarketplaceModal();
        if (typeof showToast === "function") showToast("Skill installed into " + botName + ".", "success");
    } catch (err) {
        if (typeof showToast === "function") showToast("Install failed: " + err.message, "error");
    } finally {
        btn.disabled = false;
    }
}

function closeMarketplaceModal() {
    const modal = document.getElementById("mkt-modal");
    if (modal) modal.classList.remove("open");
}

document.addEventListener("click", function (event) {
    const target = event.target.closest("[data-mkt-close],[data-mkt-open],[data-mkt-tag]");
    if (!target) return;
    if (target.hasAttribute("data-mkt-close")) { closeMarketplaceModal(); return; }
    if (target.hasAttribute("data-mkt-open")) { openSkillDetail(target.getAttribute("data-mkt-open")); return; }
    if (target.hasAttribute("data-mkt-tag")) {
        MktState.activeTag = target.getAttribute("data-mkt-tag");
        mktLoadSkills();
        return;
    }
});

document.addEventListener("DOMContentLoaded", function () {
    const search = document.getElementById("mkt-search");
    search.addEventListener("input", function () {
        clearTimeout(MktState.searchTimer);
        MktState.searchTimer = setTimeout(function () {
            MktState.query = search.value.trim();
            mktLoadSkills();
        }, 350);
    });
    document.getElementById("mkt-install-btn").addEventListener("click", installSelectedSkill);
    mktLoadSkills();
});
