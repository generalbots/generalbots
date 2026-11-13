/* #2005 — incremental SVG engine behind the live drawing preview.
 *
 * The model streams a fenced ```svg block token by token. Re-parsing the whole
 * document on every chunk (the #1693 iframe behaviour) can only produce a
 * static picture at the end, so this module keeps a persistent surface and
 * appends only what finished:
 *
 *   tokenize   → scans complete tags, holding back an unterminated tail
 *   buildTree  → elements still streaming keep `end = -1`
 *   render     → closed subtrees are appended once; an open wrapper is created
 *                once and receives its children as they complete
 *   animateIn  → each new node draws itself in (dash draw-on for stroked
 *                paths, fade+scale for shapes and text)
 *
 * Generated markup is inserted into the suite document, so `sanitize` strips
 * scripts, inline handlers, foreignObject and external references first.
 */
"use strict";

window.GBLiveSvg = window.GBLiveSvg || (function () {
  var SVG_NS = "http://www.w3.org/2000/svg";

  /* ------------------------------------------------------------- tokenizer */

  // Scans one markup token at `at`. Returns null when the token has not fully
  // streamed yet (an unterminated tag or comment), which is what keeps a
  // half-written <rect x="1 during the stream from being rendered.
  function scanTag(s, at) {
    if (s.charAt(at) !== "<") return null;
    if (s.slice(at, at + 4) === "<!--") {
      var c = s.indexOf("-->", at + 4);
      return c < 0 ? null : { kind: "markup", end: c + 3 };
    }
    if (s.slice(at, at + 9) === "<![CDATA[") {
      var cd = s.indexOf("]]>", at + 9);
      return cd < 0 ? null : { kind: "markup", end: cd + 3 };
    }
    var next = s.charAt(at + 1);
    if (next === "!" || next === "?") {
      var term = next === "?" ? "?>" : ">";
      var decl = s.indexOf(term, at + 2);
      return decl < 0 ? null : { kind: "markup", end: decl + term.length };
    }
    var closing = next === "/";
    var i = at + (closing ? 2 : 1);
    var name = "";
    while (i < s.length && !/[\s/>]/.test(s.charAt(i))) {
      name += s.charAt(i);
      i++;
    }
    if (!name) return null;
    // Quotes matter: an attribute may legitimately contain ">".
    var quote = null;
    for (; i < s.length; i++) {
      var ch = s.charAt(i);
      if (quote) { if (ch === quote) quote = null; continue; }
      if (ch === '"' || ch === "'") { quote = ch; continue; }
      if (ch === ">") {
        return {
          kind: closing ? "close" : "open",
          name: name.toLowerCase(),
          selfClosing: !closing && s.charAt(i - 1) === "/",
          end: i + 1,
        };
      }
    }
    return null;
  }

  function tokenize(source) {
    var tokens = [];
    var i = 0;
    while (i < source.length) {
      var lt = source.indexOf("<", i);
      if (lt < 0) {
        tokens.push({ kind: "text", start: i, end: source.length });
        break;
      }
      if (lt > i) tokens.push({ kind: "text", start: i, end: lt });
      var tag = scanTag(source, lt);
      if (!tag) break; // incomplete tail: hold it back until more arrives
      if (tag.kind === "open") {
        tokens.push({
          kind: tag.selfClosing ? "self" : "open",
          name: tag.name, start: lt, end: tag.end,
        });
      } else if (tag.kind === "close") {
        tokens.push({ kind: "close", name: tag.name, start: lt, end: tag.end });
      } else {
        tokens.push({ kind: "markup", start: lt, end: tag.end });
      }
      i = tag.end;
    }
    return tokens;
  }

  // Turns the token list into a tree. `end` stays -1 while an element is still
  // open, which is exactly the flag the renderer needs to decide "append the
  // finished subtree" vs "create the wrapper and descend".
  function buildTree(tokens, source) {
    var root = { children: [] };
    var stack = [root];
    for (var i = 0; i < tokens.length; i++) {
      var t = tokens[i];
      var parent = stack[stack.length - 1];
      if (t.kind === "open") {
        var node = {
          name: t.name, start: t.start, openTag: source.slice(t.start, t.end),
          end: -1, children: [],
        };
        parent.children.push(node);
        stack.push(node);
      } else if (t.kind === "self") {
        parent.children.push({
          name: t.name, start: t.start, end: t.end,
          openTag: source.slice(t.start, t.end), children: [],
        });
      } else if (t.kind === "close") {
        for (var d = stack.length - 1; d > 0; d--) {
          if (stack[d].name === t.name) {
            stack[d].end = t.end;
            // Pop the closed element itself: the next sibling belongs to its
            // parent. Keeping it on the stack nested every following element
            // inside the last closed one (a stripped <script> swallowed the
            // rest of the scene).
            stack.length = d;
            break;
          }
        }
      } else {
        var text = source.slice(t.start, t.end);
        if (t.kind === "text") {
          parent.children.push({ text: text, start: t.start, end: t.end });
        } else {
          parent.children.push({ raw: text, start: t.start, end: t.end });
        }
      }
    }
    return root;
  }

  /* -------------------------------------------------------------- security */

  // Generated markup is inserted into the suite document (never the iframe the
  // HTML path uses), so it must carry no script, no inline handler and no
  // network reference. innerHTML does not execute <script>, but on* handlers
  // do fire, and hrefs can exfiltrate or phone home.
  function sanitize(markup) {
    return String(markup || "")
      .replace(/<script\b[\s\S]*?<\/script\s*>/gi, "")
      .replace(/<script\b[^>]*\/?>/gi, "")
      .replace(/<foreignObject\b[\s\S]*?<\/foreignObject\s*>/gi, "")
      .replace(/<foreignObject\b[^>]*\/?>/gi, "")
      .replace(/\son[a-z]+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, "")
      .replace(/(?:href|xlink:href)\s*=\s*("|')\s*(?:javascript:|data:text\/html)[^"']*\1/gi, "")
      .replace(/(?:href|xlink:href)\s*=\s*("|')\s*(?:https?:)?\/\/[^"']*\1/gi, "");
  }

  /* --------------------------------------------------------------- helpers */

  function createElementFromTag(openTag) {
    var tag = /^<\s*([a-zA-Z][\w:.-]*)/.exec(openTag);
    if (!tag) return null;
    var probe = document.createElementNS(SVG_NS, "svg");
    probe.innerHTML = openTag + "</" + tag[1] + ">";
    return probe.firstElementChild || null;
  }

  // An <svg> without a viewBox cannot be scaled: shrinking the element clips
  // the drawing instead of resizing it (a 512px circle previewed as its top
  // left quarter). Numeric width/height give the same box, so derive the
  // missing viewBox from them.
  function ensureViewBox(root) {
    if (root.hasAttribute("viewBox")) return;
    var w = parseFloat(root.getAttribute("width"));
    var h = parseFloat(root.getAttribute("height"));
    if (isFinite(w) && isFinite(h) && w > 0 && h > 0) {
      root.setAttribute("viewBox", "0 0 " + w + " " + h);
    }
  }

  function appendMarkup(parentEl, markup) {
    var clean = sanitize(markup);
    if (!clean.trim()) return 0;
    var probe = document.createElementNS(SVG_NS, "svg");
    probe.innerHTML = clean;
    var added = 0;
    while (probe.firstChild) {
      var child = probe.removeChild(probe.firstChild);
      parentEl.appendChild(child);
      if (child.nodeType === 1) {
        animateIn(child);
        added += countDrawable(child);
      }
    }
    return added;
  }

  // A whole closed subtree arrives as one markup slice; the badge must report
  // the same number of parts it would have reported while streaming that
  // subtree element by element. Structural wrappers are not parts.
  var STRUCTURAL = /^(g|svg|defs|symbol|marker|mask|pattern|clippath|lineargradient|radialgradient|style|title|desc|metadata)$/;

  function countDrawable(el) {
    var name = (el.tagName || "").toLowerCase();
    var total = STRUCTURAL.test(name) ? 0 : 1;
    var descendants = el.querySelectorAll("*");
    for (var i = 0; i < descendants.length; i++) {
      if (!STRUCTURAL.test(descendants[i].tagName.toLowerCase())) total++;
    }
    return total;
  }

  function animateIn(el) {
    if (!el || typeof el.animate !== "function") return;
    var tag = (el.tagName || "").toLowerCase();
    var stroke = el.getAttribute("stroke");
    var stroked = /^(path|line|polyline|polygon)$/.test(tag) &&
      stroke && stroke !== "none";
    if (stroked && typeof el.getTotalLength === "function") {
      try {
        var len = el.getTotalLength();
        if (isFinite(len) && len > 0) {
          el.style.strokeDasharray = len + " " + len;
          el.style.strokeDashoffset = String(len);
          var draw = el.animate(
            [{ strokeDashoffset: len }, { strokeDashoffset: 0 }],
            { duration: 520, easing: "ease-out", fill: "none" }
          );
          draw.onfinish = function () {
            el.style.strokeDasharray = "";
            el.style.strokeDashoffset = "";
          };
          return;
        }
      } catch (e) { /* geometry with no measurable length falls back */ }
    }
    el.style.transformBox = "fill-box";
    el.style.transformOrigin = "center";
    el.animate(
      [
        { opacity: 0, transform: "scale(0.92)" },
        { opacity: 1, transform: "scale(1)" },
      ],
      { duration: 320, easing: "cubic-bezier(.2,.8,.3,1)", fill: "none" }
    );
  }

  /* --------------------------------------------------------------- renderer */

  function syncChildren(nodes, parentEl, prefix, state, source) {
    for (var i = 0; i < nodes.length; i++) {
      var node = nodes[i];
      if (node.text !== undefined) {
        if (!node.text.trim()) continue;
        var tk = prefix + "t" + i;
        if (!state.applied[tk]) {
          state.applied[tk] = true;
          parentEl.appendChild(document.createTextNode(node.text));
        }
        continue;
      }
      var key = prefix + "e" + i;
      if (node.raw) {
        if (!state.applied[key]) {
          state.applied[key] = true;
          state.parts += appendMarkup(parentEl, node.raw);
        }
        continue;
      }
      if (!node.name) continue;
      if (node.end >= 0) {
        // Closed subtree.
        if (!state.applied[key]) {
          state.applied[key] = true;
          state.parts += appendMarkup(parentEl, source.slice(node.start, node.end));
          continue;
        }
        // The container was already created while this element streamed open:
        // its last children may have arrived in the same chunk as the closing
        // tag, so finish them here instead of skipping the subtree.
        var open = state.containers[key];
        if (open) {
          syncChildren(node.children, open, key + ".", state, source);
        }
        continue;
      }
      // Still streaming: create the wrapper, then fill its finished children.
      var wrap = state.containers[key];
      if (!wrap) {
        wrap = createElementFromTag(sanitize(node.openTag));
        if (!wrap) continue;
        parentEl.appendChild(wrap);
        state.containers[key] = wrap;
        // A <g> is structural; popping it in would animate the whole scene
        // twice. Its children animate individually instead.
        if (node.name !== "g") animateIn(wrap);
      }
      // Remember the key: once this element closes it must not be appended a
      // second time from its raw slice (that would duplicate the subtree).
      state.applied[key] = true;
      syncChildren(node.children, wrap, key + ".", state, source);
    }
  }

  function render(state, block) {
    var tree = buildTree(tokenize(block.body), block.body);
    var svgNode = null;
    for (var i = 0; i < tree.children.length; i++) {
      if (tree.children[i].name === "svg") { svgNode = tree.children[i]; break; }
    }
    if (!svgNode) {
      state.badge.textContent = "preparing…";
      return;
    }
    if (!state.root) {
      var root = createElementFromTag(sanitize(svgNode.openTag));
      if (!root || root.tagName.toLowerCase() !== "svg") return;
      if (!root.getAttribute("preserveAspectRatio")) {
        root.setAttribute("preserveAspectRatio", "xMidYMid meet");
      }
      ensureViewBox(root);
      state.host.appendChild(root);
      state.root = root;
    }
    syncChildren(svgNode.children, state.root, "r.", state, block.body);
    state.badge.textContent = block.closed
      ? state.parts + " parts"
      : "drawing… " + state.parts + " parts";
  }

  return {
    render: render,
    sanitize: sanitize,
    tokenize: tokenize,
    buildTree: buildTree,
  };
})();
