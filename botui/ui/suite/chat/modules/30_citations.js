"use strict";

/**
 * Citation site-chips renderer (#1175-fe).
 * Exposes window.GBRenderCitations(msgEl, msgData): when msgData.citations
 * (array of {title,url,host?}) is present, inserts a div.source-chips block
 * before the message text and linkifies [n] markers in the rendered text.
 * Hooks addMessage via the assignment-preservation pattern; the bot frame is
 * captured from the "gb-ws-frame" event dispatched by the tabs module.
 */

(function () {
  var lastBotFrame = null;

  document.addEventListener("gb-ws-frame", function (e) {
    var d = e.detail || {};
    if (d.message_type === 2 && Array.isArray(d.citations) && d.citations.length) {
      lastBotFrame = d;
    }
  });

  function hostOf(url) {
    try { return new URL(url).hostname.replace(/^www\./, ""); }
    catch (e) { return String(url).replace(/^https?:\/\//i, "").split("/")[0] || "source"; }
  }

  function buildChip(citation) {
    var url = citation.url || citation.href || "";
    if (!url) return null;
    var host = citation.host || hostOf(url);
    var title = citation.title || host;
    var chip = document.createElement("button");
    chip.type = "button";
    chip.className = "source-chip";
    chip.setAttribute("data-url", url);
    chip.title = title + "\n" + url;
    var avatar = document.createElement("span");
    avatar.className = "source-chip-avatar";
    avatar.textContent = (host.charAt(0) || "S").toUpperCase();
    var label = document.createElement("span");
    label.className = "source-chip-host";
    label.textContent = host;
    chip.appendChild(avatar);
    chip.appendChild(label);
    chip.addEventListener("click", function () {
      window.open(url, "_blank", "noopener");
    });
    return chip;
  }

  function openCitation(citations, n) {
    var c = citations && citations[n - 1];
    if (c && (c.url || c.href)) {
      window.open(c.url || c.href, "_blank", "noopener");
    }
  }

  function linkifyMarkers(container, citations) {
    var walker = document.createTreeWalker(
      container,
      NodeFilter.SHOW_TEXT,
      {
        acceptNode: function (node) {
          if (!node.nodeValue || node.nodeValue.indexOf("[") === -1) {
            return NodeFilter.FILTER_REJECT;
          }
          var p = node.parentNode;
          while (p && p !== container) {
            var tag = p.nodeName;
            if (tag === "SCRIPT" || tag === "STYLE" || tag === "SUP" ||
                tag === "BUTTON" || tag === "A") {
              return NodeFilter.FILTER_REJECT;
            }
            p = p.parentNode;
          }
          return /\[\d{1,2}\]/.test(node.nodeValue)
            ? NodeFilter.FILTER_ACCEPT
            : NodeFilter.FILTER_REJECT;
        },
      }
    );
    var hits = [];
    while (walker.nextNode()) hits.push(walker.currentNode);
    hits.forEach(function (node) {
      var frag = document.createDocumentFragment();
      var rest = node.nodeValue;
      var re = /\[(\d{1,2})\]/g;
      var m;
      var lastIdx = 0;
      while ((m = re.exec(rest)) !== null) {
        var n = parseInt(m[1], 10);
        if (n < 1 || n > citations.length) continue;
        if (m.index > lastIdx) {
          frag.appendChild(document.createTextNode(rest.slice(lastIdx, m.index)));
        }
        var sup = document.createElement("sup");
        sup.className = "gb-cite-ref";
        sup.setAttribute("data-cite", String(n));
        sup.textContent = "[" + n + "]";
        sup.addEventListener("click", function (idx) {
          return function () { openCitation(citations, idx); };
        }(n));
        frag.appendChild(sup);
        lastIdx = m.index + m[0].length;
      }
      if (lastIdx < rest.length) {
        frag.appendChild(document.createTextNode(rest.slice(lastIdx)));
      }
      if (lastIdx > 0) node.parentNode.replaceChild(frag, node);
    });
  }

  /**
   * Renders citation chips (and superscript markers) for one bot message.
   * Idempotent per message element.
   */
  window.GBRenderCitations = function (msgEl, msgData) {
    if (!msgEl || !msgData || !Array.isArray(msgData.citations) ||
        !msgData.citations.length) {
      return;
    }
    if (msgEl.getAttribute("data-gb-citations") === "done") return;
    var content = msgEl.querySelector(".message-content") || msgEl;
    if (content.querySelector(".source-chips")) return;

    var chips = document.createElement("div");
    chips.className = "source-chips";
    var added = false;
    msgData.citations.forEach(function (c) {
      var chip = buildChip(c || {});
      if (chip) { chips.appendChild(chip); added = true; }
    });
    if (!added) return;

    var thinking = content.querySelector(":scope > details.thinking-section");
    if (thinking && thinking.nextSibling) {
      content.insertBefore(chips, thinking.nextSibling);
    } else {
      content.insertBefore(chips, content.firstChild);
    }
    msgEl.setAttribute("data-gb-citations", "done");

    var body = msgEl.querySelector(".bot-message") || content;
    linkifyMarkers(body, msgData.citations);
  };

  // Hook: wrap the global addMessage (assignment preservation).
  function installHook() {
    if (typeof window.addMessage !== "function" || window.addMessage.__gbCitationsHooked) {
      return;
    }
    var orig = window.addMessage;
    var wrapped = function (sender) {
      var result = orig.apply(this, arguments);
      try {
        if (sender === "bot" && lastBotFrame) {
          var pane = document.getElementById("messages");
          var lastMsgEl = pane ? pane.lastElementChild : null;
          if (lastMsgEl && lastMsgEl.classList.contains("bot")) {
            window.GBRenderCitations(lastMsgEl, lastBotFrame);
          }
          lastBotFrame = null;
        }
      } catch (e) { /* rendering must never break message flow */ }
      return result;
    };
    wrapped.__gbCitationsHooked = true;
    window.addMessage = wrapped;
  }

  // chat-messages.js loads before this module, so the hook installs
  // immediately; retry on DOMContentLoaded as a safety net.
  installHook();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", installHook);
  } else {
    installHook();
  }
})();
