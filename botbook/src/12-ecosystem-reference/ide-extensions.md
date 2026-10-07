# IDEs 🟡 BETA

General Bots ships its own editing surfaces in the Suite, inside the desktop
shell. There is no external editor extension, no LSP server and no debugger.
The two surfaces below are the whole surface.

## Editor

The Editor app (`botui/ui/suite/editor.html`) is a browser editor built on
Monaco, registered in `botserver/src/apps/registry.rs` under id `editor` with
the description "Code and file editor with git integration."

It edits files held in Drive rather than a local checkout, and it exposes the
git operations of the `botgit` crate as REST routes:

| Method | Route | Purpose |
|--------|-------|---------|
| GET | `/api/git/status` | Working tree status |
| GET | `/api/git/diff/:file` | Diff for one file |
| POST | `/api/git/commit` | Commit the staged changes |
| POST | `/api/git/push` | Push commits |
| POST | `/api/git/pull` | Pull remote changes |
| GET | `/api/git/tree` | File listing with status |

The tree route is the only place a file list is assembled, and it comes from
`git ls-files` plus `git status`; it rejects `..` traversal.

## Terminal

The Terminal app (`botui/ui/suite/terminal/terminal.html`) is a system shell in
the browser, built on xterm.js. It is registered under id `terminal` with the
description "System terminal in the browser."

## External editors

Any external editor works for editing BASIC scripts, because a `.bas` file is
plain text. The repository ships no editor package, grammar or language server
for it, so syntax highlighting and completion must be configured in the editor
itself. The keyword reference for writing such a grammar is in
[the BASIC chapter](../04-basic-scripting/basics.md).

## Common Features Across All Editors

### Snippets

All editor integrations include useful snippets to speed up development. The tool definition snippet creates parameter blocks:

```basic
PARAM ${name} AS ${type} LIKE "${example}" DESCRIPTION "${description}"
DESCRIPTION "${tool_description}"
${body}
```

The dialog flow snippet sets up conversation structures:

```basic
TALK "${greeting}"
HEAR response
IF response = "${expected}" THEN
    ${action}
END IF
```

The knowledge base snippet configures KB access:

```basic
USE KB "${collection}"
# System AI now has access to the KB
TALK "How can I help you with ${collection}?"
CLEAR KB
```

### File Associations

| Extension | File Type | Purpose |
|-----------|-----------|---------|
| `.bas` | BASIC Script | Dialog logic |
| `.gbdialog` | Dialog Package | Contains .bas files |
| `.gbkb` | Knowledge Base | Document collections |
| `.gbot` | Bot Config | Contains config.csv |
| `.gbtheme` | Theme Package | CSS themes |
| `.gbai` | Bot Package | Root container |

## Debugging Support

There is no interactive debugger. A BASIC script runs to completion inside a
run, and the outcome is visible in the run record: the state, the tool-call
count, the last tool name and the error text. `GET /api/vibe/run/:run_id`
returns all of them.

For a script that fails while it is being written, the durable output is the
compile-time transform dump written next to the runtime dumps, plus the error
position in the run record. The vocabulary of a failure is therefore the script
line, not a stack of debugger frames.

## Best Practices

Effective IDE configuration significantly improves development productivity. Enable format on save to keep code consistently formatted across your project. Configure linting to catch errors early in the development cycle. Set up keyboard shortcuts for common tasks like deployment and script execution to speed up your workflow. Create and use snippets to reduce repetitive typing when writing common patterns. Finally, keep your extensions updated to benefit from the latest features and bug fixes.

## Troubleshooting

When a file does not appear in the Editor, check that the bot's Drive bucket is
reachable and that `git status` reports the file as tracked or untracked; the
tree route lists untracked files as well.

If syntax highlighting is missing in an external editor, ensure file extensions
are properly associated with the BASIC language mode and restart the editor
after changing the association.

When commands are not working, verify your server connection settings are correct, check API credentials if authentication is required, and review the editor console for error messages that might indicate the cause.