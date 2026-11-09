"use strict";
/* Chat attachment cleanup — zero-dependency unit test
 * (run: node tests/chat_file_list_cleanup.test.js)
 *
 * #2008 — the raw `vm_exec` result bubble carries a `**Files**` heading plus a
 * bullet list of vmfile links. renderVmFileLinks turns each link into a card,
 * vmGroupAttachments moves the cards into the output folder, and the leftover
 * `<ul><li></li></ul>` + heading were rendered as empty bullets. This test
 * loads `vmDropOrphanFileList` from 05_folder.js against a minimal DOM shim
 * and asserts the post-move cleanup. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const FOLDER_MODULE = path.join(
  __dirname,
  "..",
  "ui",
  "suite",
  "chat",
  "modules",
  "vm_attachments",
  "05_folder.js"
);

let passed = 0;
let failed = 0;

function assertEqual(actual, expected, label) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a === e) {
    passed++;
  } else {
    failed++;
    console.error("FAIL " + label + "\n  expected: " + e + "\n  actual:   " + a);
  }
}

/* ── minimal DOM ─────────────────────────────────────────────────────────── */

class DomNode {
  constructor(tagName, className) {
    this.tagName = String(tagName || "div").toUpperCase();
    this.className = className || "";
    this.children = [];
    this.parentElement = null;
  }

  append(...nodes) {
    for (const child of nodes) {
      child.parentElement = this;
      this.children.push(child);
    }
    return this;
  }

  get textContent() {
    return this.children.map((child) => child.textContent).join("");
  }

  remove() {
    if (!this.parentElement) return;
    const index = this.parentElement.children.indexOf(this);
    if (index !== -1) this.parentElement.children.splice(index, 1);
    this.parentElement = null;
  }

  get previousElementSibling() {
    if (!this.parentElement) return null;
    const index = this.parentElement.children.indexOf(this);
    return index > 0 ? this.parentElement.children[index - 1] : null;
  }

  matches(selector) {
    return selector
      .split(",")
      .map((part) => part.trim())
      .some((part) => {
        if (part.startsWith(".")) {
          return (" " + this.className + " ").indexOf(" " + part.slice(1) + " ") !== -1;
        }
        return this.tagName === part.toUpperCase();
      });
  }

  querySelectorAll(selector) {
    const found = [];
    const visit = (node) => {
      for (const child of node.children) {
        if (child.matches(selector)) found.push(child);
        visit(child);
      }
    };
    visit(this);
    return found;
  }

  querySelector(selector) {
    const all = this.querySelectorAll(selector);
    return all.length ? all[0] : null;
  }
}

class DomText extends DomNode {
  constructor(value) {
    super("#text");
    this.value = String(value);
  }

  get textContent() {
    return this.value;
  }

  matches() {
    return false;
  }
}

function text(value) {
  // marked wraps a bare line in a paragraph, so the shim does the same.
  return new DomNode("p").append(new DomText(value));
}

/* ── load the module ─────────────────────────────────────────────────────── */

function loadFolderModule() {
  const sandbox = {
    window: {},
    document: {
      createElement: (tag) => new DomNode(tag),
      querySelector: () => null,
      querySelectorAll: () => [],
    },
    console,
  };
  sandbox.window = sandbox;
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(FOLDER_MODULE, "utf8"), sandbox, {
    filename: FOLDER_MODULE,
  });
  return sandbox;
}

function messageWithFiles(heading) {
  const content = new DomNode("div", "message-content bot-message");
  content.append(text(heading));
  const list = new DomNode("ul");
  // Cards have already moved into the folder grid by the time cleanup runs,
  // so every list item is empty.
  list.append(new DomNode("li"), new DomNode("li"));
  content.append(list);
  return { content, list };
}

/* ── cases ───────────────────────────────────────────────────────────────── */

const sandbox = loadFolderModule();
if (typeof sandbox.vmDropOrphanFileList !== "function") {
  console.error("FAIL vmDropOrphanFileList is not exported by 05_folder.js");
  process.exit(1);
}

// 1. English "Files" heading + emptied bullets disappear.
{
  const fixture = messageWithFiles("Files");
  sandbox.vmDropOrphanFileList(fixture.content);
  assertEqual(fixture.content.querySelectorAll("li").length, 0, "empty bullets removed");
  assertEqual(fixture.content.querySelectorAll("ul").length, 0, "empty list removed");
  assertEqual(fixture.content.querySelectorAll("p").length, 0, "Files heading removed");
}

// 2. Portuguese heading is removed too.
{
  const fixture = messageWithFiles("Arquivos");
  sandbox.vmDropOrphanFileList(fixture.content);
  assertEqual(fixture.content.querySelectorAll("ul").length, 0, "Arquivos list removed");
  assertEqual(fixture.content.querySelectorAll("p").length, 0, "Arquivos heading removed");
}

// 3. A bullet list with real text is not touched.
{
  const content = new DomNode("div", "message-content bot-message");
  content.append(text("Files"));
  const list = new DomNode("ul");
  const item = new DomNode("li");
  item.append(text("report.pdf"));
  list.append(item);
  content.append(list);
  sandbox.vmDropOrphanFileList(content);
  assertEqual(content.querySelectorAll("li").length, 1, "real bullet kept");
  assertEqual(content.querySelectorAll("ul").length, 1, "real list kept");
  assertEqual(
    content.children.filter((child) => child.tagName === "P").length,
    1,
    "heading kept with a real list"
  );
}

// 4. A `li` that still holds an ungrouped card is left for hydration.
{
  const content = new DomNode("div", "message-content bot-message");
  const list = new DomNode("ul");
  const item = new DomNode("li");
  item.append(new DomNode("span", "gb-attachment"));
  list.append(item);
  content.append(list);
  sandbox.vmDropOrphanFileList(content);
  assertEqual(content.querySelectorAll("li").length, 1, "card item kept");
}

// 5. Cleanup is a no-op on content without a list.
{
  const content = new DomNode("div", "message-content bot-message");
  content.append(text("All good."));
  sandbox.vmDropOrphanFileList(content);
  assertEqual(content.querySelectorAll("p").length, 1, "plain paragraph kept");
}

console.log(passed + " passed, " + failed + " failed");
process.exit(failed ? 1 : 0);
