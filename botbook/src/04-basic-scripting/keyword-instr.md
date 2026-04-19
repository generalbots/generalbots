# INSTR

The `INSTR` keyword returns the position of a substring within a string, following classic BASIC semantics.

## Syntax

```basic
position = INSTR(string, substring)
position = INSTR(start, string, substring)
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `start` | number | Optional. Starting position for the search (1-based) |
| `string` | string | The string to search in |
| `substring` | string | The substring to find |

## Return Value

- Returns the 1-based position of the first occurrence of `substring` in `string`
- Returns `0` if the substring is not found
- Returns `0` if either string is empty

## Description

`INSTR` searches for the first occurrence of a substring within another string. Unlike zero-based indexing in many modern languages, INSTR uses 1-based positioning consistent with traditional BASIC.

When the optional `start` parameter is provided, the search begins at that position rather than at the beginning of the string.

## Examples

### Basic Usage

```basic
text = "Hello, General Bots!"
pos = INSTR(text, "General")
TALK "Found 'General' at position: " + pos
' Output: Found 'General' at position: 8
```

### Checking if Substring Exists

```basic
email = HEAR "Enter your email:"
IF INSTR(email, "@") > 0 THEN
    TALK "Valid email format"
ELSE
    TALK "Email must contain @"
END IF
```

### Starting Search at Position

```basic
text = "one two one three one"
first = INSTR(text, "one")      ' Returns 1
second = INSTR(5, text, "one")  ' Returns 9 (starts after first "one")
third = INSTR(10, text, "one")  ' Returns 19
```

### Extracting Data

```basic
data = "Name: John Smith"
colon_pos = INSTR(data, ":")
IF colon_pos > 0 THEN
    ' Get everything after ": "
    name = MID(data, colon_pos + 2)
    TALK "Extracted name: " + name
END IF
```

### Case-Sensitive Search

```basic
text = "General Bots"
pos1 = INSTR(text, "bots")    ' Returns 0 (not found - case matters)
pos2 = INSTR(text, "Bots")    ' Returns 9 (found)
```

### Finding Multiple Occurrences

```basic
text = "apple,banana,cherry,apple"
search = "apple"
count = 0
pos = 1

DO WHILE pos > 0
    pos = INSTR(pos, text, search)
    IF pos > 0 THEN
        count = count + 1
        pos = pos + 1  ' Move past current match
    END IF
LOOP

TALK "Found '" + search + "' " + count + " times"
```

### Validating Input Format

```basic
phone = HEAR "Enter phone number (XXX-XXX-XXXX):"
dash1 = INSTR(phone, "-")
dash2 = INSTR(dash1 + 1, phone, "-")

IF dash1 = 4 AND dash2 = 8 THEN
    TALK "Phone format is correct"
ELSE
    TALK "Invalid format. Use XXX-XXX-XXXX"
END IF
```

## Comparison with Other Keywords

| Keyword | Purpose |
|---------|---------|
| `INSTR` | Find position of substring |
| `FORMAT` | Format strings with patterns |
| `FIRST` | Get first element of array |
| `LAST` | Get last element of array |

## Notes

- **1-based indexing**: Position 1 is the first character, not 0
- **Case-sensitive**: "ABC" and "abc" are different
- **Empty strings**: Returns 0 if either string is empty
- **Not found**: Returns 0 when substring doesn't exist

## Error Handling

```basic
text = HEAR "Enter text to search:"
search = HEAR "Enter search term:"

pos = INSTR(text, search)
IF pos = 0 THEN
    TALK "'" + search + "' was not found in your text"
ELSE
    TALK "Found at position " + pos
END IF
```

## See Also

- [FORMAT](./keyword-format.md) - String formatting
- [SET](./keyword-set.md) - Variable assignment
- [IS NUMERIC](./keyword-is-numeric.md) - Check if string is numeric