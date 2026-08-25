"use strict";

(function () {
    var currentProjectId = null;
    var selectedMemberId = null;   // resolved UUID from the typeahead
    var selectedTransferId = null; // resolved UUID from the transfer typeahead

    function projectId() {
        if (currentProjectId) return currentProjectId;
        if (typeof window.currentProjectId !== "undefined" && window.currentProjectId) {
            currentProjectId = window.currentProjectId;
        }
        return currentProjectId;
    }

    async function api(path, options) {
        var resp = await fetch(path, options);
        var data;
        try {
            data = await resp.json();
        } catch (e) {
            data = { success: false, error: "invalid response" };
        }
        if (!resp.ok) {
            data.http_status = resp.status;
        }
        return data;
    }

    function esc(s) {
        var div = document.createElement("div");
        div.textContent = s == null ? "" : String(s);
        return div.innerHTML;
    }

    function displayName(m) {
        if (m.group_name) return "group " + m.group_name;
        if (m.user_name) return m.user_name;
        if (m.email) return m.email;
        return "member";
    }

    function renderMembers(members) {
        var listEl = document.getElementById("vibeMembersList");
        if (members.length === 0) {
            listEl.innerHTML = "<div style='color:#999;font-size:13px;'>No members yet. Add a member below.</div>";
            return;
        }
        var roleBadge = { owner: "#f7b500", admin: "#4a9eff", developer: "#84d669", viewer: "#888" };
        listEl.innerHTML = members.map(function (m) {
            var color = roleBadge[m.role] || "#888";
            return "<div style='display:flex;justify-content:space-between;align-items:center;padding:8px 0;border-bottom:1px solid var(--border,#333);font-size:13px;'>"
                + "<span style='color:var(--text,#eee);'>"
                + (m.user_name ? "<span style='font-weight:600;'>" + esc(m.user_name) + "</span>"
                    + (m.email ? " <span style='color:var(--text-muted,#999);'>· " + esc(m.email) + "</span>" : "")
                    : esc(displayName(m)))
                + "</span>"
                + "<span style='color:" + color + ";text-transform:uppercase;font-size:11px;font-weight:700;'>" + esc(m.role) + "</span>"
                + "</div>";
        }).join("");
    }

    async function load() {
        var listEl = document.getElementById("vibeMembersList");
        var pid = projectId();
        if (!pid) {
            listEl.innerHTML = "<div style='color:#999;font-size:13px;'>No project selected. Create a project first.</div>";
            return;
        }
        listEl.innerHTML = "<div style='color:#999;font-size:13px;'>Loading…</div>";
        var data = await api("/api/vibe/projects/" + pid + "/members");
        if (!data.success) {
            listEl.innerHTML = "<div style='color:#f88;font-size:13px;'>" + esc(data.error || "Failed to load members") + "</div>";
            return;
        }
        loadMeter(pid);
        renderMembers(data.members || []);
    }

    // ---- typeahead ----------------------------------------------------------

    function suggestionsFor(inputId, boxId, onPick) {
        var input = document.getElementById(inputId);
        var box = document.getElementById(boxId);
        if (!input || !box) return;
        var timer = null;

        input.addEventListener("input", function () {
            clearTimeout(timer);
            var q = input.value.trim();
            if (q.length < 2) {
                box.style.display = "none";
                box.innerHTML = "";
                return;
            }
            timer = setTimeout(function () {
                api("/api/vibe/users/search?q=" + encodeURIComponent(q)).then(function (data) {
                    var users = (data && data.users) || [];
                    if (!users.length) {
                        box.innerHTML = "<div style='padding:8px 10px;color:#888;font-size:12px;'>No users found</div>";
                        box.style.display = "block";
                        return;
                    }
                    box.innerHTML = users.map(function (u) {
                        var label = u.username + (u.email ? " · " + u.email : "");
                        return "<div data-id='" + esc(u.id) + "' style='padding:8px 10px;cursor:pointer;font-size:13px;color:var(--text,#eee);border-bottom:1px solid var(--border,#333);'>"
                            + "<span style='font-weight:600;'>" + esc(u.username) + "</span>"
                            + " <span style='color:var(--text-muted,#999);'>" + esc(u.email) + "</span></div>";
                    }).join("");
                    box.style.display = "block";
                });
            }, 250);
        });

        box.addEventListener("click", function (e) {
            var row = e.target.closest("[data-id]");
            if (!row) return;
            var id = row.getAttribute("data-id");
            input.value = row.querySelector("span").textContent;
            box.style.display = "none";
            box.innerHTML = "";
            onPick(id);
        });

        document.addEventListener("click", function (e) {
            if (!input.contains(e.target) && !box.contains(e.target)) {
                box.style.display = "none";
            }
        });
    }

    function setupSuggest() {
        suggestionsFor("vibeMemberTarget", "vibeMemberSuggest", function (id) {
            selectedMemberId = id;
        });
        suggestionsFor("vibeTransferTarget", "vibeTransferSuggest", function (id) {
            selectedTransferId = id;
        });
    }

    // ---- actions ------------------------------------------------------------

    async function add() {
        var pid = projectId();
        if (!pid) return;
        var kind = document.getElementById("vibeMemberKind").value;
        var role = document.getElementById("vibeMemberRole").value;
        var target = kind === "group"
            ? document.getElementById("vibeMemberTarget").value.trim()
            : (selectedMemberId || document.getElementById("vibeMemberTarget").value.trim());
        if (!target) {
            alert("Choose a user or enter a group name first.");
            return;
        }
        var path = kind === "group"
            ? "/api/vibe/projects/" + pid + "/members/group/" + encodeURIComponent(target)
            : "/api/vibe/projects/" + pid + "/members/" + encodeURIComponent(target);
        var data = await api(path, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ role: role })
        });
        if (data.success) {
            document.getElementById("vibeMemberTarget").value = "";
            selectedMemberId = null;
            await load();
        } else {
            alert(data.error || "Failed to add member");
        }
    }

    async function transfer() {
        var pid = projectId();
        if (!pid) return;
        var target = selectedTransferId || document.getElementById("vibeTransferTarget").value.trim();
        if (!target) {
            alert("Choose a new owner first.");
            return;
        }
        var label = document.getElementById("vibeTransferTarget").value.trim();
        if (!confirm("Transfer ownership of this project to " + label + "?")) return;
        var data = await api("/api/vibe/projects/" + pid + "/members/transfer-ownership", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ user_id: target })
        });
        if (data.success) {
            document.getElementById("vibeTransferTarget").value = "";
            selectedTransferId = null;
            await load();
        } else {
            alert(data.error || "Transfer failed");
        }
    }

    async function loadMeter(pid) {
        var el = document.getElementById("vibeMeterUsage");
        if (!el) return;
        el.textContent = "Loading usage…";
        var data = await api("/api/vibe/projects/" + pid + "/metering");
        if (!data.success || !data.summary) {
            el.textContent = "";
            return;
        }
        var s = data.summary;
        var rows = s.rows || [];
        if (rows.length === 0) {
            el.textContent = "Usage: 0 minutes (" + s.plan + " plan)";
            return;
        }
        var parts = rows.map(function (r) {
            return r.meter + ": " + Number(r.amount).toFixed(2);
        });
        el.textContent = "Usage (" + s.plan + " plan): " + parts.join(", ");
    }

    function open() {
        var modal = document.getElementById("vibeMembersModal");
        if (!modal) return;
        // Floating tool window (VB6-style); falls back to in-window display
        // when the desktop shell is absent (isolated run).
        if (window.VibeWindows) window.VibeWindows.openMembers();
        modal.style.display = "flex";
        var wm = window.WindowManager;
        if (wm && !/[?&]isolated=1/.test(window.location.search)) {
            wm.focusWindow("vibe-members");
        }
        setupSuggest();
        load();
    }

    function close() {
        var modal = document.getElementById("vibeMembersModal");
        if (modal) modal.style.display = "none";
        var wm = window.WindowManager;
        if (wm && wm.getWindow("vibe-members")) wm.close("vibe-members");
    }

    document.addEventListener("gb:vibe-project", function (e) {
        if (e.detail && e.detail.id) {
            currentProjectId = e.detail.id;
        }
    });

    window.VibeMembers = { open: open, close: close, add: add, transfer: transfer };
})();
