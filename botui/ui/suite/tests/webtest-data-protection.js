/**
 * Static safety guard for #2470 — restore.sh must keep BOTH fixes the driver
 * made, so a future regression fails CI instead of silently returning:
 *   1. validate_path_arg() exists, is wired into --snapshot/--archive parsing
 *      and rejects every shell metacharacter (defense in depth for run_sh).
 *   2. restore_archive() takes a btrfs safety snapshot BEFORE the
 *      destructive `gpg ... | tar` decrypt pipeline writes over live data.
 *
 * Run with: node botui/ui/suite/tests/webtest-data-protection.js
 * Optional: GB_RESTORE_PATH=/path/to/restore.sh to point at a modified copy
 * (used by the non-vacuity probe). This is a pure source audit: the only child
 * process is `bash -n` (read-only syntax check) — no restore is executed.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var child = require("child_process");

var root = path.resolve(__dirname, "..", "..", "..", "..");
var target = process.env.GB_RESTORE_PATH || path.join(
    root, "desktop", "os", "data-protection", "extra", "usr", "local",
    "libexec", "gb-data", "restore.sh"
);

var pass = 0, fail = 0;
function check(name, ok) {
    if (ok) { pass++; console.log("[PASS] " + name); }
    else { fail++; console.log("[FAIL] " + name); }
}
function finish() {
    console.log("TOTAL: " + (pass + fail) + " | PASS: " + pass + " | FAIL: " + fail);
    process.exitCode = fail === 0 ? 0 : 1;
}

var src;
try {
    src = fs.readFileSync(target, "utf8");
    check("restore.sh readable: " + path.relative(root, target), true);
} catch (e) {
    check("restore.sh readable: " + target, false);
    finish();
    process.exit(1);
}

// Guard 1 — path validation is defined and called on both destructive args.
check("validate_path_arg() function is defined", /validate_path_arg\(\)\s*\{/.test(src));
check("--snapshot parsing calls validate_path_arg", /validate_path_arg\s+"--snapshot"/.test(src));
check("--archive parsing calls validate_path_arg", /validate_path_arg\s+"--archive"/.test(src));

// The metacharacter case list must be inside validate_path_arg(), not a comment.
var vStart = src.indexOf("validate_path_arg()");
var vEnd = vStart >= 0 ? src.indexOf("\n}", vStart) : -1;
var vBody = vStart >= 0 ? src.slice(vStart, vEnd >= 0 ? vEnd : src.length) : "";
[
    ["single quote", /\*"'"\*/],
    ["double quote", /\*'"'\*/],
    ["dollar", /\*'\$'\*/],
    ["backtick", /\*'`'\*/],
    ["semicolon", /\*';'\*/],
    ["pipe", /\*'\|'\*/],
    ["ampersand", /\*'&'\*/],
    ["newline", /\$'\\n'/]
].forEach(function (entry) {
    check("metacharacter list covers " + entry[0], entry[1].test(vBody));
});

// Guard 2 — syntax must stay valid after the edits, read-only check.
try {
    child.execFileSync("bash", ["-n", target], { stdio: "pipe" });
    check("bash -n syntax check", true);
} catch (e) {
    check("bash -n syntax check", false);
}

// Guard 3 — safety snapshot strictly precedes the destructive decrypt write.
var aStart = src.indexOf("restore_archive()");
var aEnd = aStart >= 0 ? src.indexOf("\nmain()", aStart) : -1;
var aBody = aStart >= 0 ? src.slice(aStart, aEnd >= 0 ? aEnd : src.length) : "";
var snapAt = aBody.indexOf("btrfs subvolume snapshot");
var gpgAt = aBody.search(/gpg[^\n]*\|\s*tar/);
check("restore_archive() takes a btrfs safety snapshot", snapAt >= 0);
check("restore_archive() has a gpg | tar decrypt pipeline", gpgAt >= 0);
check("safety snapshot precedes the gpg | tar write", snapAt >= 0 && gpgAt >= 0 && snapAt < gpgAt);

finish();
