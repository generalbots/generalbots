# EXIT FOR Keyword

**Syntax**

```
EXIT FOR
```

**Parameters**

_None_ – This keyword takes no arguments.

**Description**

`EXIT FOR` terminates the execution of the nearest enclosing `FOR EACH … IN … NEXT` loop prematurely. When the interpreter encounters `EXIT FOR`, it stops iterating over the collection and continues execution after the `NEXT` statement that matches the loop variable.

**Example**

```basic
FOR EACH item IN my_list
    IF item = "stop" THEN
        EXIT FOR
    ENDIF
    TALK item
NEXT item
TALK "Loop ended."
```

In this script, the loop stops as soon as `item` equals `"stop"`, and the subsequent `TALK "Loop ended."` is executed.

**Usage Notes**

- `EXIT FOR` can only be used inside a `FOR EACH … IN … NEXT` block.
- It does not accept any parameters; it simply signals an early exit.
- The keyword is case‑insensitive; `exit for` works the same way.
