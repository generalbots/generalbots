"use strict";
/* Desktop surface regression tests (#1596, #1614, #1615, #1617, #1618, #1619).
 * Zero-dependency static assertions over the shipped desktop files
 * (run: node tests/desktop_surfaces.test.js)
 *
 * These are the acceptance conditions that are cheap to regress: a placeholder
 * string, a tray button returning, a hardcoded quick action, or the blank dock
 * pill coming back with the stylesheet.
 */

const fs = require("fs");
const path = require("path");

const ROOT = path.join(__dirname, "..", "..");
const read = (rel) => fs.readFileSync(path.join(ROOT, rel), "utf8");

let passed = 0;
let failed = 0;
function check(condition, label) {
  if (condition) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

const product = read("botserver/.product.desktop-local");
const desktop = read("botui/ui/suite/desktop.html");
const taskbarCss = read("botui/ui/suite/css/desktop/taskbar.css");
const baseJs = read("botui/ui/suite/js/base.js");
const windowManager = read("botui/ui/suite/js/window-manager.js");
const sidebarSections = read("botui/ui/suite/js/sidebar-sections.js");

// #1614/#1615 — the profile ships chat+drive and hides the out-of-scope apps.
check(/^apps=chat,drive\s*$/m.test(product), "desktop profile ships only chat,drive");
check(/^hidden_apps=integrations,automations\s*$/m.test(product),
  "desktop profile hides integrations and automations");
check(baseJs.indexOf("function isAppHidden") !== -1, "hidden_apps has a client helper");
check(/!isAppHidden\(a\)/.test(baseJs), "enabled apps are filtered through hidden_apps");
check(/isAppHidden\(appName\)/.test(baseJs), "nav items are filtered through hidden_apps");
check(/window\.isAppHidden\(id\)/.test(windowManager),
  "the window manager refuses to open a hidden app");
check(/This application is not part of this edition/.test(windowManager),
  "hidden app shows a clear not-available message");

// #1596 — quick actions declare the app they launch and are filtered.
check(/app:\s*"vibe"/.test(sidebarSections), "Vibe quick action declares its app");
check(/isAppEnabled\(action\.app\)/.test(sidebarSections),
  "quick actions are filtered by the product app set");
check(!/label:\s*"Vibe",\s*\n\s*run:/.test(sidebarSections),
  "no unconditional Vibe quick action remains");

// #1619 — search placeholder and the disabled-search removal.
check(/placeholder="Search\.\.\."/.test(desktop), "search placeholder reads 'Search...'");
check(!/Search apps, people, products/.test(desktop), "old placeholder is gone");
check(/search_enabled === false/.test(desktop) &&
  /taskbarSearchWrap/.test(desktop), "search field is removed when search is disabled");

// #1617 — the three tray buttons are gone; notifications and lock remain.
const traySection = desktop.slice(desktop.indexOf('class="taskbar-tray"'), desktop.indexOf("</footer>"));
check(traySection.indexOf("trayControlCenter") === -1, "Control Center tray button is gone");
check(traySection.indexOf("trayMissionControl") === -1, "Mission Control tray button is gone");
check(traySection.indexOf("trayVirtualDesktops") === -1, "Virtual Desktops tray button is gone");
check(traySection.indexOf("trayNotifications") !== -1, "notifications button remains");
check(traySection.indexOf("trayLock") !== -1, "lock button remains");

// #1618 — the dock hugs its content; no reserved 420px pill.
const taskbarRules = taskbarCss.replace(/\/\*[\s\S]*?\*\//g, "");
check(/taskbar-shell/.test(desktop), "taskbar has a real content wrapper");
check(!/min-width:\s*420px/.test(taskbarRules), "no 420px reserved dock width");
check(/\.toolbar-v15 \.taskbar-shell/.test(taskbarRules), "dock wrapper has its own rule");

// The taskbar document has no console-erroring dead handlers for the removed
// buttons (their modules stay loaded for keyboard shortcuts only).
check(!/toggleControlCenter|toggleMissionControl|toggleVirtualDesktops/.test(desktop),
  "no handlers reference the removed tray buttons");

console.log(`desktop_surfaces: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
