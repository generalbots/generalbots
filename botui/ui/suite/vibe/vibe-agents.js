function updateMantis1(status, detail) {
var card = document.querySelector(
'.as-agent-card[data-agent-id="1"]',
);
if (!card) return;
var bar = card.querySelector(".as-agent-bar .as-bar-fill");
if (status === "working") {
card.style.borderLeftColor = "#f59e0b";
if (!card.querySelector(".as-agent-bar")) {
var barWrapper = document.createElement("div");
barWrapper.className = "as-agent-bar";
barWrapper.innerHTML =
'<div class="as-bar-fill bred" style="width:0%;transition:width 0.5s;"></div>';
card.appendChild(barWrapper);
}
} else if (status === "done") {
card.style.borderLeftColor = "var(--accent)";
bar = card.querySelector(".as-bar-fill");
if (bar) bar.style.width = "100%";
setTimeout(function () {
var b = card.querySelector(".as-agent-bar");
if (b) b.remove();
}, 2000);
}
}

function updateAgentCard(agentId, status, detail) {
var card = document.querySelector(
'.as-agent-card[data-agent-id="' + agentId + '"]',
);
if (!card) return;
card.style.opacity = "1";

var badge = card.querySelector(".as-badge");
var dot = card.querySelector(".as-status-dot");

if (status === "WORKING") {
card.style.borderLeft = "3px solid #f59e0b";
if (dot) {
dot.className = "as-status-dot yellow";
}
if (badge) {
badge.textContent = "WORKING";
badge.className = "as-badge badge-bred";
}
if (!card.querySelector(".as-agent-bar")) {
var barWrapper = document.createElement("div");
barWrapper.className = "as-agent-bar";
barWrapper.innerHTML =
'<div class="as-bar-fill bred" style="width:0%;transition:width 0.5s;"></div>';
card.appendChild(barWrapper);
}
} else if (status === "EVOLVED" || status === "DONE") {
card.style.borderLeft = "3px solid var(--accent)";
if (dot) {
dot.className = "as-status-dot green";
}
if (badge) {
badge.textContent = "EVOLVED";
badge.className = "as-badge badge-evolved";
}
var agBar = card.querySelector(".as-bar-fill");
if (agBar) agBar.style.width = "100%";
setTimeout(function () {
var b = card.querySelector(".as-agent-bar");
if (b) b.remove();
}, 2000);
} else if (status === "BRED") {
card.style.borderLeft = "3px solid #f59e0b";
if (dot) {
dot.className = "as-status-dot yellow";
}
if (badge) {
badge.textContent = "BRED";
badge.className = "as-badge badge-bred";
}
} else if (status === "FAILED") {
card.style.borderLeft = "3px solid #ef4444";
if (dot) {
dot.className = "as-status-dot red";
}
if (badge) {
badge.textContent = "FAILED";
badge.className = "as-badge badge-bred";
badge.style.background = "#ef4444";
}
}

if (detail) {
    var detailEl = card.querySelector(".as-agent-detail");
    if (!detailEl) {
        detailEl = document.createElement("span");
        detailEl.className = "as-agent-detail";
        detailEl.style.cssText =
            "font-size:10px;color: var(--text-muted);display:block;padding:0 12px 4px;";
        var body = card.querySelector(".as-agent-body");
        if (body) body.after(detailEl);
    }
    detailEl.textContent = detail;
}
}

function vibeBreedAgent() {
    var list = document.getElementById("asAgentList");
    if (!list) return;
    var next = list.querySelectorAll(".as-agent-card").length + 1;
    var card = document.createElement("div");
    card.className = "as-agent-card";
    card.style.opacity = "0.6";
    card.setAttribute("data-agent-id", String(next));
    card.setAttribute("draggable", "true");
    card.innerHTML =
        '<div class="as-agent-header"><span class="as-status-dot gray"></span>' +
        '<span class="as-agent-name">Mantis #' + next + "</span>" +
        '<span class="as-drag-handle" style="margin-left:auto">⋮</span></div>' +
        '<div class="as-agent-body"><span class="as-agent-icons" style="filter: grayscale(1)">🥚</span>' +
        '<span class="as-badge badge-wild" style="background: var(--surface-active, #ccc)">WILD</span></div>';
    list.appendChild(card);
    return card;
}

function vibeCreateWorkspace() {
    var list = document.getElementById("asWorkspaceList");
    if (!list) return;
    var n = list.querySelectorAll(".as-workspace-item").length + 1;
    var slug = ("new-project-" + n)
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-");
    var item = document.createElement("div");
    item.className = "as-workspace-item";
    item.innerHTML =
        '<button class="as-workspace-toggle" type="button" style="background: var(--bg);border-left: 3px solid var(--accent);">' +
        '<span class="as-workspace-arrow">▼</span><span>New Project ' + n + "</span></button>" +
        '<div class="as-workspace-body" style="display:block">' +
        '<div class="as-workspace-dropzone" data-workspace="' + slug + '">' +
        "Drag a Mantis to Include</div></div>";
    list.appendChild(item);
    return item;
}

document.addEventListener("dragstart", function (e) {
    var card = e.target.closest(".as-agent-card");
    if (!card) return;
    card.classList.add("dragging");
    if (e.dataTransfer) {
        e.dataTransfer.setData("text/plain", card.getAttribute("data-agent-id"));
        e.dataTransfer.effectAllowed = "move";
    }
});

document.addEventListener("dragover", function (e) {
    var zone = e.target.closest(".as-workspace-dropzone");
    if (!zone) return;
    e.preventDefault();
    zone.classList.add("drag-over");
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
});

document.addEventListener("dragleave", function (e) {
    var zone = e.target.closest(".as-workspace-dropzone");
    if (zone && !zone.contains(e.relatedTarget)) {
        zone.classList.remove("drag-over");
    }
});

document.addEventListener("drop", function (e) {
    var zone = e.target.closest(".as-workspace-dropzone");
    if (!zone) return;
    e.preventDefault();
    zone.classList.remove("drag-over");
    var card = document.querySelector(".as-agent-card.dragging");
    if (!card) return;
    var nameEl = card.querySelector(".as-agent-name");
    var name = nameEl ? nameEl.textContent : "Mantis";
    var prev = zone.previousElementSibling;
    if (prev && prev.className === "as-workspace-agent" && prev.textContent === name) {
        return;
    }
    var chip = document.createElement("div");
    chip.className = "as-workspace-agent";
    chip.textContent = name;
    zone.parentElement.insertBefore(chip, zone);
});

document.addEventListener("dragend", function (e) {
    var card = e.target.closest(".as-agent-card");
    if (card) card.classList.remove("dragging");
    document
        .querySelectorAll(".as-workspace-dropzone.drag-over")
        .forEach(function (z) {
            z.classList.remove("drag-over");
        });
});

(function () {
    var __cb = function () {
        document
            .querySelectorAll("#asAgentList .as-agent-card")
            .forEach(function (c) {
                c.setAttribute("draggable", "true");
            });
    };
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", __cb);
    } else {
        __cb();
    }
})();
