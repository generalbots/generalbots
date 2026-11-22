/* Engine step rendering — four cards matching the app's Settings > Modelo page.
 * Split from wizard-steps.js; load order kept in index.html. */
(function () {
  "use strict";
  var W = window.GBWizard;
  var I = window.WizardI18n;
  var t = function (key, params) { return I.t(key, params); };
  var esc = function (value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  };

  function svg(d, size, w) {
    return '<svg width="' + (size || 15) + '" height="' + (size || 15) +
      '" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="' +
      (w || 1.7) + '" stroke-linecap="round" stroke-linejoin="round"' +
      ' aria-hidden="true"><path d="' + d + '"/></svg>';
  }

  function sl() {
    return '<span class="wiz-sl">//</span>';
  }

  function segs(n, tone) {
    var h = "";
    for (var i = 0; i < 5; i += 1) {
      h += '<i class="' + (i < n ? "on" : "") + '"></i>';
    }
    return '<span class="wiz-segs" data-tone="' + tone + '" role="img"' +
      ' aria-label="' + esc(t("model.privacy-label") + " " + n + "/5") + '">' +
      h + '</span>';
  }

  /* ---- The four engine cards ---- */

  function renderEngineCards(draft) {
    var cards = W.MODELS.map(function (m) {
      var selected = false;
      if (m.id === "acp") {
        selected = !!draft.want_agents;
      } else {
        selected = !draft.want_agents && draft.model.mode === m.id;
      }
      return '<button type="button" class="wiz-engine' +
        (selected ? " selected" : "") + '"' +
        ' data-engine="' + esc(m.id) + '"' +
        ' aria-pressed="' + (selected ? "true" : "false") + '"' +
        ' aria-controls="wizEngineDetail">' +
        '<span class="wiz-engine-top">' +
          '<span class="wiz-engine-ico">' + svg(m.icon, 15) + '</span>' +
          '<span class="wiz-engine-num">' + m.num + '</span>' +
          '<span class="wiz-engine-sp"></span>' +
          (selected ? '<span class="wiz-engine-tag">' +
            esc(t("model.selected")) + '</span>' : '') +
        '</span>' +
        '<span class="wiz-engine-txt">' +
          '<strong>' + esc(t(m.titleKey)) + '</strong>' +
          '<span>' + esc(t(m.descKey)) + '</span>' +
        '</span>' +
        '<span class="wiz-engine-foot">' +
          '<span class="wiz-engine-dest" data-tone="' + m.tone + '">' +
            '<span class="wiz-engine-sq"></span>' +
            esc(t(m.consequenceKey)) +
          '</span>' +
          segs(m.privacy, m.tone) +
        '</span></button>';
    }).join("");

    return cards;
  }

  /* ---- Detail panel per engine ---- */

  function detailLocal(draft) {
    return '<div class="wiz-detail-body">' +
      '<p class="wiz-detail-text">' + esc(t("model.local.detail-text")) + '</p>' +
      '<p class="wiz-detail-text">' + esc(t("model.local.detail-size")) + '</p>' +
      '<div id="wizLocalMachine"></div>' +
    '</div>';
  }

  function detailOwnKey(draft) {
    var m = draft.model;
    var providerOptions = ["openai", "anthropic", "nvidia", "custom"].map(function (id) {
      return '<option value="' + id + '"' +
        (m.provider === id ? " selected" : "") + ">" +
        esc(id === "custom" ? t("model.other") : id.charAt(0).toUpperCase() + id.slice(1)) +
        "</option>";
    }).join("");
    var endpoint = m.endpoint || (m.provider === "custom" ? "" : W.defaultModelEndpoint(m.provider));
    var modelName = m.model || (m.provider === "custom" ? "" : W.defaultModelName(m.provider));
    return '<div class="wiz-detail-body">' +
      '<div class="field-grid">' +
        '<div class="field"><label for="modelProvider">' + esc(t("model.provider")) + '</label>' +
          '<select id="modelProvider">' + providerOptions + '</select></div>' +
        '<div class="field"><label for="modelEndpoint">' + esc(t("model.endpoint")) + '</label>' +
          '<input type="text" id="modelEndpoint" value="' + esc(endpoint) + '" placeholder="https://…" />' +
          '<span class="hint">' + esc(t("model.endpoint-hint")) + '</span></div>' +
        '<div class="field"><label for="modelName">' + esc(t("model.name")) + '</label>' +
          '<input type="text" id="modelName" value="' + esc(modelName) + '" placeholder="' + esc(t("model.name-placeholder")) + '" /></div>' +
        '<div class="field"><label for="modelKey">' + esc(t("model.key")) + '</label>' +
          '<div class="inline-row">' +
            '<input type="password" id="modelKey" value="' + esc(m.api_key) + '" autocomplete="off" placeholder="' + esc(t("model.key-placeholder-own")) + '" />' +
            '<button type="button" class="btn btn-secondary btn-sm" id="toggleKeyBtn" aria-pressed="false">' + esc(t("model.show-key")) + '</button>' +
          '</div>' +
          '<span class="hint">' + esc(t("model.key-hint-own")) + '</span></div>' +
        '<div class="inline-row">' +
          '<button type="button" class="btn btn-secondary btn-sm" id="testModelBtn">' + esc(t("model.test")) + '</button>' +
          '<span class="note" id="modelTestStatus" role="status">' + esc(t("model.untested")) + '</span>' +
        '</div>' +
      '</div></div>';
  }

  function detailFree(draft) {
    var m = draft.model;
    var free = W.freeProviderById(m.provider || "kilo");
    var freeOptions = W.FREE_PROVIDERS.map(function (p) {
      return '<option value="' + p.id + '"' +
        (m.provider === p.id ? " selected" : "") + ">" + esc(p.name) + "</option>";
    }).join("");
    return '<div class="wiz-detail-body">' +
      '<p class="wiz-detail-text">' + esc(t("model.free.detail-text")) + '</p>' +
      '<details class="wiz-advanced" id="wizFreeAdvanced">' +
        '<summary>' + esc(t("model.advanced")) + '</summary>' +
        '<div class="field-grid" style="margin-top:12px">' +
          '<div class="field"><label for="modelProvider">' + esc(t("model.free-provider")) + '</label>' +
            '<select id="modelProvider">' + freeOptions + '</select></div>' +
          '<div class="field"><label for="modelEndpoint">' + esc(t("model.endpoint")) + '</label>' +
            '<input type="text" id="modelEndpoint" value="' + esc(m.endpoint || W.defaultModelEndpoint(m.provider)) + '" placeholder="https://…" /></div>' +
          '<div class="field"><label for="modelName">' + esc(t("model.name")) + '</label>' +
            '<input type="text" id="modelName" value="' + esc(m.model || W.defaultModelName(m.provider)) + '" /></div>' +
          (free && free.needsKey ? '<div class="field"><label for="modelKey">' + esc(t("model.key-optional")) + '</label>' +
            '<input type="password" id="modelKey" value="' + esc(m.api_key) + '" autocomplete="off" />' +
            '<span class="hint">' + esc(t("model.key-hint-llm7")) + '</span></div>' : '') +
        '</div>' +
      '</details></div>';
  }

  function detailAcp(draft) {
    var fallback = draft.model.mode === "local" ? "local" : "free";
    return '<div class="wiz-detail-body">' +
      '<p class="wiz-detail-text">' + esc(t("model.acp.detail-line1")) + '</p>' +
      '<p class="wiz-detail-text">' + esc(t("model.acp.detail-line2")) + '</p>' +
      '<p class="wiz-detail-text">' + esc(t("model.acp.detail-wsl2")) + '</p>' +
      '<div class="wiz-detail-section" style="margin-top:8px">' +
        '<span class="wiz-detail-label">' + esc(t("model.acp.fallback-label")) + '</span>' +
        '<div class="wiz-fallback-choices">' +
          '<label class="wiz-fallback-choice">' +
            '<input type="radio" name="acpFallback" value="free"' +
            (fallback === "free" ? " checked" : "") + ' />' +
            '<span>' + esc(t("model.acp.fallback-free")) + '</span>' +
          '</label>' +
          '<label class="wiz-fallback-choice">' +
            '<input type="radio" name="acpFallback" value="local"' +
            (fallback === "local" ? " checked" : "") + ' />' +
            '<span>' + esc(t("model.acp.fallback-local")) + '</span>' +
          '</label>' +
        '</div>' +
      '</div>' +
      '<div id="wizAcpHostCheck"></div>' +
    '</div>';
  }

  function engineDetail(draft) {
    if (draft.want_agents) { return detailAcp(draft); }
    switch (draft.model.mode) {
      case "local":   return detailLocal(draft);
      case "own-key": return detailOwnKey(draft);
      case "free":    return detailFree(draft);
      default:        return detailFree(draft);
    }
  }

  /* ---- Full engine step ---- */

  function renderEngine(draft) {
    var selectedModel = W.MODELS.find(function (m) {
      return draft.want_agents ? m.id === "acp" : m.id === draft.model.mode;
    });
    var detailName = selectedModel ? t(selectedModel.titleKey) : "";

    return '' +
      '<p class="lead">' + esc(t("model.lead")) + '</p>' +
      '<div class="wiz-engine-section">' +
        '<div class="wiz-engine-label">' + sl() +
          esc(t("model.choose-label")) + '</div>' +
        '<div class="wiz-engine-grid" id="wizEngineGrid">' +
          renderEngineCards(draft) +
        '</div>' +
        '<div class="wiz-engine-dhead">' +
          '<div>' + sl() + '<span class="wiz-engine-k">' +
            esc(t("model.detail-label")) + '</span>' +
            '<span class="wiz-engine-dname">' + esc(detailName) + '</span>' +
          '</div>' +
        '</div>' +
        '<div class="wiz-engine-dbody" id="wizEngineDetail">' +
          engineDetail(draft) +
        '</div>' +
      '</div>';
  }

  window.GBWizardEngine = {
    renderEngine: renderEngine,
    engineDetail: engineDetail,
    svg: svg,
    sl: sl,
    segs: segs,
    esc: esc
  };
})();
