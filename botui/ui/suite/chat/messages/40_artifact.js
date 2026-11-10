/* Chat messages module: Inline sandboxed artifacts and user attachment previews. */
function renderInlineArtifact(html, titleOverride) {
  var box = document.createElement("div");
  box.className = "gb-inline-artifact";
  var head = document.createElement("div");
  head.className = "gb-inline-artifact-head";
  var title = document.createElement("span");
  title.className = "gb-inline-artifact-title";
  title.textContent = titleOverride || inlineArtifactTitle(html);
  head.appendChild(title);
  var expand = document.createElement("button");
  expand.type = "button";
  expand.className = "gb-inline-artifact-btn";
  expand.textContent = "Expand";
  expand.setAttribute("aria-expanded", "false");
  expand.addEventListener("click", function () {
    // #2099 — expand opens the artifact over the whole pane; the bubble's 80%
    // cap no longer applies because the box leaves `.message`.
    if (window.GBArtifactOverlay) {
      window.GBArtifactOverlay.toggle(box, expand, {
        label: title.textContent || "Preview",
      });
      return;
    }
    box.classList.toggle("expanded");
    expand.textContent = box.classList.contains("expanded") ? "Collapse" : "Expand";
  });
  head.appendChild(expand);
  var frame = document.createElement("iframe");
  frame.className = "gb-inline-artifact-frame";
  // allow-scripts enables the interactive chart; no allow-same-origin keeps it
  // away from the suite DOM, cookies and storage.
  frame.setAttribute("sandbox", "allow-scripts allow-forms allow-modals");
  frame.setAttribute("referrerpolicy", "no-referrer");
  frame.setAttribute("loading", "lazy");
  frame.setAttribute("title", title.textContent);
  frame.srcdoc = html;
  frame.addEventListener("load", function () {
    setTimeout(function () { snapToBottom(); stickToBottom(900); }, 60);
  });
  box.appendChild(head);
  box.appendChild(frame);
  return box;
}

// User-uploaded attachments (#1673): the chip under a user turn offers a
// Preview button that streams the staged Drive inbox copy through the
// sanitized preview endpoint. Nothing is fetched until the button is pressed.
function userAttachmentPreviewUrl(name) {
  var bot =
    (window.ChatState && ChatState.currentBotName) ||
    window.__INITIAL_BOT_NAME__ ||
    "default";
  var url =
    "/api/chat/attachments/preview?bot=" +
    encodeURIComponent(bot) +
    "&name=" +
    encodeURIComponent(name);
  var token = "";
  try {
    token =
      (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      "";
  } catch (e) { /* storage unavailable */ }
  if (token) url += "&access_token=" + encodeURIComponent(token);
  return url;
}

function toggleUserAttachmentPreview(chip, name) {
  if (!chip) return;
  var existing = chip.nextElementSibling;
  if (existing && existing.classList && existing.classList.contains("gb-user-preview")) {
    existing.remove();
    return;
  }
  var box = document.createElement("div");
  box.className = "gb-attachment-inline gb-user-preview";

  var head = document.createElement("div");
  head.className = "gb-attachment-inline-head";
  var title = document.createElement("span");
  title.textContent = name;
  head.appendChild(title);

  var expand = document.createElement("button");
  expand.type = "button";
  expand.className = "gb-attachment-inline-btn";
  expand.textContent = "Expand";
  expand.setAttribute("aria-expanded", "false");
  expand.addEventListener("click", function () {
    // #2099 — same modal layer as the other inline previews.
    if (window.GBArtifactOverlay) {
      window.GBArtifactOverlay.toggle(box, expand, { label: name });
      return;
    }
    box.classList.toggle("expanded");
    expand.textContent = box.classList.contains("expanded") ? "Collapse" : "Expand";
  });
  head.appendChild(expand);

  var close = document.createElement("button");
  close.type = "button";
  close.className = "gb-attachment-inline-btn";
  close.setAttribute("aria-label", "Close preview");
  close.textContent = "×";
  close.addEventListener("click", function () {
    if (window.GBArtifactOverlay) window.GBArtifactOverlay.close(box);
    box.remove();
  });
  head.appendChild(close);

  var body = document.createElement("div");
  body.className = "gb-attachment-inline-body";
  var frame = document.createElement("iframe");
  frame.src = userAttachmentPreviewUrl(name);
  frame.title = "Preview of " + name;
  // Chrome refuses to run its PDF viewer inside a sandboxed frame; the
  // server already sends application/pdf + nosniff, so a PDF is safe.
  if (!/\.pdf$/i.test(name)) frame.setAttribute("sandbox", "");
  frame.setAttribute("referrerpolicy", "no-referrer");
  frame.setAttribute("loading", "lazy");
  body.appendChild(frame);

  box.appendChild(head);
  box.appendChild(body);
  chip.after(box);
  if (typeof snapToBottom === "function") snapToBottom();
}
