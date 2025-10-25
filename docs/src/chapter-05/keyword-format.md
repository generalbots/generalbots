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
