"use strict";
/* Sheets enterprise controls (#2437) — external-link host allowlist and
 * fail-closed local state. Credential handling through Vault is deferred; this
 * preview stores no secrets and never fetches a remote URL. */

(function () {
  var C = window.SheetControls;

  function normalizeHost(value) {
    return String(value || "").trim().toLowerCase().replace(/^https?:\/\//, "").split(/[/?#:]/)[0];
  }

  function validHost(host) {
    var clean = normalizeHost(host);
    if (clean.length < 4 || clean.length > 253) return false;
    if (clean.indexOf("..") !== -1 || clean.charAt(0) === "." || clean.charAt(clean.length - 1) === ".") return false;
    return /^[a-z0-9-]+(?:\.[a-z0-9-]+)+$/.test(clean) && /\.[a-z]{2,}$/.test(clean);
  }

  function addHost(host) {
    var clean = normalizeHost(host);
    if (!validHost(clean)) return false;
    C.mutate(function (state) {
      if (state.links.allowlist.indexOf(clean) === -1) state.links.allowlist.push(clean);
    });
    refreshState();
    return true;
  }

  function removeHost(host) {
    var clean = normalizeHost(host);
    C.mutate(function (state) {
      state.links.allowlist = state.links.allowlist.filter(function (item) { return item !== clean; });
    });
    refreshState();
    return true;
  }

  function isApproved(host) {
    return C.state().links.allowlist.indexOf(normalizeHost(host)) !== -1;
  }

  function addLink(url) {
    var host = normalizeHost(url);
    if (!validHost(host)) return { ok: false, key: "sheet-ctrl-link-host-invalid" };
    var approved = isApproved(host);
    C.mutate(function (state) {
      state.links.current = state.links.current.filter(function (link) { return link.host !== host; });
      state.links.current.push({ host: host, url: String(url || "").trim(), approved: approved });
    });
    return { ok: true, host: host, approved: approved };
  }

  /* Removing a host from the allowlist revokes every link that used it. */
  function refreshState() {
    var changed = 0;
    C.mutate(function (state) {
      state.links.current.forEach(function (link) {
        var approved = state.links.allowlist.indexOf(link.host) !== -1;
        if (link.approved !== approved) {
          link.approved = approved;
          changed++;
        }
      });
    });
    return changed;
  }

  function render(container, ctx) {
    var doc = container.ownerDocument || document;
    function make(tag, className, text) {
      var node = doc.createElement(tag);
      if (className) node.className = className;
      if (text != null) node.textContent = text;
      return node;
    }
    var links = C.state().links;
    var grid = make("div", "ss-ctrl-form");

    var allowLabel = make("p", "ss-ctrl-label", C.t("sheet-ctrl-link-allowlist", "Approved hosts"));
    allowLabel.setAttribute("data-i18n", "sheet-ctrl-link-allowlist");
    grid.appendChild(allowLabel);
    var allowList = make("ul", "ss-ctrl-list");
    if (!links.allowlist.length) {
      var emptyHosts = make("li", "ss-ctrl-empty", C.t("sheet-ctrl-link-empty", "No external links are listed."));
      emptyHosts.setAttribute("data-i18n", "sheet-ctrl-link-empty");
      allowList.appendChild(emptyHosts);
    } else {
      links.allowlist.forEach(function (host) {
        var item = make("li", "ss-ctrl-list-item");
        item.textContent = host;
        item.setAttribute("data-host", host);
        var button = make("button", "ss-ctrl-button-small", C.t("sheet-ctrl-remove", "Remove"));
        button.type = "button";
        button.setAttribute("data-i18n", "sheet-ctrl-remove");
        button.addEventListener("click", function () {
          removeHost(host);
          ctx.setStatus(C.t("sheet-ctrl-status-ready", "Ready."), false);
          ctx.rerender();
        });
        item.appendChild(button);
        allowList.appendChild(item);
      });
    }
    grid.appendChild(allowList);

    var hostInput = make("input", "ss-ctrl-input");
    hostInput.type = "text";
    hostInput.id = "ss-ctrl-link-host";
    hostInput.placeholder = C.t("sheet-ctrl-link-host", "Host name");
    hostInput.setAttribute("data-i18n-placeholder", "sheet-ctrl-link-host");
    var addHostButton = make("button", "ss-ctrl-button", C.t("sheet-ctrl-link-add-host", "Add approved host"));
    addHostButton.type = "button";
    addHostButton.id = "ss-ctrl-link-add-host";
    addHostButton.setAttribute("data-i18n", "sheet-ctrl-link-add-host");
    var hostRow = make("div", "ss-ctrl-row");
    hostRow.appendChild(hostInput);
    hostRow.appendChild(addHostButton);
    grid.appendChild(hostRow);

    var currentLabel = make("p", "ss-ctrl-label", C.t("sheet-ctrl-link-current", "Current external links"));
    currentLabel.setAttribute("data-i18n", "sheet-ctrl-link-current");
    grid.appendChild(currentLabel);
    var currentList = make("ul", "ss-ctrl-list");
    if (!links.current.length) {
      var emptyLinks = make("li", "ss-ctrl-empty", C.t("sheet-ctrl-link-empty", "No external links are listed."));
      emptyLinks.setAttribute("data-i18n", "sheet-ctrl-link-empty");
      currentList.appendChild(emptyLinks);
    } else {
      links.current.forEach(function (link) {
        var item = make("li", "ss-ctrl-list-item");
        var badge = make("span", link.approved ? "ss-ctrl-badge-approved" : "ss-ctrl-badge-blocked",
          C.t(link.approved ? "sheet-ctrl-link-approved" : "sheet-ctrl-link-blocked",
            link.approved ? "Approved" : "Blocked"));
        badge.setAttribute("data-i18n", link.approved ? "sheet-ctrl-link-approved" : "sheet-ctrl-link-blocked");
        if (!link.approved) {
          badge.title = C.t("sheet-ctrl-link-blocked-note", "Unapproved host - blocked locally (fail closed).");
          badge.setAttribute("data-i18n-title", "sheet-ctrl-link-blocked-note");
        }
        item.textContent = link.host + " ";
        item.appendChild(badge);
        currentList.appendChild(item);
      });
    }
    grid.appendChild(currentList);

    var urlInput = make("input", "ss-ctrl-input");
    urlInput.type = "text";
    urlInput.id = "ss-ctrl-link-url";
    urlInput.placeholder = C.t("sheet-ctrl-link-url", "External link URL");
    urlInput.setAttribute("data-i18n-placeholder", "sheet-ctrl-link-url");
    var addLinkButton = make("button", "ss-ctrl-button", C.t("sheet-ctrl-add", "Add"));
    addLinkButton.type = "button";
    addLinkButton.id = "ss-ctrl-link-add";
    addLinkButton.setAttribute("data-i18n", "sheet-ctrl-add");
    var linkRow = make("div", "ss-ctrl-row");
    linkRow.appendChild(urlInput);
    linkRow.appendChild(addLinkButton);
    grid.appendChild(linkRow);
    container.appendChild(grid);

    addHostButton.addEventListener("click", function () {
      var ok = addHost(hostInput.value);
      ctx.setStatus(ok
        ? C.t("sheet-ctrl-link-added", "Host added to the local allowlist.")
        : C.t("sheet-ctrl-link-host-invalid", "Enter a valid host name."), !ok);
      if (ok) ctx.rerender();
    });
    addLinkButton.addEventListener("click", function () {
      var result = addLink(urlInput.value);
      ctx.setStatus(result.ok
        ? C.t(result.approved ? "sheet-ctrl-link-approved" : "sheet-ctrl-link-blocked-note",
          result.approved ? "Approved" : "Unapproved host - blocked locally (fail closed).")
        : C.t("sheet-ctrl-link-host-invalid", "Enter a valid host name."), !result.ok);
      if (result.ok) ctx.rerender();
    });
  }

  window.SheetControlsLinks = {
    normalizeHost: normalizeHost,
    validHost: validHost,
    addHost: addHost,
    removeHost: removeHost,
    isApproved: isApproved,
    addLink: addLink,
    refreshState: refreshState,
    render: render,
  };
})();
