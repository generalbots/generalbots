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
const defaultProduct = read("botserver/.product");
const chatPartial = read("botui/ui/suite/partials/chat.html");
const settingsHtml = read("botui/ui/suite/settings/index.html");
const settingsJs = read("botui/ui/suite/settings/settings-main.js");
const catalogTypes = read("botserver/src/apps/integration_catalog/types.rs");
const llmProviders = read("botserver/src/apps/integration_catalog/providers/llm.rs");
const cargoToml = read("botserver/Cargo.toml");
const pipelineExec = read("botserver/src/core/bot/pipeline/exec.rs");
const desktop = read("botui/ui/suite/desktop.html");
const taskbarCss = read("botui/ui/suite/css/desktop/taskbar.css");
const baseJs = read("botui/ui/suite/js/base.js");
const composerCss = read("botui/ui/suite/chat/chat-composer.css");
const windowManager = read("botui/ui/suite/js/window-manager.js");
const sidebarSections = read("botui/ui/suite/js/sidebar-sections.js");

// #1614/#1615 — the profile ships chat+drive plus the everyday tools the
// desktop edition deliberately keeps (clock, player, sheet, docs, …). Out of
// scope domains must never be advertised.
const productApps = ((product.match(/^apps=(.*)$/m) || [])[1] || "")
  .split(",")
  .map((s) => s.trim())
  .filter(Boolean);
check(productApps.indexOf("chat") !== -1 && productApps.indexOf("drive") !== -1,
  "desktop profile ships chat and drive");
check(["mail", "calendar", "people", "tickets", "billing", "meet", "social", "marketing", "hr", "integrations"]
  .every((app) => productApps.indexOf(app) === -1),
  "desktop profile ships no out-of-scope domain app");
check(/^hidden_apps=integrations,automations,research,sources\s*$/m.test(product),
  "desktop profile hides integrations, automations, research and sources");
check(baseJs.indexOf("function isAppHidden") !== -1, "hidden_apps has a client helper");
check(/!isAppHidden\(a\)/.test(baseJs), "enabled apps are filtered through hidden_apps");
check(/isAppHidden\(appName\)/.test(baseJs), "nav items are filtered through hidden_apps");
check(/if \(!Array\.isArray\(window\.productConfig\.apps\)\)/.test(baseJs),
  "a declared product app set is authoritative (no forced editor/sheet)");
check(/apps\.length === 0/.test(baseJs),
  "legacy core apps only fill an empty/default app set");
check(/window\.isAppHidden\(id\)/.test(windowManager),
  "the window manager refuses to open a hidden app");
check(/This application is not part of this edition/.test(windowManager),
  "hidden app shows a clear not-available message");

// #1596 — quick actions declare the app they launch and are filtered. The
// unconditional Vibe quick action is gone from the desktop sidebar entirely.
check(!/label:\s*"Vibe",\s*\n\s*run:/.test(sidebarSections),
  "no unconditional Vibe quick action remains");
check(/label:\s*tr\("chat-new-conversation"/.test(sidebarSections),
  "the sidebar keeps its chat quick action");

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

// #1626 — Research is not part of the default product, not even as a preview app.
const defaultPreview = (defaultProduct.match(/^preview_apps=(.*)$/m) || [])[1] || "";
check(defaultPreview.split(",").map((s) => s.trim()).indexOf("research") === -1,
  "default product does not advertise Research, even in Preview mode");

// #1629 — the chat composer has no create-a-free-account CTA.
check(chatPartial.indexOf("publicSignupCta") === -1, "chat no longer renders a signup CTA");
check(!/free account/i.test(chatPartial), "chat has no create-a-free-account copy");

// #1630 — the composer is a single modern capsule with scrollable chips.
check(chatPartial.indexOf("chat-composer.css") !== -1, "chat loads the composer stylesheet");
check(/border-radius:\s*28px/.test(composerCss), "composer is a rounded capsule");
check(/scroll-snap-type:\s*x proximity/.test(composerCss), "suggestion chips scroll horizontally");
check(/@media \(max-width: 640px\)/.test(composerCss), "composer has a narrow-screen layout");

// #1627 — one LLM provider list: Integrations owns it, Settings does not.
check(settingsHtml.indexOf("llm-providers-list") === -1, "settings no longer duplicates the LLM provider list");
check(/Category::Llm/.test(llmProviders), "LLM seeds use the dedicated LLM category");
check(/Llm => "LLM Providers"/.test(catalogTypes), "the catalog exposes an LLM Providers category");
check(/categoryOrder = \[[\s\S]*?"llm"/.test(read("botui/ui/suite/integrations/modules/01_state.js")),
  "integrations renders the LLM category first");

// #1628 — settings hides sections the chat+drive edition cannot serve.
// #1675 keeps Integrations visible on purpose: it hosts MCP server management.
check(/function applyEditionVisibility/.test(settingsJs), "settings has an edition visibility filter");
check(/\["billing", "vibe"\]\.forEach\(hideSettingsSection\)/.test(settingsJs),
  "billing and vibe are hidden in the slim edition");
check(!/\["billing", "vibe", "integrations"\]\.forEach/.test(settingsJs),
  "integrations stays visible for MCP management (#1675)");
check(/data-admin-only/.test(settingsJs), "administration entries are hidden in the slim edition");

// #1628 — navigation targets exist and the remaining switches are wired.
const navIds = [...settingsHtml.matchAll(/<a href="#([a-z-]+)"[^>]*class="nav-item/g)].map((m) => m[1]);
const missingSections = navIds.filter((id) => ["integrations", "about"].indexOf(id) === -1 && settingsHtml.indexOf('id="' + id + "-section\"") === -1);
check(navIds.length > 5, "settings has a navigation rail");
check(missingSections.length === 0, "every settings nav item targets a rendered section" + (missingSections.length ? ": " + missingSections.join(", ") : ""));
check(/window\.selectLanguage = function/.test(settingsJs), "language switch is wired");
check(/window\.setTheme = function/.test(settingsJs), "theme switch is wired");
check(/function bindToggles/.test(settingsJs) && /function loadSavedSettings/.test(settingsJs),
  "preference toggles load and persist");

// #1631 — the web-search switch reaches the pipeline in the slim build.
check(/web-search = \[[^\]]*dep:botresearch/.test(cargoToml), "web-search feature compiles botresearch");
check(/desktop-local = \[[\s\S]*?"web-search"/.test(cargoToml), "desktop-local enables web-search");
check(/any\(feature = "research", feature = "web-search"\)/.test(pipelineExec), "pipeline honours web-search without the Research app");

// #1680 — accessibility contracts that the Lighthouse gate checks. The widget
// pane needs a role for its aria-label, the tablist must contain only tabs,
// and the close control must meet the 24 px target size.
check(/pane\.setAttribute\("role", "region"\)/.test(read("botui/ui/suite/js/widget-registry.js")),
  "widget pane has a region role for its aria-label");
const tabsJs = read("botui/ui/suite/chat/modules/20_tabs.js");
const tabCss = read("botui/ui/suite/chat/chat.p6.css");
check(tabsJs.indexOf('role="tablist"') !== -1, "tabs render inside a tablist");
check(!/strip\.setAttribute\("role", "tablist"\)/.test(tabsJs),
  "the strip itself is not the tablist (buttons would be illegal children)");
check(/\.gb-tab-close\s*\{[^}]*min-width:\s*24px[^}]*min-height:\s*24px/s.test(tabCss),
  "tab close target is at least 24x24 px");

console.log(`desktop_surfaces: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
