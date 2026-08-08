"use strict";

(function () {
    var modal = null;
    var currentProjectId = null;

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
        div.textContent = s;
        return div.innerHTML;
    }

    async function load() {
        var listEl = document.getElementById("vibeMembersList");
        var roleEl = document.getElementById("vibeMembersRole");
        var pid = projectId();
        if (!pid) {
            listEl.innerHTML = "<div style='color:#999;font-size:13px;'>No project selected. Create a project first.</div>";
            roleEl.textContent = "";
            return;
        }
        roleEl.textContent = "";
        listEl.textContent = "Loading...";
        var data = await api("/api/vibe/projects/" + pid + "/members");
        if (!data.success) {
            listEl.innerHTML = "<div style='color:#f88;font-size:13px;'>" + esc(data.error || "Failed to load members") + "</div>";
            return;
        }
        loadMeter(pid);
        var members = data.members || [];
        listEl.innerHTML = members.length === 0
            ? "<div style='color:#999;font-size:13px;'>No memberships yet.</div>"
            : members.map(function (m) {
                var who = m.user_id ? ("user " + m.user_id) : ("group " + (m.group_name || "?"));
                return "<div style='display:flex;justify-content:space-between;align-items:center;padding:6px 0;border-bottom:1px solid var(--border,#333);font-size:13px;'>"
                    + "<span style='color:var(--text,#eee);'>" + esc(who) + "</span>"
                    + "<span style='color:#8f8;'>" + esc(m.role) + "</span>"
                    + "</div>";
            }).join("");
    }

    async function add() {
        var pid = projectId();
        if (!pid) return;
        var target = document.getElementById("vibeMemberTarget").value.trim();
        var role = document.getElementById("vibeMemberRole").value;
        var kind = document.getElementById("vibeMemberKind").value;
        if (!target) return;
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
            await load();
        } else {
            alert(data.error || "Failed to add member");
        }
    }

    async function transfer() {
        var pid = projectId();
        if (!pid) return;
        var target = document.getElementById("vibeTransferTarget").value.trim();
        if (!target) return;
        if (!confirm("Transfer ownership of this project to " + target + "?")) return;
        var data = await api("/api/vibe/projects/" + pid + "/members/transfer-ownership", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ user_id: target })
        });
        if (data.success) {
            document.getElementById("vibeTransferTarget").value = "";
            await load();
        } else {
            alert(data.error || "Transfer failed");
        }
    }

    async function loadMeter(pid) {
        var el = document.getElementById("vibeMeterUsage");
        if (!el) return;
        el.textContent = "Loading usage...";
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
            var label = r.meter === "vm_hours" ? "hours" : r.meter.replace(/_/g, " ");
            return r.meter + ": " + Number(r.amount).toFixed(2) + " " + label;
        });
        el.textContent = "Usage (" + s.plan + " plan): " + parts.join(", ");
    }

    function open() {
        var modal = document.getElementById("vibeMembersModal");
        if (!modal) return;
        modal.style.display = "flex";
        load();
    }

    function close() {
        var modal = document.getElementById("vibeMembersModal");
        if (modal) modal.style.display = "none";
    }

    document.addEventListener("gb:vibe-project", function (e) {
        if (e.detail && e.detail.id) {
            currentProjectId = e.detail.id;
        }
    });

    window.VibeMembers = { open: open, close: close, add: add, transfer: transfer };
})();