/* Chat render helpers: mentions, think tags and inline markdown.
 * Split out of chat-messages.js so every chat file stays under 450 lines. */
"use strict";

function renderMentionInMessage(content) {
  return content.replace(/@(\w+):([^\s]+)/g, function (match, type, name) {
    var entityType = EntityTypes[type.toLowerCase()];
    if (entityType) {
      return '<span class="mention-tag" data-type="' + type + '" data-name="' + escapeHtml(name) + '">' +
        '<span class="mention-icon">' + entityType.icon + "</span>" +
        '<span class="mention-text">@' + type + ":" + escapeHtml(name) + "</span>" +
        "</span>";
    }
    return match;
  });
}

// #1437 — stick the record id onto freshly-rendered person/contact mention
// tags so clicking them deep-links to the CRM record instead of only
// navigating to the app. The raw text only keeps `@type:name`, so the id is
// restored positionally from ChatState.selectedMentions (same kind, in order).
function enrichMentionIds(container) {
  if (!container || !Array.isArray(ChatState.selectedMentions)) return;
  var pool = ChatState.selectedMentions.filter(function (m) {
    return (m.kind === "contact" || m.kind === "person") && m.id;
  });
  container.querySelectorAll(".mention-tag[data-type]").forEach(function (tag) {
    var kind = (tag.getAttribute("data-type") || "").toLowerCase();
    if (kind !== "contact" && kind !== "person") return;
    for (var i = 0; i < pool.length; i++) {
      if (pool[i].kind === kind) {
        tag.setAttribute("data-id", String(pool[i].id));
        pool.splice(i, 1);
        return;
      }
    }
  });
}

function stripThinkTags(content) {
  // R6: Remove <think>...</think> but do NOT trim — preserves leading '<' in HTML chunks
  return content.replace(/<think>[\s\S]*?(?:<\/think>|$)/gi, "");
}

function stripReasoningPrefix(content) {
  // Nemotron and reasoning models output chain-of-thought before actual response
  // Strip everything before the first HTML tag (<div, <p, <h, etc.)
  var htmlStart = content.search(/<[a-zA-Z]/);
  if (htmlStart > 0) {
    return content.substring(htmlStart);
  }
  return content;
}

function stripSectorInfo(content) {
  var c = content.replace(/[-–—]\s*(Setor|Departamento|Cargo|Se[cç]ão|Enfermaria|Administra[çc][aã]o)\s*[^.<>]*/gi, "");
  return c.replace(/Ramal(\d)/g, "Ramal $1");
}

function stripMarkdownBlocks(content) {
  var cleanContent = stripThinkTags(content);
  cleanContent = stripReasoningPrefix(cleanContent);
  cleanContent = stripSectorInfo(cleanContent);
  // Unwrap markdown-language fences embedded in prose (before trailing-fence strip,
  // which would otherwise remove the closing ``` and leave the block unwrapped)
  cleanContent = cleanContent.replace(/(^|\n)```(?:markdown|md|text|plain)\s*\n([\s\S]*?)\n?```\s*(\n|$)/gi, "$1$2$3");
  // Unwrap whole-message code fences (HTML/XML/markdown/text) so tables and
  // markup render as HTML instead of displaying as raw markdown. The opening
  // fence must be unlabeled or one of those languages: the previous optional
  // group also matched ```python, leaving "python" as the first prose line and
  // turning every code answer into a paragraph with no copy button.
  cleanContent = cleanContent
    .replace(/^```(?:(?:html|xml|markdown|md|text|plain)\s*\n|\n)/i, "")
    .replace(/\n?```\s*$/i, "");
  var hasHtmlTags = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
  if (hasHtmlTags) {
    cleanContent = stripSectorInfo(cleanContent);
    return cleanContent;
  }
  var htmlMatch = cleanContent.match(/^```(?:html|xml|markdown|md|text|plain)?\s*\n([\s\S]+?)\n?```$/i);
  if (htmlMatch) return htmlMatch[1];
  return cleanContent;
}

// #1693 — a ```svg/```html block that the live preview renders into its
// sandboxed iframe must not also be painted inline: a 512px drawing inside the
// bubble dwarfs the answer and shows the same content twice. Strip the fenced
// block from the text handed to the markdown renderer; the iframe is the
// single render. An unterminated fence is stripped too, so a drawing that is
// still streaming does not flash its raw source before the frame catches up.
function stripLiveArtifactBlock(content) {
  var text = String(content || "");
  // #2019 — the artifact module knows every detection variant (escaped markup,
  // ~~~ fences, language-less fences, XML prolog, prose-wrapped SVG, JSON
  // envelope); strip exactly what it detected so the bubble never keeps a copy.
  if (window.GBLiveArtifact && typeof window.GBLiveArtifact.strip === "function") {
    return window.GBLiveArtifact.strip(text);
  }
  // A message that is only a bare <svg> is owned by the preview too.
  if (/^\s*<svg[\s>][\s\S]*<\/svg>\s*$/i.test(text)) return "";
  // #2005 — a bare <svg> is previewed while it streams (before </svg>), so its
  // source must stay out of the bubble for that whole window as well.
  if (/^\s*<svg[\s>]/i.test(text) && text.indexOf("```") < 0) return "";
  // #2019 — the same reply may arrive HTML-escaped (`&lt;svg …&gt;`); the live
  // preview unescapes it, so the escaped source is the preview's too.
  if (/^\s*&lt;\s*svg[\s>]/i.test(text) && text.indexOf("```") < 0) return "";
  return text
    .replace(/```(?:svg|html|htm|xml)\s*\n[\s\S]*?```\s*/gi, "")
    .replace(/```(?:svg|html|htm|xml)\s*\n[\s\S]*$/i, "")
    .trim();
}

function reexecuteScripts(container) {
  var scripts = container.querySelectorAll('script');
  for (var i = 0; i < scripts.length; i++) {
    try {
      var text = scripts[i].textContent || '';
      if (text.trim()) {
        var fn = new Function(text);
        fn();
      }
    } catch (e) {
      console.warn('Script exec error:', e);
    }
  }
}

function renderThinkingSection(reasoning) {
  if (!reasoning || reasoning.trim() === "") return "";
  var safeReasoning = escapeHtml(reasoning);
  return '<details class="thinking-section">' +
    '<summary><span class="thinking-summary-icon">\u{1F9E0}</span> Thinking</summary>' +
    '<div class="thinking-content">' + safeReasoning + '</div>' +
    '</details>';
}

/* v3-indicator-fixed */

// Raw HTML blocks (the model often answers with <ul>/<p>) are passed through
// by marked verbatim, so markdown links and bold inside them stay literal.
// Convert the inline syntax left in those blocks so citations are clickable.
function renderInlineMarkdown(html) {
  if (!html) return html;
  var out = html.replace(
    /\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)/g,
    '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>'
  );
  out = out.replace(/\*\*([^*<>]+)\*\*/g, "<strong>$1</strong>");
  return out;
}

function convertNumberedLists(html) {
  var lines = html.split('\n');
  var result = [];
  var group = null;

  for (var i = 0; i < lines.length; i++) {
    var trimmed = lines[i].trim();
    var m = trimmed.match(/^(\d+)\.\s+([\s\S]*)/);

    if (m) {
      if (!group) group = [];
      group.push({ num: parseInt(m[1], 10), content: m[2] });
    } else {
      if (group) {
        if (group.length >= 2) flushGroup();
        else result.push(group.map(function(g) { return g.num + '. ' + g.content; }).join('\n'));
        group = null;
      }
      result.push(lines[i]);
    }
  }
  if (group) {
    if (group.length >= 2) flushGroup();
    else result.push(group.map(function(g) { return g.num + '. ' + g.content; }).join('\n'));
  }

  function flushGroup() {
    var startAttr = group[0].num !== 1 ? ' start="' + group[0].num + '"' : '';
    var lis = group.map(function(it) { return '<li>' + it.content.trim() + '</li>'; }).join('');
    result.push('<ol' + startAttr + '>' + lis + '</ol>');
  }

  return result.join('\n');
}

// Parses the query string of an app:// deep link into a plain object.
function parseDeepLinkParams(query) {
  var params = {};
  if (!query) return params;
  query.split("&").forEach(function (pair) {
    var kv = pair.split("=");
    if (kv[0]) params[decodeURIComponent(kv[0])] = decodeURIComponent(kv[1] || "");
  });
  return params;
}

// #2226 — the app catalog (/api/apps/catalog, public) is the authority for
// which apps exist and which deep-link keys each declares. It is kept here
// because window.APPS_REGISTRY only carries launcher-visible apps (preview
// apps are filtered out unless Preview mode is on).
var deepLinkCatalog = null;
(function loadDeepLinkCatalog() {
  if (typeof window.fetch !== "function") return;
  window.fetch("/api/apps/catalog")
    .then(function (response) { return response.ok ? response.json() : null; })
    .then(function (data) {
      if (!data || !Array.isArray(data.apps)) return;
      var catalog = {};
      data.apps.forEach(function (app) {
        if (!app || !app.id) return;
        catalog[app.id] = (Array.isArray(app.deep_link_params) ? app.deep_link_params : [])
          .map(function (param) { return param && param.key; })
          .filter(Boolean);
      });
      deepLinkCatalog = catalog;
    })
    .catch(function () { /* keep the registry fallback */ });
})();

// #2226 — a deep link is only real when the app is in the catalog AND carries
// at least one of its declared record keys, so a link whose keys are all
// invented (e.g. app://vision?id=<made up>, app://docs?made_up=1) is treated
// as a hallucination and stays plain text.
function deepLinkTargetIsReal(appId, params) {
  params = params || {};
  var hasParams = Object.keys(params).length > 0;
  var declared = null;
  if (deepLinkCatalog) {
    if (!Object.prototype.hasOwnProperty.call(deepLinkCatalog, appId)) return false;
    declared = deepLinkCatalog[appId];
  } else {
    var registry = window.APPS_REGISTRY;
    var app = null;
    if (Array.isArray(registry) && registry.length) {
      registry.some(function (candidate) {
        if (candidate && candidate.id === appId) { app = candidate; return true; }
        return false;
      });
      if (!app) return false;
      declared = Array.isArray(app.deep_link_params) ? app.deep_link_params
        .map(function (param) { return param && param.key; }).filter(Boolean) : null;
    } else if (window.APP_CAPABILITIES && !window.APP_CAPABILITIES[appId]) {
      return false;
    }
  }
  if (!declared) return true; // no catalog data to validate against: keep the link
  // Bare app://appId (no params) is a valid "open the app" intent for any
  // known app; only param-bearing links need a declared record key (#2240).
  if (!hasParams) return true;
  var keys = declared.map(function (param) { return typeof param === "string" ? param : param && param.key; }).filter(Boolean);
  if (!keys.length) return true; // app exists but declares no record keys: treat as open-app
  return keys.some(function (key) { return params[key] !== undefined && params[key] !== ""; });
}

// Converts deep links of the form [Label](app://appId?k=v&k2=v2) in bot
// replies into clickable buttons that open the related app window
// contextualized in the desktop shell. Disabled outside the web desktop
// (no window manager) — there the link stays as plain text.
function renderDeepLinks(html) {
  if (!html || !window.WindowManager || !window.openDeepLink) return html;
  // Protect code blocks: app:// inside <pre>/<code> stays literal (#2240).
  var codeBlocks = [];
  html = html.replace(/(<pre[\s\S]*?<\/pre>|<code[\s\S]*?<\/code>)/gi, function (m) {
    codeBlocks.push(m);
    return "\u0000CODE" + (codeBlocks.length - 1) + "\u0000";
  });
  // marked converts markdown links to <a href="app://...">label</a>;
  // match that anchor form (and any app:// href).
  var pattern = /<a\s+[^>]*href="app:\/\/[^"]*"[^>]*>([\s\S]*?)<\/a>/gi;
  html = html.replace(pattern, function (m, label) {
    var hrefMatch = m.match(/href="app:\/\/([^"]+)"/i);
    if (!hrefMatch) return m;
    var target = hrefMatch[1];
    var appId = target.split("?")[0];
    var params = parseDeepLinkParams(target.indexOf("?") !== -1 ? target.split("?")[1] : "");
    // #2226 / #2235 F7 — unknown app or invented keys: render the label as
    // plain text, never as a dead app:// anchor or a button.
    if (!deepLinkTargetIsReal(appId, params)) return label;
    var escapedLabel = String(label).replace(/"/g, "&quot;");
    var json = JSON.stringify(params).replace(/"/g, "&quot;");
    return '<button type="button" class="app-deeplink-btn" ' +
      'title="Open ' + appId + '" ' +
      'data-gb-app="' + appId + '" data-gb-params="' + json + '" ' +
      'onclick="window.openDeepLink(this.dataset.gbApp, JSON.parse(this.dataset.gbParams))">' +
      escapedLabel + "</button>";
  });
  // Also convert bare app://appId (with or without query params) URLs the LLM
  // may emit in prose into the same button (fallback when it does not use
  // markdown links). Templated app://appId/{...} paths stay literal.
  var barePattern = /(^|[\s(>])(app:\/\/[A-Za-z0-9\-_.]+(?:\?[A-Za-z0-9\-_.=&\{\}]+)?)/g;
  html = html.replace(barePattern, function (m, pre, target) {
    var stripped = target.replace(/^app:\/\//i, "");
    // templated path segments are not valid app ids — leave as-is
    if (stripped.indexOf("/") !== -1) return m;
    var appId = stripped.split("?")[0];
    var params = parseDeepLinkParams(stripped.indexOf("?") !== -1 ? stripped.split("?")[1] : "");
    // #2226 — a bare URL already reads as text; only a real target becomes a button.
    if (!deepLinkTargetIsReal(appId, params)) return m;
    var json = JSON.stringify(params).replace(/"/g, "&quot;");
    return pre + '<button type="button" class="app-deeplink-btn" ' +
      'title="Open ' + appId + '" ' +
      'data-gb-app="' + appId + '" data-gb-params="' + json + '" ' +
      'onclick="window.openDeepLink(this.dataset.gbApp, JSON.parse(this.dataset.gbParams))">' +
      'Open ' + appId + "</button>";
  });
  // Restore code blocks.
  html = html.replace(/\u0000CODE(\d+)\u0000/g, function (m, idx) {
    return codeBlocks[Number(idx)] || m;
  });
  return html;
}
