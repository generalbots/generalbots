# LAST Keyword

**Syntax**

```
LAST "text"
```

**Parameters**

- `"text"` – A string expression from which the last word will be extracted.

**Description**

`LAST` returns the final whitespace‑separated token of the provided string. If the string is empty or contains only whitespace, the result is an empty string. This keyword is useful for retrieving the trailing part of a user’s input or any delimited text.

**Example**

```basic
SET command = LAST user_input
TALK "You entered the last word: " + command
```

If `user_input` is `"search books about Rust"`, `LAST` returns `"Rust"`.

**Implementation Notes**

- The keyword splits the string on any whitespace (spaces, tabs, newlines) and returns the last element.
- It does not modify the original string.
- Case‑insensitive; the returned word preserves the original casing.
