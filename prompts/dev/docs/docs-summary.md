# Prompt for Generating GeneralBots mdBook Documentation

**Goal**  
Generate a complete **mdBook** that documents the GeneralBots application for a user with basic knowledge of large‑language models (LLM).

**Rules**  
- Use **only real keywords** and features that exist in the source code (no invented commands).  
- Include Rust code **exclusively** in the **gbapp** chapter.  
- Keep the tone simple, beginner‑friendly, and instructional.  
- Base all information on the actual code base:  
  - Keywords in `src/basic/keywords/`  
  - Database models in `src/shared/models.rs`  
  - Example scripts in `templates/`

**Required Markdown Structure (headings)**  

1. **Run and Talk** – How to start the server and interact using `TALK` / `HEAR`.  
2. **About Packages** – Overview of the four package types (`.gbdialog`, `.gbkb`, `.gbtheme`, `.gbot`).  
3. **gbkb Reference** – Explain `ADD_KB`, `SET_KB`, `ADD_WEBSITE`.  
4. **gbtheme Reference** – Describe UI theming via CSS/HTML.  
5. **gbdialog Reference** – List the three example scripts (`start.bas`, `auth.bas`, `update-summary.bas`) and the core keywords (`TALK`, `HEAR`, `LLM`, etc.).  
6. **gbapp Reference** – Show a minimal Rust snippet that registers a keyword (e.g., `ADD_TOOL`).  
7. **gbot Reference** – Explain the `config.csv` format and editable parameters.  
8. **Tooling** – Table of all built‑in keywords with one‑line descriptions.  
9. **Feature‑Matrix** – Table mapping features to the chapters/keywords that implement them.  
10. **Contributing** – Steps for contributors (fork, branch, tests, formatting).  
11. **Appendix I – Database Model** – Summarise the main tables from `src/shared/models.rs`.  
12. **Glossary** – Definitions of extensions and key concepts.

**Output Requirements**  

- Produce **only** the markdown content (no surrounding explanations).  
- Use fenced code blocks with appropriate language tags (`bas` for BASIC scripts, `rust` for Rust snippets).  
- Include a **Table of Contents** with markdown links to each chapter.  
- Ensure the document can be built directly with `mdbook build docs/src`.  

**Example Skeleton (to be expanded by the generator)**  

```markdown
# GeneralBots User Documentation (mdBook)

## Table of Contents
- [Run and Talk](#run-and-talk)
- [About Packages](#about-packages)
- [gbkb Reference](#gbkb-reference)
- [gbtheme Reference](#gbtheme-reference)
- [gbdialog Reference](#gbdialog-reference)
- [gbapp Reference](#gbapp-reference)
- [gbot Reference](#gbot-reference)
- [Tooling](#tooling)
- [Feature‑Matrix](#feature-matrix)
- [Contributing](#contributing)
- [Appendix I – Database Model](#appendix‑i---database-model)
- [Glossary](#glossary)

## Run and Talk
```bas
TALK "Welcome! How can I help you today?"
HEAR user_input
```
*Start the server:* `cargo run --release`

## About Packages
| Component | Extension | Role |
|-----------|-----------|------|
| Dialog scripts | `.gbdialog` | BASIC‑style conversational logic |
| Knowledge bases | `.gbkb` | Vector‑DB collections |
| UI themes | `.gbtheme` | CSS/HTML assets |
| Bot config | `.gbot` | CSV mapping to `UserSession` |

## gbkb Reference
...

## gbapp Reference
```rust
pub fn add_tool_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    // registration logic …
}
```


When you are ready, output the full markdown document that satisfies the specifications above.  
*Do not include any commentary outside the markdown itself.*

# FORMAT Keyword

The **FORMAT** keyword formats numbers, dates, and text for display. Use it when you need a quick, readable representation without writing custom code.

## Syntax
```basic
RESULT = FORMAT(VALUE, PATTERN)
```

## BASIC EXAMPLE
```basic
NUMBER = 1234.56
TEXT = "John"
DATE = "2024-03-15 14:30:00"
TALK FORMAT(NUMBER, "n")      ' 1234.56
TALK FORMAT(TEXT, "Hello @!") ' Hello John!
TALK FORMAT(DATE, "dd/MM/yyyy") ' 15/03/2024
```
- **VALUE** – any number, date string (`YYYY‑MM‑DD HH:MM:SS`), or text.
- **PATTERN** – a short format string (see tables below).

## Quick Reference

### Numeric Patterns
| Pattern | Example | Output |
|---------|---------|--------|
| `n` | `FORMAT(1234.5, "n")` | `1234.50` |
| `F` | `FORMAT(1234.5, "F")` | `1234.50` |
| `f` | `FORMAT(1234.5, "f")` | `1234` |
| `0%` | `FORMAT(0.85, "0%")` | `85%` |
| `C2[en]` | `FORMAT(1234.5, "C2[en]")` | `$1,234.50` |
| `C2[pt]` | `FORMAT(1234.5, "C2[pt]")` | `R$ 1.234,50` |

### Date Patterns
| Code | Meaning | Example |
|------|---------|---------|
| `yyyy` | 4‑digit year | `2024` |
| `yy`   | 2‑digit year | `24` |
| `MM`   | month (01‑12) | `03` |
| `M`    | month (1‑12) | `3` |
| `dd`   | day (01‑31) | `05` |
| `d`    | day (1‑31) | `5` |
| `HH`   | 24‑hour (00‑23) | `14` |
| `hh`   | 12‑hour (01‑12) | `02` |
| `mm`   | minutes (00‑59) | `05` |
| `ss`   | seconds (00‑59) | `09` |
| `tt`   | AM/PM | `PM` |

**Example**
```basic
DATE = "2024-03-15 14:30:25"
TALK FORMAT(DATE, "dd/MM/yyyy HH:mm")   ' 15/03/2024 14:30
```

### Text Patterns
| Placeholder | Effect |
|-------------|--------|
| `@` | Insert original text |
| `!` | Upper‑case |
| `&` | Lower‑case |

**Example**
```basic
NAME = "Maria"
TALK FORMAT(NAME, "Hello, !")   ' Hello, MARIA
```

## Practical Tips
- **Test each pattern** in isolation before combining.
- **Locale codes** (`en`, `pt`, `fr`, …) go inside `C2[…]` for currency.
- **Dates must follow** `YYYY‑MM‑DD HH:MM:SS`; otherwise formatting fails.
- **Combine patterns** by nesting calls:
  ```basic
  TALK FORMAT(FORMAT(VALUE, "C2[en]"), "!")   ' $1,234.50 (uppercase not needed here)
  ```

## Common Pitfalls
- Using a date pattern on a non‑date string → returns the original string.
- Forgetting locale brackets (`C2[en]`) → defaults to system locale.
- Mixing placeholders (`@`, `!`, `&`) in the same pattern – only the last one applies.

Use **FORMAT** whenever you need a clean, user‑friendly output without extra code. It keeps scripts short and readable.
