/* Skills Marketplace — part 1: state, catalog fetch, tag chips, grid rendering */
"use strict";

const MktState = {
    skills: [],
    tags: [],
    activeTag: "",
    query: "",
    searchTimer: null,
    selectedSlug: null,
    detail: null
};

function mktEsc(value) {
    const div = document.createElement("div");
    div.textContent = value == null ? "" : String(value);
    return div.innerHTML;
}

function mktFmtDownloads(count) {
    const n = Number(count) || 0;
    if (n >= 1000000) return (n / 1000000).toFixed(1).replace(/\.0$/, "") + "M";
    if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, "") + "k";
    return String(n);
}

async function mktApi(endpoint, options = {}) {
    const headers = Object.assign({ "Content-Type": "application/json" }, options.headers || {});
    const token = typeof getToken === "function" ? getToken() : localStorage.getItem("management_token");
    if (token) headers["Authorization"] = "Bearer " + token;
    const response = await fetch(endpoint, Object.assign({}, options, { headers }));
    if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.error || "Request failed (" + response.status + ")");
    }
    return response.json();
}

async function mktLoadSkills() {
    const grid = document.getElementById("mkt-grid");
    try {
        const params = new URLSearchParams();
        if (MktState.query) params.set("q", MktState.query);
        if (MktState.activeTag) params.set("tag", MktState.activeTag);
        const qs = params.toString();
        const data = await mktApi("/api/marketplace/skills" + (qs ? "?" + qs : ""));
        MktState.skills = data.items || [];
        renderMktTags();
        renderMktGrid();
    } catch (err) {
        if (grid) grid.innerHTML = '<div class="mkt-state">Failed to load marketplace: ' +
            mktEsc(err.message) + "</div>";
    }
}

function mktCollectTags() {
    const counts = {};
    for (const skill of MktState.skills) {
        for (const tag of skill.tags || []) {
            counts[tag] = (counts[tag] || 0) + 1;
        }
    }
    MktState.tags = Object.keys(counts).sort();
}

function renderMktTags() {
    mktCollectTags();
    const row = document.getElementById("mkt-tags");
    if (!row) return;
    let html = '<button type="button" class="mkt-tag-chip' + (MktState.activeTag === "" ? " active" : "") +
        '" data-mkt-tag="">All</button>';
    for (const tag of MktState.tags) {
        html += '<button type="button" class="mkt-tag-chip' + (MktState.activeTag === tag ? " active" : "") +
            '" data-mkt-tag="' + mktEsc(tag) + '">' + mktEsc(tag) + "</button>";
    }
    row.innerHTML = html;
}

function mktCardHtml(skill) {
    const glyph = skill.icon_glyph || (skill.name || "?").charAt(0).toUpperCase();
    const tags = (skill.tags || []).slice(0, 3)
        .map(t => '<span class="mkt-mini-tag">' + mktEsc(t) + "</span>").join("");
    let html = '<button type="button" class="mkt-card" data-mkt-open="' + mktEsc(skill.slug) + '" aria-label="View details for ' + mktEsc(skill.name) + '">';
    html += '<div class="mkt-card-head">';
    html += '<span class="mkt-avatar" aria-hidden="true">' + mktEsc(glyph) + "</span>";
    html += "<div><h3 class=\"mkt-name\">" + mktEsc(skill.name) + "</h3>" +
        '<div class="mkt-publisher">by ' + mktEsc(skill.publisher_name || "Community") + "</div></div>";
    html += "</div>";
    html += '<p class="mkt-desc">' + mktEsc(skill.description || "") + "</p>";
    html += '<div class="mkt-card-foot">' + tags +
        '<span class="mkt-downloads">↓ ' + mktEsc(mktFmtDownloads(skill.downloads)) + "</span></div>";
    html += "</button>";
    return html;
}

function renderMktGrid() {
    const grid = document.getElementById("mkt-grid");
    if (!grid) return;
    grid.innerHTML = MktState.skills.length
        ? MktState.skills.map(mktCardHtml).join("")
        : '<div class="mkt-empty">No skills match your search. Try different keywords or clear the tag filter.</div>';
}
