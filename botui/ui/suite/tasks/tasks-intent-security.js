/* =============================================================================
   TASKS — QUICK-INTENT RESULT SECURITY (#2838)
   Shared, guarded helper for escaping result text and validating app URLs.
   Loaded before tasks-state.js / tasks-scoped-state.js.
   ============================================================================= */

// =============================================================================
// QUICK-INTENT RESULT SECURITY
// =============================================================================
// Guarded singleton so both the full and scoped tasks shells can load this file
// more than once (HTMX swaps) without redeclaring globals.

if (typeof TasksIntentSecurity === "undefined") {
  var TasksIntentSecurity = {
    // Escapes text before it is placed inside an innerHTML template.
    escapeHtml: function (value) {
      if (value === null || value === undefined) return "";
      return String(value)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
    },

    // Returns a normalized absolute http(s) URL, or null for any other scheme.
    // Using URL() also neutralizes obfuscated schemes (e.g. "java\tscript:").
    safeHttpUrl: function (value) {
      if (value === null || value === undefined) return null;
      const raw = String(value).trim();
      if (!raw) return null;
      let parsed;
      try {
        const base =
          typeof window !== "undefined" &&
          window.location &&
          window.location.href
            ? window.location.href
            : undefined;
        parsed = base ? new URL(raw, base) : new URL(raw);
      } catch (e) {
        return null;
      }
      if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
        return null;
      }
      return parsed.href;
    },

    // i18n lookup with an English fallback (never hardcode user-facing strings).
    t: function (key, fallback) {
      return typeof window !== "undefined" && window.gbT
        ? window.gbT(key, fallback)
        : fallback;
    },

    // Renders the completed/success or error quick-intent card. All response
    // text is escaped; the link href is validated and set via setAttribute.
    renderResult: function (resultDiv, response) {
      if (!resultDiv) return;
      const S = TasksIntentSecurity;
      const card = document.createElement("div");
      card.className = "result-card";
      const message = document.createElement("div");

      if (response && response.success) {
        message.className = "result-message result-success";
        message.innerHTML =
          "✓ " +
          S.escapeHtml(
            response.message || S.t("tasks-intent-done", "Done!"),
          );
        card.appendChild(message);

        if (response.app_url) {
          const safeUrl = S.safeHttpUrl(response.app_url);
          if (safeUrl) {
            const link = document.createElement("a");
            link.className = "result-link";
            link.setAttribute("href", safeUrl);
            link.setAttribute("target", "_blank");
            link.setAttribute("rel", "noopener noreferrer");
            link.textContent = S.t("tasks-intent-open-app", "Open App →");
            card.appendChild(link);
          }
        }

        if (response.task_id) {
          const meta = document.createElement("div");
          meta.setAttribute(
            "style",
            "margin-top:8px;color:#666;font-size:13px;",
          );
          meta.innerHTML =
            S.t("tasks-intent-task-id", "Task ID:") +
            " " +
            S.escapeHtml(response.task_id);
          card.appendChild(meta);
        }
      } else {
        message.className = "result-message result-error";
        message.innerHTML =
          "✗ " +
          S.escapeHtml(
            (response && (response.error || response.message)) ||
              S.t("tasks-intent-error", "Something went wrong"),
          );
        card.appendChild(message);
      }

      resultDiv.innerHTML = "";
      resultDiv.appendChild(card);
    },

    // Console-invocable self-test: TasksIntentSecurity.selfTest()
    // Verifies escaping, scheme rejection and DOM guard-clean rendering.
    selfTest: function () {
      const S = TasksIntentSecurity;
      const results = [];
      const check = function (name, pass, detail) {
        results.push({ name: name, pass: !!pass, detail: detail || "" });
      };

      const payload = '<img src=x onerror="window.__gbXssFired=true">';
      const escaped = S.escapeHtml(payload);
      check(
        "escapeHtml strips markup",
        escaped.indexOf("<") === -1 && escaped.indexOf(">") === -1,
        escaped,
      );
      check("escapeHtml strips quotes", escaped.indexOf('"') === -1, escaped);
      check("reject javascript:", S.safeHttpUrl("javascript:alert(1)") === null);
      check(
        "reject obfuscated javascript:",
        S.safeHttpUrl("java\tscript:alert(1)") === null,
      );
      check(
        "reject data:",
        S.safeHttpUrl("data:text/html,<script>alert(1)</script>") === null,
      );
      check("allow https", !!S.safeHttpUrl("https://example.com/app"));
      const attr = S.safeHttpUrl(
        'https://example.com/app?a=1" onmouseover="alert(1)',
      );
      check("attr payload encoded", !!attr && attr.indexOf('"') === -1, attr);

      if (typeof document !== "undefined") {
        const host = document.createElement("div");
        S.renderResult(host, {
          success: true,
          message: payload,
          app_url: 'javascript:alert(1)" onmouseover="alert(1)',
          task_id: payload,
        });
        check("no img element rendered", host.querySelector("img") === null);
        const anchor = host.querySelector("a");
        check(
          "no javascript href rendered",
          !anchor || anchor.getAttribute("href").indexOf("javascript:") !== 0,
          anchor ? anchor.getAttribute("href") : "no link",
        );
      }
      return results;
    },
  };
}
