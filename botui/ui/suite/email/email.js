(function () {
  "use strict";

  function openAiCompose() {
    var modal = document.getElementById("ai-compose-modal");
    if (modal) {
      modal.style.display = "flex";
    }
  }

  function closeAiCompose() {
    var modal = document.getElementById("ai-compose-modal");
    if (modal) {
      modal.style.display = "none";
    }
    document.getElementById("ai-body").innerHTML = "";
    document.getElementById("ai-prompt").value = "";
    document.getElementById("ai-subject").value = "";
    document.getElementById("ai-to").value = "";
    document.getElementById("ai-cc").value = "";
    document.getElementById("ai-smart-replies").innerHTML = "";
  }

  function generateWithAI() {
    var prompt = document.getElementById("ai-prompt").value;
    if (!prompt || !prompt.trim()) {
      showNotification("Please describe what you want to say", "warning");
      return;
    }

    var tone = document.getElementById("ai-tone").value;
    var fullPrompt = "Write a " + tone + " email. " + prompt;
    var subject = document.getElementById("ai-subject").value;
    if (subject) {
      fullPrompt = "Subject: " + subject + ". " + fullPrompt;
    }

    var body = document.getElementById("ai-body");
    showProgress(true, "Generating email draft...");

    fetch("/api/ai/generate-reply", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        email_id: "00000000-0000-0000-0000-000000000000",
        context: fullPrompt
      })
    })
    .then(function (r) { return r.json(); })
    .then(function (res) {
      showProgress(false);
      if (res && res.suggestions && res.suggestions.length > 0) {
        body.innerHTML = res.suggestions[0];
        showSmartReplies(res.suggestions.slice(1));
      } else {
        body.innerHTML = "<p>[AI generation returned no result. Try again with a more detailed prompt.]</p>";
      }
    })
    .catch(function (err) {
      showProgress(false);
      body.innerHTML = "<p>[Failed to generate: " + err.message + "]</p>";
    });
  }

  function refineWithAI(instruction) {
    var body = document.getElementById("ai-body");
    var draft = body.innerHTML;
    if (!draft || draft.length < 10) {
      showNotification("Write or generate a draft first", "warning");
      return;
    }

    showProgress(true, "Refining draft...");

    fetch("/api/email/refine-draft", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        draft: draft,
        instruction: instruction
      })
    })
    .then(function (r) { return r.json(); })
    .then(function (res) {
      showProgress(false);
      if (res && res.draft) {
        body.innerHTML = res.draft;
        showNotification("Draft refined successfully", "success");
      }
    })
    .catch(function (err) {
      showProgress(false);
      showNotification("Refinement failed: " + err.message, "error");
    });
  }

  function showCustomRefine() {
    var modal = document.getElementById("custom-refine-modal");
    if (modal) {
      modal.style.display = "flex";
    }
  }

  function closeCustomRefine() {
    var modal = document.getElementById("custom-refine-modal");
    if (modal) {
      modal.style.display = "none";
    }
    document.getElementById("custom-refine-instruction").value = "";
  }

  function applyCustomRefine() {
    var instruction = document.getElementById("custom-refine-instruction").value;
    if (!instruction || !instruction.trim()) {
      showNotification("Please enter a refinement instruction", "warning");
      return;
    }
    closeCustomRefine();
    refineWithAI(instruction);
  }

  function showSmartReplies(suggestions) {
    var container = document.getElementById("ai-smart-replies");
    if (!container || !suggestions || suggestions.length === 0) return;
    var chips = suggestions.map(function (s) {
      return '<button class="smart-reply-chip" onclick="useSmartReply(\'' +
        s.replace(/'/g, "\\'") + '\')">' + s + '</button>';
    }).join("");
    container.innerHTML = '<span class="smart-reply-label">Alternatives:</span>' + chips;
  }

  window.useSmartReply = function (text) {
    var body = document.getElementById("ai-body");
    if (body) {
      body.innerHTML = text;
    }
  };

  function saveAiDraft() {
    var body = document.getElementById("ai-body");
    var draft = body ? body.innerHTML : "";
    if (!draft || draft.length < 5) {
      showNotification("Nothing to save. Generate or write a draft first.", "warning");
      return;
    }

    fetch("/api/email/draft/auto", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        account_id: "00000000-0000-0000-0000-000000000000",
        to: document.getElementById("ai-to").value || "unspecified",
        cc: document.getElementById("ai-cc").value || null,
        subject: document.getElementById("ai-subject").value || "(no subject)",
        body_html: draft
      })
    })
    .then(function (r) { return r.json(); })
    .then(function (res) {
      if (res && res.id) {
        showNotification("Draft saved (id: " + res.id + ")", "success");
      } else {
        showNotification("Draft saved", "success");
      }
    })
    .catch(function (err) {
      showNotification("Failed to save draft: " + err.message, "error");
    });
  }

  function sendAiEmail() {
    showNotification("Send via /api/email/send (configure account first)", "info");
    closeAiCompose();
  }

  function showProgress(visible, message) {
    var el = document.getElementById("ai-refine-progress");
    if (el) {
      el.style.display = visible ? "flex" : "none";
      if (message) {
        var span = el.querySelector("span");
        if (span) span.textContent = message;
      }
    }
  }

  function showConflictResolver() {
    var modal = document.getElementById("conflict-resolver-modal");
    if (modal) {
      modal.style.display = "flex";
    }
  }

  function closeConflictResolver() {
    var modal = document.getElementById("conflict-resolver-modal");
    if (modal) {
      modal.style.display = "none";
    }
    document.getElementById("cr-result").style.display = "none";
    document.getElementById("cr-result").innerHTML = "";
  }

  function resolveMeeting() {
    var subject = document.getElementById("cr-subject").value;
    var start = document.getElementById("cr-start").value;
    var end = document.getElementById("cr-end").value;
    var attendees = document.getElementById("cr-attendees").value;

    if (!subject || !start || !end) {
      showNotification("Please fill in subject, start, and end time", "warning");
      return;
    }

    var resultDiv = document.getElementById("cr-result");
    resultDiv.style.display = "block";
    resultDiv.innerHTML = '<div class="spinner"></div><p>Resolving conflicts...</p>';

    fetch("/api/email/meeting/resolve", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        subject: subject,
        proposed_start: new Date(start).toISOString(),
        proposed_end: new Date(end).toISOString(),
        organizer: "user@example.com",
        attendees: attendees ? attendees.split(",").map(function (a) { return a.trim(); }) : [],
        body: ""
      })
    })
    .then(function (r) { return r.json(); })
    .then(function (res) {
      if (res && res.has_conflicts !== undefined) {
        var html = "";
        if (res.has_conflicts) {
          html += '<div class="alert alert-warning"><strong>Conflicts detected:</strong>';
          if (res.conflicts && res.conflicts.length > 0) {
            html += '<ul>' + res.conflicts.map(function (c) {
              return "<li>" + c + "</li>";
            }).join("") + "</ul>";
          }
          html += "</div>";
          if (res.suggested_alternatives && res.suggested_alternatives.length > 0) {
            html += "<p><strong>Suggested alternatives:</strong></p><ul>";
            html += res.suggested_alternatives.map(function (a) {
              return "<li>" + a + "</li>";
            }).join("");
            html += "</ul>";
          }
        } else {
          html += '<div class="alert alert-success"><strong>No conflicts!</strong> The time slot is available.</div>';
        }
        if (res.reply_draft) {
          html += "<hr /><p><strong>Reply draft:</strong></p><pre style='white-space:pre-wrap;background:#1e293b;padding:0.75rem;border-radius:6px;'>" +
            escapeHtml(res.reply_draft) + "</pre>";
          html += '<button class="btn-primary" onclick="copyReplyDraft(\'' +
            res.reply_draft.replace(/'/g, "\\'") + '\')">Copy to draft</button>';
        }
        resultDiv.innerHTML = html;
      }
    })
    .catch(function (err) {
      resultDiv.innerHTML = '<div class="alert alert-error">Error: ' + err.message + "</div>";
    });
  }

  window.copyReplyDraft = function (draft) {
    var body = document.getElementById("ai-body");
    if (body) {
      body.innerHTML = draft.replace(/\n/g, "<br>");
      closeConflictResolver();
      openAiCompose();
      showNotification("Reply draft copied to compose", "success");
    }
  };

  function showAutoResponder() {
    var modal = document.getElementById("auto-responder-modal");
    if (modal) {
      modal.style.display = "flex";
    }
  }

  function closeAutoResponder() {
    var modal = document.getElementById("auto-responder-modal");
    if (modal) {
      modal.style.display = "none";
    }
  }

  function saveAutoResponder() {
    var status = document.getElementById("ar-status").value;
    var subject = document.getElementById("ar-subject").value;
    var message = document.getElementById("ar-message").value;

    fetch("/api/ui/email/auto-responder", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        status: status,
        subject: subject,
        message: message
      })
    })
    .then(function () {
      showNotification("Auto-responder settings saved", "success");
      closeAutoResponder();
    })
    .catch(function (err) {
      showNotification("Failed to save: " + err.message, "error");
    });
  }

  function showNotification(message, type) {
    if (typeof window.showNotification === "function") {
      window.showNotification(message, type);
    } else {
      console.log("[" + type + "] " + message);
    }
  }

  function escapeHtml(text) {
    var div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  function initEmail() {
    var navItems = document.querySelectorAll('.nav-item[data-folder]');
    navItems.forEach(function (item) {
      item.addEventListener("click", function () {
        navItems.forEach(function (i) { i.classList.remove("active"); });
        this.classList.add("active");
      });
    });
  }

  window.openAiCompose = openAiCompose;
  window.closeAiCompose = closeAiCompose;
  window.generateWithAI = generateWithAI;
  window.refineWithAI = refineWithAI;
  window.showCustomRefine = showCustomRefine;
  window.closeCustomRefine = closeCustomRefine;
  window.applyCustomRefine = applyCustomRefine;
  window.saveAiDraft = saveAiDraft;
  window.sendAiEmail = sendAiEmail;
  window.showConflictResolver = showConflictResolver;
  window.closeConflictResolver = closeConflictResolver;
  window.resolveMeeting = resolveMeeting;
  window.showAutoResponder = showAutoResponder;
  window.closeAutoResponder = closeAutoResponder;
  window.saveAutoResponder = saveAutoResponder;

  if (document.readyState === "loading") {
    (function(){ var __cb = initEmail; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    initEmail();
  }

  document.body.addEventListener("htmx:afterSwap", function (evt) {
    if (evt.detail.target && evt.detail.target.id === "main-content") {
      if (document.querySelector(".ai-compose-panel")) {
        initEmail();
      }
    }
  });
})();
