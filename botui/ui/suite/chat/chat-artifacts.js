/* #1636 — artifact detection, document rendering and Save to Drive. */
"use strict";

// #1636 — artifact detection and document rendering.
//
// The old rule only offered "Open as document" for a fenced code block of 300+
// characters, so a long prose answer or a markdown report — the most common
// shape of a generated document — had no entry point. The detector below
// covers prose (by length and paragraph structure), markdown reports
// (headings/tables) and standalone HTML pages, and keeps the code path.
window.GBArtifacts = window.GBArtifacts || (function () {
  "use strict";

  function wordCount(text) {
    var matches = String(text || "").trim().match(/\S+/g);
    return matches ? matches.length : 0;
  }

  function detect(content) {
    if (!content) return null;
    var codeBlocks = content.match(/```[\s\S]{300,}?```/g);
    if (codeBlocks && codeBlocks.length > 0) return { kind: "code" };
    if (/<html[\s>]|<body[\s>]|<article[\s>]/i.test(content)) return { kind: "html" };
    var prose = content.replace(/```[\s\S]*?```/g, " ");
    var words = wordCount(prose);
    var headings = (prose.match(/^#{1,6}\s+\S/gm) || []).length;
    var tableRows = (prose.match(/^\s*\|.*\|\s*$/gm) || []).length;
    var paragraphs = prose.split(/\n\s*\n/).filter(function (p) {
      return p.trim().length > 80;
    }).length;
    // 700 words is well above the old threshold's equivalent text amount; 250
    // keeps a real report actionable without offering the panel for every
    // two-paragraph reply. Structure lowers the bar because a titled report is
    // clearly a document even when it is short.
    if (
      words >= 250 ||
      (words >= 120 && (headings > 0 || tableRows > 2)) ||
      paragraphs >= 3
    ) {
      return { kind: "markdown" };
    }
    return null;
  }

  function titleFor(content, kind) {
    var source = String(content || "");
    // Drawn artifacts have no markdown heading: the old fallback used the
    // first line, so an SVG panel was titled "<svg xmlns=\"http://…". Use the
    // document's <title> when present, otherwise a human label (#1693).
    var docTitle = source.match(/<title[^>]*>([\s\S]*?)<\/title>/i);
    if (docTitle && docTitle[1].trim()) return docTitle[1].trim().slice(0, 80);
    if (/```svg/i.test(source) || /^\s*<svg[\s>]/i.test(source)) return "SVG drawing";
    if (/```html/i.test(source) || /<!doctype html|<html[\s>]/i.test(source)) return "HTML document";
    var heading = source.match(/^\s{0,3}#{1,6}\s+(.+)$/m);
    if (heading) return heading[1].replace(/[*_`]/g, "").trim().slice(0, 80);
    var lines = String(content || "").split(/\n/);
    for (var i = 0; i < lines.length; i++) {
      var line = lines[i].replace(/^[*_`>\s-]+|[*_`\s]+$/g, "").trim();
      if (line.length >= 8) return line.slice(0, 80);
    }
    return kind === "code" ? "Code document" : "Document";
  }

  function slug(text) {
    var value = String(text || "")
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 60);
    return value || "artifact-" + Date.now();
  }

  function escapeHtml(text) {
    return String(text || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  // Renders markdown/HTML into the panel without executing anything the model
  // produced: no scripts, frames, objects, event handlers or javascript: URLs.
  function safeHtml(html) {
    var tpl = document.createElement("template");
    tpl.innerHTML = String(html || "");
    var blocked = "script,style,iframe,object,embed,link,meta,base,form,input,button";
    Array.prototype.forEach.call(
      tpl.content.querySelectorAll(blocked),
      function (node) {
        node.remove();
      },
    );
    Array.prototype.forEach.call(tpl.content.querySelectorAll("*"), function (el) {
      Array.prototype.slice.call(el.attributes).forEach(function (attr) {
        var name = attr.name.toLowerCase();
        if (name.indexOf("on") === 0) {
          el.removeAttribute(attr.name);
        } else if (
          (name === "href" || name === "src") &&
          /^\s*(javascript|data:text\/html)/i.test(attr.value)
        ) {
          el.removeAttribute(attr.name);
        }
      });
    });
    return tpl.innerHTML;
  }

  function renderMarkdown(text) {
    if (typeof marked !== "undefined" && marked.parse) {
      try {
        return safeHtml(marked.parse(text));
      } catch (e) {
        /* fall through to plain text */
      }
    }
    return "<p>" + escapeHtml(text).replace(/\n/g, "<br>") + "</p>";
  }

  return {
    detect: detect,
    titleFor: titleFor,
    slug: slug,
    escapeHtml: escapeHtml,
    safeHtml: safeHtml,
    renderMarkdown: renderMarkdown,
    wordCount: wordCount,
  };
})();

function openArtifactPanel(content, sourceEl) {
  var existing = document.getElementById("gb-artifact-panel");
  if (existing) existing.remove();

  var detected = window.GBArtifacts.detect(content) || { kind: "markdown" };
  var kind = detected.kind;
  var codeMatch = content.match(/```(\w*)\n([\s\S]*?)```/);
  var lang = codeMatch ? codeMatch[1] : "";
  var body = codeMatch ? codeMatch[2] : content;
  var title = window.GBArtifacts.titleFor(content, kind);
  if (codeMatch && lang === "html") kind = "html";
  if (kind === "code" && /^\s*<(!doctype|html)/i.test(body)) kind = "html";
  // An SVG drawing is the one artifact that has an editable destination:
  // Canvas imports it into real shapes instead of showing read-only code
  // (#1708).
  var isSvg = (codeMatch && lang === "svg") || /^\s*<svg[\s>]/i.test(body);

  var panel = document.createElement("div");
  panel.id = "gb-artifact-panel";
  panel.className = "gb-artifact-panel";
  var preview = "";
  if (kind === "html") {
    preview = '<iframe class="gb-artifact-iframe" sandbox="allow-scripts"></iframe>';
  } else if (kind === "markdown") {
    preview =
      '<div class="gb-artifact-doc">' +
      window.GBArtifacts.renderMarkdown(body) +
      "</div>";
  } else {
    var highlighted =
      window.ChatCodeUX && window.ChatCodeUX.highlight
        ? window.ChatCodeUX.highlight(body, lang)
        : window.GBArtifacts.escapeHtml(body);
    preview = '<pre class="gb-artifact-code"><code>' + highlighted + "</code></pre>";
  }
  panel.innerHTML =
    '<div class="gb-artifact-header">' +
    '<span class="gb-artifact-title">' +
    window.GBArtifacts.escapeHtml(title) +
    "</span>" +
    '<div class="gb-artifact-actions">' +
    '<input class="gb-artifact-folder" type="text" placeholder="Folder (optional)" aria-label="Drive folder" />' +
    '<button class="gb-artifact-copy" title="Copy">Copy</button>' +
    (isSvg ? '<button class="gb-artifact-open-canvas" title="Open in Canvas">Open in Canvas</button>' : "") +
    '<button class="gb-artifact-save" title="Save to Drive">Save</button>' +
    '<button class="gb-artifact-close" title="Close">Close</button>' +
    "</div></div>" +
    '<div class="gb-artifact-sub">' +
    (kind === "markdown" ? "Markdown document" : kind === "html" ? "HTML page" : "Code") +
    " · " +
    window.GBArtifacts.wordCount(body) +
    " words</div>" +
    '<div class="gb-artifact-body">' +
    preview +
    "</div>";

  document.body.appendChild(panel);

  if (kind === "html") {
    var iframe = panel.querySelector("iframe");
    if (iframe) iframe.srcdoc = body;
  }
  panel.querySelector(".gb-artifact-close").addEventListener("click", function () {
    panel.remove();
  });
  panel.querySelector(".gb-artifact-copy").addEventListener("click", function () {
    if (navigator.clipboard) navigator.clipboard.writeText(body);
  });
  var openCanvasBtn = panel.querySelector(".gb-artifact-open-canvas");
  if (openCanvasBtn) {
    openCanvasBtn.addEventListener("click", function () {
      var folderInput = panel.querySelector(".gb-artifact-folder");
      var folder = folderInput ? folderInput.value.trim().replace(/^\/+|\/+$/g, "") : "";
      var fileName = window.GBArtifacts.slug(title) + ".svg";
      var path = folder ? folder + "/" + fileName : fileName;
      var token = "";
      try {
        token =
          (window.getGBAccessToken && window.getGBAccessToken()) ||
          localStorage.getItem("gb-access-token") ||
          sessionStorage.getItem("gb-access-token") ||
          "";
      } catch (e) {}
      fetch("/api/files/write", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(token ? { Authorization: "Bearer " + token } : {}),
        },
        body: JSON.stringify({
          path: path,
          content: btoa(unescape(encodeURIComponent(body))),
          scope: "user",
        }),
      })
        .then(function (res) {
          if (!res.ok) throw new Error("HTTP " + res.status);
          if (window.GBToasts) {
            GBToasts.show("Canvas", "Opening " + fileName + " in Canvas", "success");
          }
          if (typeof window.openDeepLink === "function") {
            window.openDeepLink("canvas", { file: path, scope: "user" });
          } else if (window.WindowManager && window.WindowManager.openDeepLink) {
            window.WindowManager.openDeepLink("canvas", { file: path, scope: "user" });
          }
        })
        .catch(function (err) {
          if (window.GBToasts) {
            GBToasts.show("Canvas", "Could not open in Canvas: " + err.message, "error");
          }
        });
    });
  }
  panel.querySelector(".gb-artifact-save").addEventListener("click", function () {
    var ext = kind === "markdown" ? "md" : kind === "html" ? "html" : lang || "txt";
    var fileName = window.GBArtifacts.slug(title) + "." + ext;
    var folderInput = panel.querySelector(".gb-artifact-folder");
    var folder = folderInput ? folderInput.value.trim().replace(/^\/+|\/+$/g, "") : "";
    var path = folder ? folder + "/" + fileName : fileName;
    var token = "";
    try {
      token =
        (window.getGBAccessToken && window.getGBAccessToken()) ||
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        "";
    } catch (e) {}
    // #1548: save directly, without depending on the Drive app having been
    // opened first (DriveModule/apiRequest were only defined after that).
    fetch("/api/files/write", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: "Bearer " + token } : {}),
      },
      body: JSON.stringify({
        path: path,
        content: btoa(unescape(encodeURIComponent(body))),
        scope: "user",
      }),
    })
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        if (window.GBToasts) {
          GBToasts.show("Saved", "File saved to Drive as " + path, "success");
        }
        // #1636 — keep a link/timestamp back to the conversation on the message
        // that produced the document, so the saved file is not orphaned.
        if (sourceEl) {
          var note = document.createElement("a");
          note.className = "gb-artifact-saved";
          note.href = "#";
          note.textContent =
            "Saved to Drive: " + path + " · " + new Date().toLocaleTimeString();
          note.addEventListener("click", function (e) {
            e.preventDefault();
            if (window.openDeepLink) window.openDeepLink("drive", { path: path });
          });
          sourceEl.appendChild(note);
        }
      })
      .catch(function (err) {
        if (window.GBToasts) {
          GBToasts.show("Save failed", err.message || "Could not save to Drive", "error");
        }
      });
  });
}
