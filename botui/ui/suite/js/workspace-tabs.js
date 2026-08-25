"use strict";
/* Workspace Tabs (#1168): persistent, renamable, browser-style tabs on top
   of the desktop. Each tab remembers which app windows belong to it; the
   desktop shows one tab's windows at a time (mission-control style), and
   the whole layout survives reloads via localStorage. Double-click a tab
   to rename it (floating input, no modal). */

const WorkspaceTabs = (() => {
  const STORAGE_KEY = "gb-workspace-tabs";
  const TABBAR_ID = "gb-workspace-tabbar";

  function load() {
    try {
      const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) || "null");
      if (parsed && Array.isArray(parsed.tabs) && parsed.tabs.length) return parsed;
    } catch (e) {
      /* fall through to default */
    }
    return { tabs: [{ id: "ws-" + Date.now(), name: "Main", windows: [] }], active: 0 };
  }

  function save(state) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch (e) {
      /* storage full/blocked — non-fatal */
    }
  }

  function currentWindows() {
    const wm = window.WindowManager;
    return (wm && wm.openWindows) || [];
  }

  function activeWindowIds() {
    return currentWindows().map((w) => w.id);
  }

  function render(state) {
    let bar = document.getElementById(TABBAR_ID);
    if (!bar) {
      bar = document.createElement("div");
      bar.id = TABBAR_ID;
      bar.className = "gb-workspace-tabbar";
      const shell = document.querySelector("#desktop, .desktop, #desktop3d");
      (shell || document.body).prepend(bar);
    }
    let html = "";
    state.tabs.forEach((tab, i) => {
      html +=
        '<div class="gb-ws-tab' + (i === state.active ? " active" : "") + '" data-i="' + i + '">' +
        '<span class="gb-ws-tab-name"></span>' +
        '<span class="gb-ws-tab-x" title="Close tab">\u00d7</span>' +
        "</div>";
    });
    html += '<button class="gb-ws-tab-add" title="New workspace tab">+</button>';
    bar.innerHTML = html;
    state.tabs.forEach((tab, i) => {
      const node = bar.querySelector('[data-i="' + i + '"]');
      if (!node) return;
      node.querySelector(".gb-ws-tab-name").textContent = tab.name;
    });
  }

  function showTab(state) {
    const ids = new Set((state.tabs[state.active] || { windows: [] }).windows);
    const wm = window.WindowManager;
    const all = currentWindows();
    all.forEach((w) => {
      const el = document.getElementById("window-" + w.id);
      if (!el) return;
      el.style.display = ids.has(w.id) ? "" : "none";
    });
    render(state);
  }

  function addTab(state) {
    const tab = { id: "ws-" + Date.now(), name: "Workspace " + (state.tabs.length + 1), windows: [] };
    state.tabs.push(tab);
    state.active = state.tabs.length - 1;
    save(state);
    showTab(state);
  }

  function closeTab(state, i) {
    if (state.tabs.length <= 1) return;
    state.tabs.splice(i, 1);
    if (state.active >= state.tabs.length) state.active = state.tabs.length - 1;
    save(state);
    showTab(state);
  }

  function renameTab(state, i) {
    const tab = state.tabs[i];
    const done = (name) => {
      if (name) {
        tab.name = String(name).trim() || tab.name;
        save(state);
        render(state);
      }
    };
    if (window.WindowManager && window.WindowManager.promptFloating) {
      window.WindowManager.promptFloating("Rename workspace", "Workspace name:", tab.name, done);
    } else {
      done(window.prompt("Workspace name:", tab.name));
    }
  }

  function bind(state) {
    const bar = document.getElementById(TABBAR_ID);
    if (!bar) return;
    bar.addEventListener("click", (e) => {
      const add = e.target.closest(".gb-ws-tab-add");
      if (add) return addTab(state);
      const x = e.target.closest(".gb-ws-tab-x");
      if (x) {
        const i = parseInt(x.closest(".gb-ws-tab").dataset.i, 10);
        return closeTab(state, i);
      }
      const tab = e.target.closest(".gb-ws-tab");
      if (tab) {
        const i = parseInt(tab.dataset.i, 10);
        if (i !== state.active) {
          state.active = i;
          save(state);
          showTab(state);
        }
      }
    });
    bar.addEventListener("dblclick", (e) => {
      const tab = e.target.closest(".gb-ws-tab");
      if (tab) renameTab(state, parseInt(tab.dataset.i, 10));
    });
    // Remember which tab new windows belong to, then keep the active set.
    const wm = window.WindowManager;
    if (wm && wm._origOpen && !wm.__wsTabsHooked) {
      wm.__wsTabsHooked = true;
      wm._origOpen = wm.open.bind(wm);
      wm.open = (...args) => {
        const result = wm._origOpen(...args);
        const tab = state.tabs[state.active];
        if (tab) {
          tab.windows = activeWindowIds();
          save(state);
        }
        return result;
      };
      const origClose = wm.close.bind(wm);
      wm.close = (id) => {
        const tab = state.tabs[state.active];
        if (tab) {
          tab.windows = tab.windows.filter((w) => w !== id);
          save(state);
        }
        return origClose(id);
      };
    }
  }

  function init() {
    if (window.__gbWsTabsStarted) return;
    window.__gbWsTabsStarted = true;
    const state = load();
    bind(state);
    // Debounced: capture window changes into the active tab.
    const capture = () => {
      const tab = state.tabs[state.active];
      if (tab) {
        tab.windows = activeWindowIds();
        save(state);
      }
    };
    // Window membership is captured by the wm.open/close hooks above;
    // these events are a belt-and-suspenders for windows created by other
    // launchers. No MutationObserver — subtree observation on <body> is the
    // performance trap that froze the desktop earlier.
    window.addEventListener("gb-window-opened", capture);
    window.addEventListener("gb-window-close", capture);
    showTab(state);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  return { init };
})();

window.WorkspaceTabs = WorkspaceTabs;
