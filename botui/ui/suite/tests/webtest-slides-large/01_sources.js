/**
 * Shipped-source inventory and scan helpers for #2529.
 *
 * Only shipped sources are read:
 *   - botui/ui/suite/slides/modules/*.js  (tests/ and sibling apps excluded)
 *   - botslides save/export Rust          (import, handlers, routes, storage,
 *                                          fragments, upload policy)
 * No file is written back: this is a measurement harness, not a patcher.
 */
"use strict";

var h = require("./00_harness");

var MODULE_DIR = h.path.join(h.SUITE_DIR, "slides", "modules");

var SLIDES_MODULES = h.fs
  .readdirSync(MODULE_DIR)
  .filter(function (name) { return name.slice(-3) === ".js"; })
  .sort()
  .map(function (name) { return "slides/modules/" + name; });

var RUST = {
  import: "botserver/crates/botslides/src/pptx/import.rs",
  elementOps: "botserver/crates/botslides/src/handlers/slides/element_ops.rs",
  handlers: "botserver/crates/botslides/src/handlers/slides/mod.rs",
  routes: "botserver/crates/botslides/src/routes.rs",
  storage: "botserver/crates/botslides/src/storage.rs",
  getBridge: "botserver/crates/botslides/src/ui_fragments/get_bridge.rs",
  sidebars: "botserver/crates/botslides/src/ui_fragments/sidebars.rs",
  panels: "botserver/crates/botslides/src/ui_fragments/panels.rs",
  uploadPolicy: "botserver/crates/botsecurity-core/src/upload_policy.rs",
  uploadTransport: "botserver/crates/botsecurity-core/src/upload_transport.rs",
};

var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

function readModule(name) {
  return h.readFile("slides/modules/" + name);
}

function rust(key) {
  return h.readRepo(RUST[key]);
}

function allJs() {
  return SLIDES_MODULES.map(function (rel) { return h.readFile(rel); }).join("\n");
}

function count(text, needle) {
  return text.split(needle).length - 1;
}

function has(text, needle) {
  return text.indexOf(needle) >= 0;
}

function modulePathByName(name) {
  return "slides/modules/" + name;
}

module.exports = {
  SLIDES_MODULES,
  RUST,
  LOCALES,
  readModule,
  rust,
  allJs,
  count,
  has,
  modulePathByName,
};
