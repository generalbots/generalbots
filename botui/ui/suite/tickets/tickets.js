(function () {
    document.querySelectorAll(".tickets-tab").forEach((tab) => {
        tab.addEventListener("click", function () {
            document.querySelectorAll(".tickets-tab").forEach((t) => t.classList.remove("active"));
            this.classList.add("active");
            const status = this.dataset.view;
            htmx.ajax("GET", "/api/ui/tickets?status=" + status, "#tickets-list-body");
        });
    });

    document.getElementById("tickets-new-btn").addEventListener("click", function () {
        openTicketsModal();
    });

    window.openTicketsModal = function () {
        document.getElementById("tickets-modal").classList.add("open");
    };

    window.closeTicketsModal = function () {
        document.getElementById("tickets-modal").classList.remove("open");
    };

    window.selectTicket = function (ticketId) {
        document.querySelectorAll(".ticket-item").forEach(function(item) {
            item.classList.remove("selected");
        });
        var el = document.querySelector('.ticket-item[data-id="' + ticketId + '"]');
        if (el) el.classList.add("selected");
        htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
    };

    window.changeTicketStatus = function (ticketId, newStatus) {
        fetch("/api/tickets/" + ticketId + "/status", {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ status: newStatus })
        }).then(function() {
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
            refreshTicketsList();
        });
    };

    window.assignTicket = function (ticketId) {
        var assignee = prompt("Assign to (email or name):");
        if (!assignee) return;
        fetch("/api/tickets/" + ticketId + "/assign", {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ assignee: assignee })
        }).then(function(r) {
            if (!r.ok) alert("Could not assign ticket. Use an existing user email.");
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
        });
    };

    window.addTicketNote = function (ticketId) {
        var note = document.getElementById("note-input-" + ticketId);
        if (!note || !note.value.trim()) return;
        fetch("/api/tickets/" + ticketId + "/activities", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                activity_type: "note",
                description: note.value.trim(),
                actor_name: (window.gbCurrentUser && window.gbCurrentUser.email) || "User"
            })
        }).then(function() {
            note.value = "";
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
        });
    };

    window.deleteTicket = function (ticketId) {
        if (!confirm("Are you sure you want to delete this ticket?")) return;
        fetch("/api/tickets/" + ticketId, { method: "DELETE" }).then(function() {
            document.getElementById("ticket-detail").innerHTML =
                '<div class="ticket-detail-empty"><p>Select a case to view details</p></div>';
            refreshTicketsList();
        });
    };

    window.resolveTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/resolve", { method: "PUT" }).then(function() {
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
            refreshTicketsList();
        });
    };

    window.closeTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/close", { method: "PUT" }).then(function() {
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
            refreshTicketsList();
        });
    };

    window.reopenTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/reopen", { method: "PUT" }).then(function() {
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
            refreshTicketsList();
        });
    };

    window.refreshTicketsList = function () {
        var active = document.querySelector(".tickets-tab.active");
        var status = active ? active.dataset.view : "all";
        htmx.ajax("GET", "/api/ui/tickets?status=" + status, "#tickets-list-body");
    };

    window.showItSmSection = function (section, btn) {
        document.querySelectorAll(".itsm-tab").forEach(function (t) {
            t.classList.remove("active");
        });
        if (btn) btn.classList.add("active");

        var container = document.getElementById("itsm-content");
        var list = document.getElementById("tickets-list-body");
        var detail = document.getElementById("ticket-detail");

        if (section === "tickets") {
            if (container) container.innerHTML = "";
            list.style.display = "";
            detail.style.display = "";
            refreshTicketsList();
            return;
        }

        list.style.display = "none";
        detail.style.display = "none";

        if (section === "problems" || section === "changes") {
            htmx.ajax("GET", "/api/ui/tickets?record_type=" + section, { target: "#itsm-content" });
        } else if (section === "cis") {
            htmx.ajax("GET", "/api/ui/tickets/cis", { target: "#itsm-content" });
        } else if (section === "kb") {
            htmx.ajax("GET", "/api/ui/tickets/kb", { target: "#itsm-content" });
        }
    };

    window.requestAiSuggestion = function (ticketId) {
        var el = document.getElementById("ai-suggestions");
        if (el) el.innerHTML = '<div style="padding:12px;color:#94a3b8">Analyzing...</div>';
        fetch("/api/tickets/" + ticketId + "/ai-suggest").then(function(r) { return r.json(); }).then(function(data) {
            if (el) {
                var suggestion = data.reasoning
                    || "Category: " + (data.suggested_category || "-")
                    + ", Priority: " + (data.suggested_priority || "-")
                    + " — No suggestion available.";
                el.innerHTML = '<div style="padding:12px;background:rgba(59,130,246,0.1);border-radius:8px;border:1px solid rgba(59,130,246,0.2)">'
                    + '<strong>AI Suggestion:</strong><br>' + suggestion
                    + '</div>';
            }
        });
    };

    document.addEventListener("keydown", function (e) {
        if (e.key === "Escape") closeTicketsModal();
    });

    if (window.i18n && window.i18n.translatePage) window.i18n.translatePage();
})();
