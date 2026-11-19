/* Element and page translation (data-i18n attributes) for the suite i18n
   runtime. Split out of i18n.js so every module stays under the 450-line
   limit; behaviour is unchanged. Loaded right after i18n.js. */
(function () {
  "use strict";
  var shared = window.__gbI18nShared || {};
  var t = shared.t;

  function elementParams(element) {
    const paramsAttr = element.getAttribute("data-i18n-params");
    if (!paramsAttr) return null;
    try {
      return JSON.parse(paramsAttr);
    } catch (e) {
      console.warn("i18n: Invalid params JSON", paramsAttr);
      return null;
    }
  }

  function translateElement(element) {
    const key = element.getAttribute("data-i18n");
    if (key) {
      const params = elementParams(element);
      const value = t(key, params);
      // A missing catalogue entry returns the key. Keep the authored English
      // text in that case instead of replacing it with a raw identifier.
      if (value !== key) element.textContent = value;
    }

    // Opt-in HTML translation for strings that intentionally carry markup
    // (e.g. a link inside a sentence). Only catalogue content is injected.
    const htmlKey = element.getAttribute("data-i18n-html");
    if (htmlKey) {
      const value = t(htmlKey, elementParams(element));
      if (value !== htmlKey) element.innerHTML = value;
    }

    const placeholderKey = element.getAttribute("data-i18n-placeholder");
    if (placeholderKey) {
      const value = t(placeholderKey);
      if (value !== placeholderKey) element.setAttribute("placeholder", value);
    }

    const titleKey = element.getAttribute("data-i18n-title");
    if (titleKey) {
      const value = t(titleKey);
      if (value !== titleKey) element.setAttribute("title", value);
    }

    const ariaLabelKey = element.getAttribute("data-i18n-aria-label");
    if (ariaLabelKey) {
      const value = t(ariaLabelKey);
      if (value !== ariaLabelKey) element.setAttribute("aria-label", value);
    }

    const valueKey = element.getAttribute("data-i18n-value");
    if (valueKey) {
      const value = t(valueKey);
      if (value !== valueKey) element.setAttribute("value", value);
    }
  }

  function translatePage(root) {
    const container = root || document;

    const elements = container.querySelectorAll(
      "[data-i18n], [data-i18n-html], [data-i18n-placeholder], [data-i18n-title], [data-i18n-aria-label], [data-i18n-value]",
    );

    elements.forEach(translateElement);

    const titleKey = document.documentElement.getAttribute("data-i18n-document-title");
    if (titleKey) {
      const value = t(titleKey);
      if (value !== titleKey) document.title = value;
    }
  }

  shared.translatePage = translatePage;
  shared.translateElement = translateElement;
  if (window.i18n) {
    window.i18n.translatePage = translatePage;
    window.i18n.translateElement = translateElement;
  }
})();
