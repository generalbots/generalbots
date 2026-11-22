/* Engine step — an overview of the app's four engines (Settings > Modelo),
 * same order, words, icons, consequence lines and meters. Nothing is chosen
 * here: setup always starts on the free engine, and the card that marks it
 * uses the app's active-card treatment. */
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

  function engineCard(m, startId) {
    var start = m.id === startId;
    return '<div class="wiz-engine' + (start ? " start" : "") + '" role="listitem"' +
      ' data-engine="' + esc(m.id) + '"' +
      (start ? ' aria-current="true"' : "") + '>' +
      '<span class="wiz-engine-top">' +
        '<span class="wiz-engine-ico">' + svg(m.icon, 15) + '</span>' +
        '<span class="wiz-engine-num">' + m.num + '</span>' +
        '<span class="wiz-engine-sp"></span>' +
        (start ? '<span class="wiz-engine-tag">' + esc(t("model.at-start")) + '</span>' : "") +
      '</span>' +
      '<span class="wiz-engine-txt">' +
        '<strong>' + esc(t(m.titleKey)) + '</strong>' +
        '<span>' + esc(t(m.descKey)) + '</span>' +
      '</span>' +
      '<span class="wiz-engine-foot">' +
        '<span class="wiz-engine-dest" data-tone="' + m.tone + '">' +
          '<span class="wiz-engine-sq"></span>' + esc(t(m.consequenceKey)) +
        '</span>' +
        segs(m.privacy, m.tone) +
      '</span></div>';
  }

  function renderEngine(draft) {
    var startId = (draft && draft.model && draft.model.mode) || "free";
    var check = svg("M20 6 9 17l-5-5", 13, 2.2);
    return '' +
      '<p class="lead wiz-engine-lead">' + esc(t("model.overview-lead")) + '</p>' +
      '<section class="wiz-engine-section" aria-labelledby="wizEngineLabel">' +
        '<div class="wiz-engine-head">' +
          '<span class="wiz-engine-label" id="wizEngineLabel">' + sl() + esc(t("model.overview-label")) + '</span>' +
          '<span class="wiz-engine-hint">' + esc(t("model.overview-hint")) + '</span>' +
        '</div>' +
        '<div class="wiz-engine-card">' +
          '<div class="wiz-engine-grid" id="wizEngineGrid" role="list">' +
            W.MODELS.map(function (m) { return engineCard(m, startId); }).join("") +
          '</div>' +
          '<div class="wiz-engine-dhead">' +
            '<div><span class="wiz-engine-k">' + sl() + esc(t("model.detail-label")) + '</span>' +
              '<span class="wiz-engine-dname">' + esc(t("model.free.title")) + '</span></div>' +
            '<span class="wiz-engine-now">' + check + esc(t("model.at-start")) + '</span>' +
          '</div>' +
          '<div class="wiz-engine-dbody" id="wizEngineDetail">' +
            '<p>' + esc(t("model.free.detail-text")) + '</p>' +
            '<p>' + esc(t("model.free.confirm-first")) + '</p>' +
          '</div>' +
        '</div>' +
      '</section>';
  }

  window.GBWizardEngine = {
    renderEngine: renderEngine,
    svg: svg,
    sl: sl,
    segs: segs,
    esc: esc
  };
})();
