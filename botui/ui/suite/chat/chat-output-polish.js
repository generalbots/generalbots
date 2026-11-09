/* Output polish: turn workspace tool output ("installed: reportlab…",
 * "PDF criado: output/…") into a console card. DOM-only, no regex over HTML. */

var GB_TOOL_OUTPUT =
  /(Executando no workspace|installed:|PDF criado|criado:|salvo em|Gerado:|Saved:|created:|written to|output\/|media\/)/;

function polishOutputBlocks(el) {
  var content = el && (el.querySelector ? el.querySelector(".message-content") : null);
  if (!content) content = el;
  if (!content || !content.childNodes || content.__polished) return;
  content.__polished = true;

  // One "unit" = a text node, or a simple block element (P/DIV/SPAN) whose
  // content is only text/BR — enough to cover marked's paragraph wrapping.
  function unitText(node) {
    if (node.nodeType === 3) return node.nodeValue;
    if (node.nodeType === 1 && /^(P|DIV|SPAN|PRE)$/i.test(node.tagName)) {
      if (Array.prototype.some.call(node.querySelectorAll("*"), function (ch) {
        return !/^(B|BR|I|EM|STRONG|CODE|A|SPAN)$/i.test(ch.tagName);
      })) return null;
      return node.innerText || node.textContent;
    }
    return null;
  }

  var units = [];
  Array.prototype.forEach.call(content.childNodes, function (n) {
    var text = unitText(n);
    if (text !== null && text.trim()) units.push({ node: n, text: text });
  });

  // Group consecutive units when at least one line looks like tool output.
  var groups = [];
  var cur = [];
  units.forEach(function (u) {
    if (u.text.split("\n").some(function (l) { return GB_TOOL_OUTPUT.test(l); })) {
      cur.push(u);
    } else if (cur.length) {
      groups.push(cur); cur = [];
    }
  });
  if (cur.length) groups.push(cur);

  groups.forEach(function (g) {
    var text = g
      .map(function (u) { return u.text; })
      .join("\n")
      .split("\n")
      .map(function (line) { return line.trim(); })
      // Tool logs end with "Files" and empty "-" bullets; they carry no
      // information once the file cards are rendered below.
      .filter(function (line) {
        return line && !/^files$/i.test(line) && !/^-+$/.test(line);
      })
      .join("\n")
      .replace(/\n{2,}/g, "\n");
    if (!text) return;
    // Collapsed by default: the run is evidence, not the answer. The reader
    // opens it only when diagnosing the workspace step.
    var card = document.createElement("details");
    card.className = "gb-console";
    var head = document.createElement("summary");
    head.className = "gb-console-head";
    head.textContent = "workspace run";
    var pre = document.createElement("pre");
    pre.textContent = text;
    card.appendChild(head);
    card.appendChild(pre);
    var anchor = g[0].node;
    anchor.parentNode.insertBefore(card, anchor);
    g.forEach(function (u) {
      if (u.node.parentNode) u.node.parentNode.removeChild(u.node);
    });
  });
}

// Run-timeline cards are appended to #messages between bubbles, so the raw
// output bubble's previousElementSibling is often a `.run-card` section, not
// the bot bubble that owns the console card. Walk back over anything that is
// not a turn bubble: a user message ends the search (never merge across turns).
function previousBotMessage(message) {
  var prev = message.previousElementSibling;
  while (prev) {
    if (prev.classList && prev.classList.contains("message")) {
      return prev.classList.contains("bot") ? prev : null;
    }
    prev = prev.previousElementSibling;
  }
  return null;
}

// The log a bubble contributes to a run card: an unfiltered tool result is a
// direct `<pre>`, but polishOutputBlocks may already have moved it inside a
// `details.gb-console`. Handle both so render order cannot create a second
// "workspace run" card (#2008).
function messageLogPre(message) {
  var content = message.querySelector(".message-content");
  if (!content) return null;
  var children = content.children || [];
  for (var i = 0; i < children.length; i++) {
    if (String(children[i].tagName || "").toUpperCase() === "PRE") return children[i];
  }
  return content.querySelector("details.gb-console > pre");
}

/* A tool run sends a status bubble ("workspace run") and then the raw output
 * as its own code block. Keep the answer readable: move the output into the
 * collapsed run card instead of leaving an `ls -la` listing as a chat bubble. */
function mergeWorkspaceOutput(el) {
  var message = el && el.classList && el.classList.contains("message")
    ? el
    : (el && el.closest ? el.closest(".message") : null);
  if (!message) return false;
  var prev = previousBotMessage(message);
  if (!prev) return false;
  var card = prev.querySelector(".gb-console");
  if (!card) return false;
  var pre = messageLogPre(message);
  if (!pre) return false;
  var text = (pre.textContent || "").trim();
  if (!text) return false;
  // Only shell/tool output; never merge a model-authored code answer.
  if (!/^(total \d+|drwx|-\w{9}|installed:|PDF created|found:|size:|\/(?:home|opt|usr|tmp|var)\b)/m.test(text)) {
    return false;
  }
  var log = card.querySelector("pre");
  if (!log) return false;
  // The status bubble and the result bubble can carry the same log; appending
  // it twice would just duplicate lines inside the collapsed card.
  if (log.textContent.indexOf(text) === -1) {
    log.textContent = (log.textContent ? log.textContent + "\n" : "") + text;
  }
  // The result bubble also carries the generated-file cards. Keep them: move
  // them under the bubble that survives before dropping the raw bubble.
  var attachments = message.querySelectorAll(".gb-attachment");
  if (attachments.length) {
    var host = prev.querySelector(".message-content") || prev;
    for (var i = 0; i < attachments.length; i++) {
      host.appendChild(attachments[i]);
    }
  }
  var summary = card.querySelector("summary");
  if (summary) {
    summary.textContent = "workspace run · " + log.textContent.split("\n").length + " lines";
  }
  message.remove();
  if (attachments.length && typeof hydrateVmAttachments === "function" && prev.isConnected) {
    hydrateVmAttachments(prev);
  }
  return true;
}
window.mergeWorkspaceOutput = mergeWorkspaceOutput;
