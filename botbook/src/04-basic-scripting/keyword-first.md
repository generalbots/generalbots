# FIRST Keyword

**Syntax**

```
FIRST "text"
```

**Parameters**

- `"text"` – A string expression from which the first word will be extracted.

**Description**

`FIRST` returns the first whitespace‑separated token of the provided string. If the string is empty or contains only whitespace, the result is an empty string. The keyword is useful for extracting a leading command or identifier from user input.

**Example**

```basic
SET command = FIRST user_input
TALK "You entered the command: " + command
```

If `user_input` is `"search books about Rust"`, `FIRST` returns `"search"`.

**Implementation Notes**

- The keyword splits the string on any whitespace (spaces, tabs, newlines) and returns the first element.
- It does not modify the original string.
- Case‑insensitive; the returned word preserves the original casing.
