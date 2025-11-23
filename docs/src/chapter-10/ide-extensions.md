# IDE Extensions

BotServer provides extensions and plugins for modern code editors to enhance the development experience with BASIC scripts, bot configurations, and platform integration.

## Zed Editor (Recommended)

Zed is a high-performance, collaborative code editor built for the modern developer.

### Installation

```bash
# Install Zed
curl https://zed.dev/install.sh | sh

# Install BotServer extension
zed --install-extension botserver
```

### Features

#### Syntax Highlighting
- BASIC keywords and functions
- Configuration CSV files
- Bot package structure recognition
- Theme CSS variables

#### Language Server Protocol (LSP)

Configure in `~/.config/zed/settings.json`:
```json
{
  "lsp": {
    "botserver": {
      "binary": {
        "path": "/usr/local/bin/botserver",
        "arguments": ["--lsp"]
      },
      "initialization_options": {
        "bot": "default",
        "enableDebug": true
      }
    }
  }
}
```

#### Key Bindings

Add to `~/.config/zed/keymap.json`:
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

Using vim-plug:
```vim
" ~/.vimrc or ~/.config/nvim/init.vim
Plug 'botserver/vim-botserver'
```

Using lazy.nvim:
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

#### Syntax Files
```vim
" ~/.vim/syntax/basic.vim
syn keyword basicKeyword TALK HEAR SET GET LLM
syn keyword basicConditional IF THEN ELSE END
syn keyword basicRepeat FOR EACH NEXT
syn match basicComment "^REM.*$"
syn match basicComment "'.*$"
```

#### Commands
- `:BotDeploy` - Deploy current bot
- `:BotRun` - Run current script
- `:BotLogs` - View server logs
- `:BotConnect` - Connect to server

## Emacs Mode

### Installation

```elisp
;; ~/.emacs.d/init.el
(add-to-list 'load-path "~/.emacs.d/botserver-mode")
(require 'botserver-mode)
(add-to-list 'auto-mode-alist '("\\.bas\\'" . botserver-mode))
```

### Features

#### Major Mode
```elisp
(define-derived-mode botserver-mode prog-mode "BotServer"
  "Major mode for editing BotServer BASIC scripts."
  (setq-local comment-start "REM ")
  (setq-local comment-end "")
  (setq-local indent-line-function 'botserver-indent-line))
```

#### Key Bindings
- `C-c C-c` - Run current script
- `C-c C-d` - Deploy bot
- `C-c C-l` - View logs

## Sublime Text Package

### Installation

```bash
# Via Package Control
# Cmd+Shift+P -> Package Control: Install Package -> BotServer

# Manual installation
cd ~/Library/Application\ Support/Sublime\ Text/Packages
git clone https://github.com/botserver/sublime-botserver BotServer
```

### Features

- BASIC syntax highlighting
- Build system for running scripts
- Snippets for common patterns
- Project templates

## TextMate Bundle

### Installation

```bash
cd ~/Library/Application\ Support/TextMate/Bundles
git clone https://github.com/botserver/botserver.tmbundle
```

### Features

- Language grammar for BASIC
- Commands for deployment
- Tab triggers for snippets

## Language Server Protocol (LSP)

BotServer includes an LSP server that works with any LSP-compatible editor:

### Starting the LSP Server

```bash
botserver --lsp --stdio
```

### Capabilities

- Completion
- Hover documentation
- Go to definition
- Find references
- Diagnostics
- Code actions

### Configuration Example

For any LSP client:
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

#### Tool Definition
```basic
PARAM ${name} AS ${type} LIKE "${example}" DESCRIPTION "${description}"
DESCRIPTION "${tool_description}"
${body}
```

#### Dialog Flow
```basic
TALK "${greeting}"
HEAR response
IF response = "${expected}" THEN
    ${action}
END IF
```

#### Knowledge Base Usage
```basic
USE KB "${collection}"
answer = LLM "${prompt}"
TALK answer
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

Set breakpoints in BASIC scripts:
```basic
TALK "Before breakpoint"
' BREAKPOINT
TALK "After breakpoint"
```

### Watch Variables

Monitor variable values during execution:
```basic
' WATCH: user_name
' WATCH: response
user_name = GET "name"
response = LLM "Hello " + user_name
```

### Step Execution

Control flow with debug commands:
- Step Over: Execute current line
- Step Into: Enter function calls
- Step Out: Exit current function
- Continue: Resume execution

## Best Practices

1. **Use Format on Save**: Keep code consistently formatted
2. **Enable Linting**: Catch errors early
3. **Configure Shortcuts**: Speed up common tasks
4. **Use Snippets**: Reduce repetitive typing
5. **Keep Extensions Updated**: Get latest features and fixes

## Troubleshooting

### LSP Not Starting
- Check botserver binary is in PATH
- Verify server is running on expected port
- Review LSP logs in editor

### Syntax Highlighting Missing
- Ensure file extensions are properly associated
- Restart editor after installing extension
- Check language mode is set correctly

### Commands Not Working
- Verify server connection settings
- Check API credentials if required
- Review editor console for errors