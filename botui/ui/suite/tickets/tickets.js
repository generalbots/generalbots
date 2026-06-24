(function () {
    document.querySelectorAll(".tickets-tab").forEach((tab) => {
        tab.addEventListener("click", function () {
            document.querySelectorAll(".tickets-tab").forEach((t) => t.classList.remove("active"));
            this.classList.add("active");
            const status = this.dataset.view;
            htmx.ajax("GET", "/api/tickets?status=" + status, "#tickets-list-body");
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
        htmx.ajax("GET", "/api/tickets/" + ticketId, "#ticket-detail");
    };

    window.changeTicketStatus = function (ticketId, newStatus) {
        fetch("/api/tickets/" + ticketId, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ status: newStatus })
        }).then(function() {
            htmx.ajax("GET", "/api/tickets/" + ticketId, "#ticket-detail");
        });
    };

    window.assignTicket = function (ticketId) {
        var assignee = prompt("Assign to (email or name):");
        if (!assignee) return;
        fetch("/api/tickets/" + ticketId + "/assign", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ assignee: assignee })
        }).then(function() {
            htmx.ajax("GET", "/api/tickets/" + ticketId, "#ticket-detail");
        });
    };

    window.addTicketNote = function (ticketId) {
        var note = document.getElementById("note-input-" + ticketId);
        if (!note || !note.value.trim()) return;
        fetch("/api/tickets/" + ticketId + "/activities", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ type: "note", content: note.value.trim() })
        }).then(function() {
            note.value = "";
            htmx.ajax("GET", "/api/tickets/" + ticketId, "#ticket-detail");
        });
    };

    window.deleteTicket = function (ticketId) {
        if (!confirm("Are you sure you want to delete this ticket?")) return;
        fetch("/api/tickets/" + ticketId, { method: "DELETE" }).then(function() {
            document.getElementById("ticket-detail").innerHTML =
                '<div class="ticket-detail-empty"><p>Select a case to view details</p></div>';
        });
    };

    window.resolveTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/resolve", { method: "POST" }).then(function() {
            htmx.ajax("GET", "/api/tickets/" + ticketId, "#ticket-detail");
        });
    };

    window.closeTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/close", { method: "POST" }).then(function() {
            htmx.ajax("GET", "/api/tickets/" + ticketId, "#ticket-detail");
        });
    };

    window.reopenTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/reopen", { method: "POST" }).then(function() {
            htmx.ajax("GET", "/api/tickets/" + ticketId, "#ticket-detail");
        });
    };

    window.requestAiSuggestion = function (ticketId) {
        var el = document.getElementById("ai-suggestions");
        if (el) el.innerHTML = '<div style="padding:12px;color:#94a3b8">Analyzing...</div>';
        fetch("/api/tickets/" + ticketId + "/ai-suggest").then(function(r) { return r.json(); }).then(function(data) {
            if (el) {
                el.innerHTML = '<div style="padding:12px;background:rgba(59,130,246,0.1);border-radius:8px;border:1px solid rgba(59,130,246,0.2)">'
                    + '<strong>AI Suggestion:</strong><br>' + (data.suggestion || 'No suggestion available.')
                    + '</div>';
            }
        });
    };

    document.addEventListener("keydown", function (e) {
        if (e.key === "Escape") closeTicketsModal();
    });

    if (window.i18n && window.i18n.translatePage) window.i18n.translatePage();
})();
