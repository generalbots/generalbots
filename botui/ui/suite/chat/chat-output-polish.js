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

/* A tool run sends a status bubble ("workspace run") and then the raw output
 * as its own code block. Keep the answer readable: move the output into the
 * collapsed run card instead of leaving an `ls -la` listing as a chat bubble. */
function mergeWorkspaceOutput(el) {
  var message = el && el.classList && el.classList.contains("message")
    ? el
    : (el && el.closest ? el.closest(".message") : null);
  if (!message) return false;
  var prev = message.previousElementSibling;
  if (!prev || !prev.classList || !prev.classList.contains("bot")) return false;
  var card = prev.querySelector(".gb-console");
  if (!card) return false;
  var pre = message.querySelector(".message-content > pre");
  if (!pre) return false;
  var text = (pre.textContent || "").trim();
  if (!text) return false;
  // Only shell/tool output; never merge a model-authored code answer.
  if (!/^(total \d+|drwx|-\w{9}|installed:|PDF created|found:|size:|\/(?:home|opt|usr|tmp|var)\b)/m.test(text)) {
    return false;
  }
  var log = card.querySelector("pre");
  if (!log) return false;
  log.textContent = (log.textContent ? log.textContent + "\n" : "") + text;
  var summary = card.querySelector("summary");
  if (summary) {
    summary.textContent = "workspace run · " + log.textContent.split("\n").length + " lines";
  }
  message.remove();
  return true;
}
window.mergeWorkspaceOutput = mergeWorkspaceOutput;
