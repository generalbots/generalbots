/**
 * OS app requirement guard — issue #2479.
 * Run with: node botui/ui/suite/tests/webtest-os-app-requires.js
 */
"use strict";
var fs = require("fs");
var path = require("path");
var vm = require("vm");
var root = path.resolve(__dirname, "..", "..", "..", "..");
var file = path.join(root, "botserver", "src", "apps", "integration_catalog", "mod.rs");
var requiresFile = path.join(root, "botserver", "src", "apps", "registry", "requires.rs");
var src = fs.readFileSync(requiresFile, "utf8");
var pass = 0, fail = 0;
function check(name, ok) { if (ok) { pass++; console.log("[PASS] " + name); } else { fail++; console.log("[FAIL] " + name); } }
check("AppDefinition has requires field", /pub requires:\s*Option<AppRequirement>/.test(fs.readFileSync(path.join(root, "botserver", "src", "apps", "registry", "mod.rs"), "utf8")));
check("mail requirement declared", /"mail"/.test(src));
check("meet requirement declared", /"meet"/.test(src));
check("is_available has a provisioned branch", /\(Some\(requirement\),\s*Some\(components\)\)/.test(src));
check("unavailable_reason names the component", /format!\("Needs/.test(src));
var shell = path.join(root, "botui", "ui", "suite", "js", "sidebar-apps.js");
var start = path.join(root, "botui", "ui", "suite", "js", "window-manager", "05_start_menu.js");
check("sidebar renders unavailable reason", fs.readFileSync(shell, "utf8").indexOf("unavailable_reason") >= 0);
check("start menu renders unavailable reason", fs.readFileSync(start, "utf8").indexOf("unavailable_reason") >= 0);
// Audit every app in .product.desktop-local apps= against the registry.
var product = fs.readFileSync(path.join(root, "botserver", ".product.desktop-local"), "utf8");
var appsLine = (product.match(/^apps=([^\n]*)/m) || [])[1] || "";
var apps = appsLine.split(",").map(function (s) { return s.trim(); }).filter(Boolean);
var registrySrc = fs.readdirSync(path.join(root, "botserver", "src", "apps", "registry"))
  .filter(function (f) { return /^entries.*\.rs$/.test(f); })
  .map(function (f) { return fs.readFileSync(path.join(root, "botserver", "src", "apps", "registry", f), "utf8"); })
  .join("\n");
check("desktop-local apps= is non-empty", apps.length > 0);
apps.forEach(function (id) {
  check("app " + id + " has a registry URL", registrySrc.indexOf('"' + id + '"') >= 0 || registrySrc.indexOf("id: \"" + id + "\"") >= 0);
});
console.log("TOTAL: " + (pass+fail) + " | PASS: " + pass + " | FAIL: " + fail);
process.exitCode = fail === 0 ? 0 : 1;
