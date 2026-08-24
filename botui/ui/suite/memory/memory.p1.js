/* Memory app — part 1: state, API, filters, grid rendering, CRUD */
"use strict";

const MemState = {
    items: [],
    kinds: [],
    kindFilter: "",
    query: "",
    searchTimer: null
};

function memToken() {
    return localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
}

async function memApi(endpoint, options = {}) {
    const headers = Object.assign({ "Content-Type": "application/json" }, options.headers || {});
    const token = memToken();
    if (token) headers["Authorization"] = "Bearer " + token;
    const response = await fetch(endpoint, Object.assign({}, options, { headers }));
    if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.error || "Request failed (" + response.status + ")");
    }
    return response.json();
}

function memEsc(value) {
    const div = document.createElement("div");
    div.textContent = value == null ? "" : String(value);
    return div.innerHTML;
}

function memSetState(message, isError) {
    const el = document.getElementById("mem-state");
    if (!el) return;
    el.textContent = message || "";
    el.classList.toggle("error", !!isError);
}

const MEM_KINDS = ["fact", "preference", "goal", "event"];

function renderKindChips() {
    const row = document.getElementById("mem-kind-chips");
    if (!row) return;
    const counts = {};
    for (const item of MemState.items) counts[item.kind] = (counts[item.kind] || 0) + 1;
    const kinds = [...new Set(MEM_KINDS.concat(MemState.kinds))].filter(k => k !== "");
    let html = '<button type="button" class="mem-chip' + (MemState.kindFilter === "" ? " active" : "") +
        '" data-mem-kind="">All (' + MemState.items.length + ")</button>";
    for (const kind of kinds) {
        html += '<button type="button" class="mem-chip' + (MemState.kindFilter === kind ? " active" : "") +
            '" data-mem-kind="' + memEsc(kind) + '">' + memEsc(kind) + " (" + (counts[kind] || 0) + ")</button>";
    }
    row.innerHTML = html;
}

function filteredItems() {
    return MemState.items.filter(function (item) {
        if (MemState.kindFilter && item.kind !== MemState.kindFilter) return false;
        if (MemState.query) {
            const q = MemState.query.toLowerCase();
            if (!String(item.content || "").toLowerCase().includes(q)) return false;
        }
        return true;
    }).sort(function (a, b) {
        if (!!b.pinned !== !!a.pinned) return b.pinned ? 1 : -1;
        return new Date(b.updated_at || b.created_at) - new Date(a.updated_at || a.created_at);
    });
}

function memCardHtml(item) {
    const confidence = Math.round((item.confidence == null ? 0.8 : item.confidence) * 100);
    let html = '<article class="mem-card" data-mem-id="' + memEsc(item.id) + '">';
    html += '<div class="mem-card-top">';
    html += '<span class="mem-kind-badge">' + memEsc(item.kind) + "</span>";
    html += '<button type="button" class="mem-pin-btn' + (item.pinned ? " pinned" : "") +
        '" data-mem-pin="' + memEsc(item.id) + '" aria-pressed="' + (!!item.pinned) +
        '" aria-label="' + (item.pinned ? "Unpin memory" : "Pin memory") + '">' + (item.pinned ? "★" : "☆") + "</button>";
    html += "</div>";
    html += '<div class="mem-content">' + memEsc(item.content) + "</div>";
    html += '<div class="mem-meta"><span>' + confidence + "%</span>" +
        '<span class="mem-conf-bar" role="presentation"><span class="mem-conf-fill" style="width:' + confidence + '%"></span></span>' +
        "<span>" + memEsc(item.source || "") + "</span></div>";
    html += '<div class="mem-card-actions">';
    html += '<button type="button" class="mem-btn small ghost" data-mem-edit="' + memEsc(item.id) + '">Edit</button>';
    html += '<button type="button" class="mem-btn small danger" data-mem-delete="' + memEsc(item.id) + '">Delete</button>';
    html += "</div>";
    html += "</article>";
    return html;
}

function renderGrid() {
    const grid = document.getElementById("mem-grid");
    if (!grid) return;
    const items = filteredItems();
    document.getElementById("mem-count").textContent =
        MemState.items.length + " memor" + (MemState.items.length === 1 ? "y" : "ies");
    renderKindChips();
    grid.innerHTML = items.length
        ? items.map(memCardHtml).join("")
        : '<div class="mem-empty">No memories found. Add one or adjust the filters.</div>';
}

async function loadItems() {
    try {
        const params = new URLSearchParams();
        if (MemState.kindFilter) params.set("kind", MemState.kindFilter);
        if (MemState.query) params.set("q", MemState.query);
        const qs = params.toString();
        const data = await memApi("/api/memory/items" + (qs ? "?" + qs : ""));
        MemState.items = data.items || [];
        MemState.kinds = [...new Set(MemState.items.map(i => i.kind).filter(Boolean))];
        memSetState("");
        renderGrid();
    } catch (err) {
        memSetState("Failed to load memories: " + err.message, true);
    }
}

async function togglePin(itemId) {
    const item = MemState.items.find(i => i.id === itemId);
    if (!item) return;
    try {
        await memApi("/api/memory/items/" + encodeURIComponent(itemId), {
            method: "PUT",
            body: JSON.stringify({ pinned: !item.pinned })
        });
        item.pinned = !item.pinned;
        renderGrid();
    } catch (err) {
        memSetState("Pin failed: " + err.message, true);
    }
}

function startInlineEdit(itemId) {
    const card = document.querySelector('.mem-card[data-mem-id="' + itemId + '"]');
    const contentEl = card ? card.querySelector(".mem-content") : null;
    const item = MemState.items.find(i => i.id === itemId);
    if (!card || !contentEl || !item || card.querySelector(".mem-edit-area")) return;
    const actions = card.querySelector(".mem-card-actions");
    contentEl.classList.add("hidden");
    const area = document.createElement("textarea");
    area.className = "mem-edit-area";
    area.value = item.content || "";
    area.setAttribute("aria-label", "Edit memory content");
    contentEl.parentNode.insertBefore(area, actions);

    const saveBtn = document.createElement("button");
    saveBtn.type = "button";
    saveBtn.className = "mem-btn small primary";
    saveBtn.textContent = "Save";
    const cancelBtn = document.createElement("button");
    cancelBtn.type = "button";
    cancelBtn.className = "mem-btn small ghost";
    cancelBtn.textContent = "Cancel";
    actions.prepend(cancelBtn, saveBtn);

    async function finish(save) {
        saveBtn.disabled = true;
        try {
            if (save) {
                await memApi("/api/memory/items/" + encodeURIComponent(itemId), {
                    method: "PUT",
                    body: JSON.stringify({ content: area.value.trim() })
                });
                item.content = area.value.trim();
            }
            area.remove();
            saveBtn.remove();
            cancelBtn.remove();
            contentEl.classList.remove("hidden");
            renderGrid();
        } catch (err) {
            memSetState("Save failed: " + err.message, true);
            saveBtn.disabled = false;
        }
    }
    saveBtn.addEventListener("click", function () { finish(true); });
    cancelBtn.addEventListener("click", function () { finish(false); });
    area.focus();
}

async function deleteItem(itemId) {
    if (!window.confirm("Delete this memory permanently?")) return;
    try {
        await memApi("/api/memory/items/" + encodeURIComponent(itemId), { method: "DELETE" });
        await loadItems();
    } catch (err) {
        memSetState("Delete failed: " + err.message, true);
    }
}

function openAddModal() {
    const modal = document.getElementById("mem-modal");
    document.getElementById("mem-form").reset();
    modal.classList.add("open");
    document.getElementById("mem-f-content").focus();
}

async function saveNewItem() {
    const content = document.getElementById("mem-f-content").value.trim();
    const kind = document.getElementById("mem-f-kind").value;
    if (!content) {
        memSetState("Content is required.", true);
        return;
    }
    try {
        await memApi("/api/memory/items", {
            method: "POST",
            body: JSON.stringify({ kind: kind, content: content })
        });
        document.getElementById("mem-modal").classList.remove("open");
        await loadItems();
    } catch (err) {
        memSetState("Save failed: " + err.message, true);
    }
}
