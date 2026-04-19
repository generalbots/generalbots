(function () {
  "use strict";

  function updateSVGTheme(objectElement) {
    try {
      const svgDoc = objectElement.contentDocument;
      if (!svgDoc) return;

      const html = document.documentElement;
      const body = document.body;
      const svgRoot = svgDoc.documentElement;

      const isRust =
        html.classList.contains("rust") || body.classList.contains("rust");
      const isLight =
        html.classList.contains("light") || body.classList.contains("light");
      const isDark =
        html.classList.contains("coal") ||
        body.classList.contains("coal") ||
        html.classList.contains("navy") ||
        body.classList.contains("navy") ||
        html.classList.contains("ayu") ||
        body.classList.contains("ayu") ||
        body.classList.contains("theme--dark");

      if (isRust || isLight) {
        svgRoot.style.setProperty("--main-text-color", "#1a1a1a");
        svgRoot.style.setProperty("--main-bg-color", "#ffffff");
        svgRoot.style.setProperty("--accent-color", "#4a90d9");
        svgRoot.style.setProperty("--border-color", "#e0e0e0");
      } else if (isDark) {
        svgRoot.style.setProperty("--main-text-color", "#e0e0e0");
        svgRoot.style.setProperty("--main-bg-color", "#1a1a2e");
        svgRoot.style.setProperty("--accent-color", "#64b5f6");
        svgRoot.style.setProperty("--border-color", "#333355");
      }

      if (
        svgDoc.defaultView &&
        typeof svgDoc.defaultView.updateTheme === "function"
      ) {
        svgDoc.defaultView.updateTheme();
      }
    } catch (e) {
      console.debug("Could not update SVG theme:", e);
    }
  }

  function initializeSVGThemes() {
    const objects = document.querySelectorAll('object[type="image/svg+xml"]');
    objects.forEach((obj) => {
      if (obj.contentDocument) {
        updateSVGTheme(obj);
      } else {
        obj.addEventListener("load", () => updateSVGTheme(obj));
      }
    });
  }

  function watchThemeChanges() {
    const observer = new MutationObserver((mutations) => {
      const classChanged = mutations.some(
        (m) => m.type === "attributes" && m.attributeName === "class",
      );

      if (classChanged) {
        initializeSVGThemes();
      }
    });

    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    observer.observe(document.body, {
      attributes: true,
      attributeFilter: ["class"],
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
      initializeSVGThemes();
      watchThemeChanges();
    });
  } else {
    initializeSVGThemes();
    watchThemeChanges();
  }

  if (window.playground_text) {
    const themeToggle = document.getElementById("theme-toggle");
    if (themeToggle) {
      themeToggle.addEventListener("click", () => {
        setTimeout(initializeSVGThemes, 100);
      });
    }

    const themeChoices = document.querySelectorAll("#theme-list button");
    themeChoices.forEach((button) => {
      button.addEventListener("click", () => {
        setTimeout(initializeSVGThemes, 100);
      });
    });
  }

  window.updateAllSVGThemes = initializeSVGThemes;
})();
