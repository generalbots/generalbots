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
      .replace(/\n{2,}/g, "\n")
      .replace(/^\s+|\s+$/g, "");
    if (!text) return;
    var card = document.createElement("div");
    card.className = "gb-console";
    var head = document.createElement("div");
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
