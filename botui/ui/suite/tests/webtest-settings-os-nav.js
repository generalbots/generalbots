/**
 * Settings nav-registry guard — issue #2466.
 *
 * Run with: node botui/ui/suite/tests/webtest-settings-os-nav.js
 *
 * Loads the shipped nav-registry.js in a VM sandbox (no browser needed) and
 * asserts that the eight OS appliance nodes are declared with the OS edition,
 * shell control and admin scope, and are visible only in the OS edition.
 * This is the checked-in guard the #2466 acceptance criteria ask for.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var FILE = path.resolve(__dirname, "..", "settings", "nav-registry.js");
var NODES = [
  "os-network", "os-power", "os-display", "os-sound",
  "os-keyboard", "os-storage-reset", "os-users", "os-diagnostics"
];
var PASS = 0;
var FAIL = 0;

function check(name, condition, detail) {
  if (condition) {
    PASS++;
    console.log("[PASS] " + name);
  } else {
    FAIL++;
    console.log("[FAIL] " + name + (detail ? " — " + detail : ""));
  }
}

var source = fs.readFileSync(FILE, "utf8");
var sandbox = { window: {}, localStorage: null, console: console };
vm.createContext(sandbox);
vm.runInContext(source, sandbox, { filename: FILE });
var registry = sandbox.window.GBNavRegistry;

check("GBNavRegistry is published", !!registry);
check("nodes table is published", !!(registry && Array.isArray(registry.nodes)));

if (registry) {
  NODES.forEach(function (id) {
    var node = registry.get(id);
    check("node " + id + " exists", !!node);
    if (node) {
      check("node " + id + " is OS-only", JSON.stringify(node.editions) === JSON.stringify(["os"]),
        JSON.stringify(node.editions));
      check("node " + id + " uses shell control", node.control === "shell", String(node.control));
      check("node " + id + " is admin-scoped", node.scope === "admin", String(node.scope));
      check("node " + id + " visible in os edition", registry.isVisible(id, { edition: "os", isAdmin: true }) === true);
      check("node " + id + " hidden in cloud edition", registry.isVisible(id, { edition: "cloud", isAdmin: true }) === false);
      check("node " + id + " hidden in desktop edition", registry.isVisible(id, { edition: "desktop", isAdmin: true }) === false);
    }
  });
}

console.log("=".repeat(64));
console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
console.log("=".repeat(64));
process.exitCode = FAIL === 0 ? 0 : 1;
