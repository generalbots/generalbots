if (window.GBAppLifecycle) GBAppLifecycle.begin("tickets");
(function () {
    document.querySelectorAll(".tickets-tab").forEach((tab) => {
        tab.addEventListener("click", function () {
            document.querySelectorAll(".tickets-tab").forEach((t) => t.classList.remove("active"));
            this.classList.add("active");
            const status = this.dataset.view;
            htmx.ajax("GET", "/api/ui/tickets?status=" + status, "#tickets-list-body");
        });
    });

    // Deep-link support: open the app contextualized to a ticket when the
    // desktop shell opened us via app://tickets?ticket_id=...
    const __dl = window.__gbAppParams__ || {};
    if (__dl.ticket_id) {
        const __target = __dl.ticket_id;
        let __tries = 0;
        const __try = function () {
            if (window.selectTicket) { window.selectTicket(__target); }
            if (__tries++ < 15) { setTimeout(__try, 500); }
        };
        setTimeout(__try, 400);
    }

    document.getElementById("tickets-new-btn").addEventListener("click", function () {
        openTicketsModal();
    });

    // #2954 — the Escape listener below is a document-level listener that
    // outlives this window. Once the app is closed #tickets-modal is gone, so
    // the lookup returns null; dereferencing .classList there threw an uncaught
    // TypeError. Fail closed when the modal is absent (already closed).
    window.openTicketsModal = function () {
        var modal = document.getElementById("tickets-modal");
        if (!modal) return;
        modal.classList.add("open");
    };

    window.closeTicketsModal = function () {
        var modal = document.getElementById("tickets-modal");
        if (!modal) return;
        modal.classList.remove("open");
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
        })
          .catch(function (fetchError) {
            window.gbLogError ? gbLogError(fetchError, "changeTicketStatus")
                                : console.error("changeTicketStatus", fetchError);
          });
    };

    window.assignTicket = function (ticketId) {
        // #2297 — themed modal input instead of the blocking native prompt.
        GBModal.prompt("Assign to (email or name):", function (assignee) {
            if (!assignee) return;
            fetch("/api/tickets/" + ticketId + "/assign", {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ assignee: assignee })
            }).then(function(r) {
                if (!r.ok) GBModal.alert("Could not assign ticket. Use an existing user email.");
                htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
            })
              .catch(function (fetchError) {
                window.gbLogError ? gbLogError(fetchError, "assignTicket")
                                    : console.error("assignTicket", fetchError);
              });
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
        })
          .catch(function (fetchError) {
            window.gbLogError ? gbLogError(fetchError, "addTicketNote")
                                : console.error("addTicketNote", fetchError);
          });
    };

    window.deleteTicket = function (ticketId) {
        GBModal.confirm("Are you sure you want to delete this ticket?", function () {
            fetch("/api/tickets/" + ticketId, { method: "DELETE" }).then(function() {
                document.getElementById("ticket-detail").innerHTML =
                    '<div class="ticket-detail-empty"><p>Select a case to view details</p></div>';
                refreshTicketsList();
            })
              .catch(function (fetchError) {
                window.gbLogError ? gbLogError(fetchError, "deleteTicket")
                                    : console.error("deleteTicket", fetchError);
              });
        });
    };

    window.resolveTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/resolve", { method: "PUT" }).then(function() {
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
            refreshTicketsList();
        })
          .catch(function (fetchError) {
            window.gbLogError ? gbLogError(fetchError, "resolveTicket")
                                : console.error("resolveTicket", fetchError);
          });
    };

    window.closeTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/close", { method: "PUT" }).then(function() {
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
            refreshTicketsList();
        })
          .catch(function (fetchError) {
            window.gbLogError ? gbLogError(fetchError, "closeTicket")
                                : console.error("closeTicket", fetchError);
          });
    };

    window.reopenTicket = function (ticketId) {
        fetch("/api/tickets/" + ticketId + "/reopen", { method: "PUT" }).then(function() {
            htmx.ajax("GET", "/api/ui/tickets/" + ticketId, "#ticket-detail");
            refreshTicketsList();
        })
          .catch(function (fetchError) {
            window.gbLogError ? gbLogError(fetchError, "reopenTicket")
                                : console.error("reopenTicket", fetchError);
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
        })
          .catch(function (fetchError) {
            window.gbLogError ? gbLogError(fetchError, "requestAiSuggestion")
                                : console.error("requestAiSuggestion", fetchError);
          });
    };

    document.addEventListener("keydown", function (e) {
        if (e.key === "Escape") closeTicketsModal();
    });

    if (window.i18n && window.i18n.translatePage) window.i18n.translatePage();
})();
