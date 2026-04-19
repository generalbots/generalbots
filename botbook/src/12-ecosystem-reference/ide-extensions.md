# IDEs

General Bots supports development with any text editor or IDE. Choose the one that works best for your workflow.

## Zed Editor (Best for Rust Development)

Zed is a high-performance, collaborative code editor that excels at Rust development and is recommended for working with General Bots core. The editor provides native Rust support with excellent syntax highlighting, delivers fast performance with minimal resource usage, includes built-in collaboration features, and offers a modern, clean interface.

### Installation

```bash
# Install Zed
curl https://zed.dev/install.sh | sh
```

## Other Popular IDEs

You can use any IDE or text editor you prefer. Visual Studio Code offers an extensive extension marketplace, good BASIC syntax highlighting with custom extensions, an integrated terminal for running General Bots, and Git integration. IntelliJ IDEA and RustRover provide excellent Rust support, powerful refactoring tools, and database tools for PostgreSQL integration. Neovim appeals to developers who prefer a lightweight, fast, highly customizable, terminal-based workflow. Sublime Text is known for being fast and responsive, with multiple cursors, powerful search capabilities, and customizable syntax highlighting.

## BASIC Script Support

For editing `.bas` files (General Bots dialog scripts), you can configure your editor with custom key bindings and project settings.

#### Key Bindings Configuration

```json
{
  "bindings": {
    "cmd-shift-b": "botserver:run-script",
    "cmd-shift-d": "botserver:deploy-bot",
    "cmd-shift-l": "botserver:view-logs"
  }
}
```

#### Project Settings

Create `.zed/settings.json` in your bot project:

```json
{
  "file_types": {
    "BASIC": ["*.bas", "*.gbdialog"],
    "Config": ["*.csv", "*.gbot"]
  },
  "format_on_save": true,
  "tab_size": 2
}
```

## Vim/Neovim Plugin

### Installation

The Vim plugin can be installed using vim-plug by adding the following to your configuration:

```vim
" ~/.vimrc or ~/.config/nvim/init.vim
Plug 'botserver/vim-botserver'
```

For Neovim users preferring lazy.nvim, use this Lua configuration:

```lua
-- ~/.config/nvim/lua/plugins/botserver.lua
return {
  'botserver/nvim-botserver',
  config = function()
    require('botserver').setup({
      server_url = 'http://localhost:8080',
      default_bot = 'edu'
    })
  end
}
```

### Features

The plugin includes syntax files for BASIC highlighting:

```vim
" ~/.vim/syntax/basic.vim
syn keyword basicKeyword TALK HEAR SET GET LLM
syn keyword basicConditional IF THEN ELSE END
syn keyword basicRepeat FOR EACH NEXT
syn match basicComment "^REM.*$"
syn match basicComment "'.*$"
```

The plugin provides several commands for interacting with botserver. Use `:BotDeploy` to deploy the current bot, `:BotRun` to run the current script, `:BotLogs` to view server logs, and `:BotConnect` to connect to the server.

## Emacs Mode

### Installation

Add the botserver mode to your Emacs configuration:

```elisp
;; ~/.emacs.d/init.el
(add-to-list 'load-path "~/.emacs.d/botserver-mode")
(require 'botserver-mode)
(add-to-list 'auto-mode-alist '("\\.bas\\'" . botserver-mode))
```

### Features

The major mode definition provides BASIC script editing support:

```elisp
(define-derived-mode botserver-mode prog-mode "botserver"
  "Major mode for editing botserver BASIC scripts."
  (setq-local comment-start "REM ")
  (setq-local comment-end "")
  (setq-local indent-line-function 'botserver-indent-line))
```

The mode includes convenient key bindings: `C-c C-c` runs the current script, `C-c C-d` deploys the bot, and `C-c C-l` displays the logs.

## Sublime Text Package

### Installation

The package can be installed via Package Control by opening the command palette with `Cmd+Shift+P`, selecting "Package Control: Install Package", and searching for "botserver". For manual installation, clone the repository directly:

```bash
cd ~/Library/Application\ Support/Sublime\ Text/Packages
git clone https://github.com/botserver/sublime-botserver botserver
```

The package provides BASIC syntax highlighting, a build system for running scripts, snippets for common patterns, and project templates.

## TextMate Bundle

### Installation

Clone the bundle to your TextMate bundles directory:

```bash
cd ~/Library/Application\ Support/TextMate/Bundles
git clone https://github.com/botserver/botserver.tmbundle
```

The bundle includes a language grammar for BASIC, commands for deployment, and tab triggers for snippets.

## Language Server Protocol (LSP)

botserver includes an LSP server that works with any LSP-compatible editor. This enables a consistent development experience across different editors and platforms.

### Starting the LSP Server

```bash
botserver --lsp --stdio
```

The LSP server provides completion suggestions, hover documentation, go to definition, find references, diagnostics for error detection, and code actions for quick fixes.

### Configuration Example

For any LSP client, use this configuration:

```json
{
  "command": ["botserver", "--lsp", "--stdio"],
  "filetypes": ["basic", "bas"],
  "rootPatterns": [".gbai", "config.csv"],
  "initializationOptions": {
    "bot": "default"
  }
}
```

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

### Breakpoints

Set breakpoints in BASIC scripts by adding a comment marker:

```basic
TALK "Before breakpoint"
' BREAKPOINT
TALK "After breakpoint"
```

### Watch Variables

Monitor variable values during execution by adding watch comments:

```basic
' WATCH: user_name
' WATCH: greeting
user_name = GET "name"
greeting = "Hello " + user_name
```

### Step Execution

The debugger supports several execution control modes. Step Over executes the current line and moves to the next. Step Into enters function calls to debug their internals. Step Out exits the current function and returns to the caller. Continue resumes normal execution until the next breakpoint.

## Best Practices

Effective IDE configuration significantly improves development productivity. Enable format on save to keep code consistently formatted across your project. Configure linting to catch errors early in the development cycle. Set up keyboard shortcuts for common tasks like deployment and script execution to speed up your workflow. Create and use snippets to reduce repetitive typing when writing common patterns. Finally, keep your extensions updated to benefit from the latest features and bug fixes.

## Troubleshooting

When the LSP server fails to start, verify that the botserver binary is in your PATH, confirm the server is running on the expected port, and review the LSP logs in your editor's output panel.

If syntax highlighting is missing, ensure file extensions are properly associated with the BASIC language mode, restart your editor after installing the extension, and check that the language mode is correctly set for open files.

When commands are not working, verify your server connection settings are correct, check API credentials if authentication is required, and review the editor console for error messages that might indicate the cause.