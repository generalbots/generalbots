#!/usr/bin/env node
/**
 * Cross-check the suite app manifests.
 *
 * Compares: APPS_REGISTRY ids (window manager) <-> .product apps=/preview_apps=/hidden_apps=
 *           <-> hxGet file exists under botui/ui.
 *
 * Exit 0 when consistent, 1 with a readable diff on mismatch.
 */
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const registryPath = "botui/ui/suite/js/window-manager/01_registry.js";
const productPath = "botserver/.product";

function read(file) {
  return fs.readFileSync(path.join(root, file), "utf8");
}

function parseRegistry(text) {
  const apps = [];
  const re = /\{\s*id:\s*"([^"]+)"([\s\S]*?)(?:\n\s*\}|},)/g;
  let m;
  while ((m = re.exec(text)) !== null) {
    const block = m[0];
    const app = { id: m[1] };
    const t = /title:\s*"([^"]*)"/.exec(block);
    if (t) app.title = t[1];
    const h = /hxGet:\s*"([^"]*)"/.exec(block);
    if (h) app.hxGet = h[1];
    const c = /category:\s*"([^"]*)"/.exec(block);
    if (c) app.category = c[1];
    apps.push(app);
  }
  return apps;
}

function parseProductList(text, key) {
  const m = new RegExp("^" + key + "=(.*)$", "m").exec(text);
  if (!m) return [];
  return m[1]
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

function hxGetFile(hxGet) {
  if (!hxGet) return null;
  const p = hxGet.split("?")[0];
  if (p.startsWith("/")) return "botui/ui" + p;
  return p;
}

// Core surfaces are always available without being listed in apps= (.product comment).
const CORE_IDS = new Set(["about", "admin", "settings", "auth"]);

const registry = parseRegistry(read(registryPath));
const regIds = registry.map((a) => a.id);
const regSet = new Set(regIds);

const product = read(productPath);
const apps = parseProductList(product, "apps");
const preview = parseProductList(product, "preview_apps");
const hidden = parseProductList(product, "hidden_apps");
const appsSet = new Set(apps);
const previewSet = new Set(preview);
const hiddenSet = new Set(hidden);
const listedSet = new Set([...apps, ...preview, ...hidden]);

const problems = [];
const notes = [];

function add(msg) {
  problems.push(msg);
}

const dup = regIds.filter((id, i) => regIds.indexOf(id) !== i);
if (dup.length) add(`Duplicate registry ids: ${[...new Set(dup)].join(", ")}`);

// Registered but not classified in .product (and not core)
const unclassified = regIds.filter((id) => !listedSet.has(id) && !CORE_IDS.has(id));
if (unclassified.length) {
  add(
    "Registered but not classified in .product (ship in apps=, preview_apps=, or hidden_apps=):\n  " +
      unclassified.join(", ")
  );
}

// Listed in apps/preview/hidden but no registry tile
const appsNoReg = apps.filter((id) => !regSet.has(id));
if (appsNoReg.length) {
  add(
    "In .product apps= but missing from APPS_REGISTRY (no launcher tile):\n  " +
      appsNoReg.join(", ")
  );
}
const previewNoReg = preview.filter((id) => !regSet.has(id));
if (previewNoReg.length) {
  add(
    "In .product preview_apps= but missing from APPS_REGISTRY:\n  " +
      previewNoReg.join(", ")
  );
}
const hiddenNoReg = hidden.filter((id) => !regSet.has(id));
if (hiddenNoReg.length) {
  add(
    "In .product hidden_apps= but missing from APPS_REGISTRY:\n  " +
      hiddenNoReg.join(", ")
  );
}

// hxGet file exists
for (const app of registry) {
  if (!app.hxGet) {
    add(`Registry app '${app.id}' has no hxGet`);
    continue;
  }
  const file = hxGetFile(app.hxGet);
  if (file && !fs.existsSync(path.join(root, file))) {
    add(`Registry app '${app.id}' hxGet target missing: ${app.hxGet} (expected ${file})`);
  }
}

// apps listed more than once across sections
const allListedArr = [...apps, ...preview, ...hidden];
const multi = allListedArr.filter((id, i) => allListedArr.indexOf(id) !== i);
if (multi.length) {
  add(`Ids listed in more than one of apps/preview_apps/hidden_apps: ${[...new Set(multi)].join(", ")}`);
}

if (problems.length) {
  console.error("App manifest check FAILED");
  console.error("");
  for (const p of problems) {
    console.error("- " + p);
  }
  console.error("");
  console.error(`Registry: ${registryPath} (${regIds.length} apps)`);
  console.error(`Product:  ${productPath} (apps=${apps.length}, preview=${preview.length}, hidden=${hidden.length})`);
  process.exit(1);
}

console.log("App manifest check OK");
console.log(`Registry apps: ${regIds.length}`);
console.log(`Product: apps=${apps.length} preview=${preview.length} hidden=${hidden.length}`);
for (const n of notes) console.log("note: " + n);
