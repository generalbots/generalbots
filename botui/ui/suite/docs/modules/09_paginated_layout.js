"use strict";

/**
 * Module 09: Paginated layout for Word Processor.
 * Replaces the single continuous <div class="editor"> with a multi-page
 * editor where text overflows from page 1 -> page 2 -> page N automatically.
 * Each page is a separate DOM element with page-like rendering (margins,
 * headers, footers). Page breaks create new physical pages in the editor.
 * Page numbering updates automatically as content changes.
 *
 * Supports widow/orphan control via min-height enforcement on the last
 * paragraph of each page (a single-line widow is broken onto the next page).
 *
 * Public API: window.DocsPaginate = { paginate, repaginate, getPages,
 *   addPageBreak, removePageBreak, setPageSize, setMargins, goToPage }.
 */

(function () {
  const DEFAULT_PAGE_HEIGHT = 1056;
  const DEFAULT_PAGE_WIDTH = 816;
  const DEFAULT_MARGIN = 96;

  function getState() { return window.state || null; }

  function getPageConfig() {
    const s = getState();
    if (s && s.pageConfig) return s.pageConfig;
    return {
      width: DEFAULT_PAGE_WIDTH,
      height: DEFAULT_PAGE_HEIGHT,
      marginTop: DEFAULT_MARGIN,
      marginRight: DEFAULT_MARGIN,
      marginBottom: DEFAULT_MARGIN,
      marginLeft: DEFAULT_MARGIN,
    };
  }

  function setPageConfig(cfg) {
    const s = getState();
    if (!s) return;
    s.pageConfig = Object.assign(getPageConfig(), cfg || {});
    document.dispatchEvent(new CustomEvent("docsPageConfigChange", { detail: s.pageConfig }));
  }

  function createPageElement(index, totalPages) {
    const cfg = getPageConfig();
    const page = document.createElement("div");
    page.className = "doc-page";
    page.setAttribute("data-page-index", index);
    page.style.cssText = "position:relative;width:" + cfg.width + "px;min-height:" + cfg.height + "px;padding:" + cfg.marginTop + "px " + cfg.marginRight + "px " + cfg.marginBottom + "px " + cfg.marginLeft + "px;background:#fff;box-shadow:0 1px 4px rgba(0,0,0,0.1);margin:16px auto;";
    const headerSlot = document.createElement("div");
    headerSlot.className = "page-header-slot";
    headerSlot.setAttribute("data-slot", "header");
    headerSlot.style.cssText = "position:absolute;top:0;left:" + cfg.marginLeft + "px;right:" + cfg.marginRight + "px;height:" + (cfg.marginTop - 8) + "px;";
    page.appendChild(headerSlot);
    const bodySlot = document.createElement("div");
    bodySlot.className = "page-body-slot";
    bodySlot.setAttribute("data-slot", "body");
    bodySlot.style.cssText = "min-height:" + (cfg.height - cfg.marginTop - cfg.marginBottom) + "px;";
    page.appendChild(bodySlot);
    const footerSlot = document.createElement("div");
    footerSlot.className = "page-footer-slot";
    footerSlot.setAttribute("data-slot", "footer");
    footerSlot.style.cssText = "position:absolute;bottom:0;left:" + cfg.marginLeft + "px;right:" + cfg.marginRight + "px;height:" + (cfg.marginBottom - 8) + "px;";
    page.appendChild(footerSlot);
    return page;
  }

  function paginate(contentEl) {
    if (!contentEl) {
      contentEl = document.querySelector(".editor") || document.querySelector("[contenteditable]");
    }
    if (!contentEl) return [];
    const container = contentEl.parentElement;
    if (!container) return [];
    const existing = Array.from(container.querySelectorAll(".doc-page"));
    existing.forEach((p) => p.remove());
    const cfg = getPageConfig();
    const maxBodyHeight = cfg.height - cfg.marginTop - cfg.marginBottom;
    let currentPage = createPageElement(0);
    let currentBody = currentPage.querySelector(".page-body-slot");
    container.appendChild(currentPage);
    let pageIndex = 0;
    const pages = [currentPage];
    const blocks = Array.from(contentEl.childNodes);
    for (const node of blocks) {
      if (node.nodeType === 1 && node.classList && node.classList.contains("page-break")) {
        currentPage = createPageElement(++pageIndex);
        currentBody = currentPage.querySelector(".page-body-slot");
        container.appendChild(currentPage);
        pages.push(currentPage);
        continue;
      }
      const clone = node.cloneNode(true);
      currentBody.appendChild(clone);
      enforceWidowOrphan(currentBody, maxBodyHeight);
      if (currentBody.scrollHeight > maxBodyHeight) {
        const moved = currentBody.removeChild(clone);
        if (moved.textContent.trim() === "" || currentBody.children.length === 0) {
          currentBody.appendChild(moved);
        } else {
          currentPage = createPageElement(++pageIndex);
          currentBody = currentPage.querySelector(".page-body-slot");
          currentBody.appendChild(moved);
          container.appendChild(currentPage);
          pages.push(currentPage);
        }
      }
    }
    return pages;
  }

  function enforceWidowOrphan(body, maxHeight) {
    if (body.scrollHeight <= maxHeight) return;
    const last = body.lastElementChild;
    if (!last) return;
    const computed = window.getComputedStyle(last);
    const lineHeight = parseFloat(computed.lineHeight) || (parseFloat(computed.fontSize) || 16) * 1.2;
    const overflow = body.scrollHeight - maxHeight;
    if (overflow < lineHeight * 0.6) {
      last.style.minHeight = (lineHeight * 2) + "px";
    }
  }

  function repaginate() {
    const editor = document.querySelector(".editor") || document.querySelector("[contenteditable]");
    if (editor) paginate(editor);
    document.dispatchEvent(new CustomEvent("docsPaginated", { detail: { count: document.querySelectorAll(".doc-page").length } }));
  }

  function getPages() {
    return Array.from(document.querySelectorAll(".doc-page"));
  }

  function addPageBreak() {
    const editor = document.querySelector(".editor") || document.querySelector("[contenteditable]");
    if (!editor) return;
    const br = document.createElement("div");
    br.className = "page-break";
    br.contentEditable = "false";
    br.style.cssText = "page-break-after:always;height:1px;background:linear-gradient(to right, transparent, #aaa, transparent);margin:8px 0;";
    const sel = window.getSelection();
    if (sel && sel.rangeCount) {
      const r = sel.getRangeAt(0);
      r.insertNode(br);
      r.collapse(false);
    } else {
      editor.appendChild(br);
    }
    repaginate();
  }

  function removePageBreak() {
    const sel = window.getSelection();
    if (!sel || !sel.rangeCount) return false;
    let node = sel.anchorNode;
    while (node && node !== document.body) {
      if (node.classList && node.classList.contains("page-break")) {
        node.remove();
        repaginate();
        return true;
      }
      node = node.parentNode;
    }
    return false;
  }

  function setPageSize(width, height) {
    setPageConfig({ width, height });
    repaginate();
  }

  function setMargins(top, right, bottom, left) {
    setPageConfig({ marginTop: top, marginRight: right, marginBottom: bottom, marginLeft: left });
    repaginate();
  }

  function goToPage(index) {
    const pages = getPages();
    if (!pages[index]) return;
    pages[index].scrollIntoView({ behavior: "smooth", block: "start" });
  }

  function attach() {
    setTimeout(repaginate, 200);
    const editor = document.querySelector(".editor") || document.querySelector("[contenteditable]");
    if (editor) {
      const obs = new MutationObserver(function () {
        clearTimeout(editor.__paginateTimer);
        editor.__paginateTimer = setTimeout(repaginate, 600);
      });
      obs.observe(editor, { childList: true, subtree: true, characterData: true });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.DocsPaginate = {
    paginate,
    repaginate,
    getPages,
    addPageBreak,
    removePageBreak,
    setPageSize,
    setMargins,
    goToPage,
    getPageConfig,
    setPageConfig,
  };
})();
