# FOR EACH Keyword

**Syntax**

```
FOR EACH $var IN $collection
    // block of statements
NEXT $var
```

**Parameters**

- `$var` – Identifier that will hold each element of the collection during iteration.
- `$collection` – An array or iterable expression whose items will be traversed.

**Description**

`FOR EACH` iterates over every element of the supplied collection, assigning the current element to the loop variable `$var` for the duration of the block. The block is executed once per element. After the loop finishes, execution continues after the matching `NEXT $var` statement.

If the collection is not an array, the keyword raises a runtime error indicating the expected type.

**Example**

```basic
SET numbers = [1, 2, 3, 4, 5]
FOR EACH n IN numbers
    TALK "Number: " + n
NEXT n
TALK "All numbers processed."
```

The script outputs each number in the list sequentially and then prints a final message.

**Control Flow**

- `EXIT FOR` can be used inside the block to break out of the loop early.
- Nested `FOR EACH` loops are supported; each must have a distinct loop variable.

**Implementation Notes**

- The keyword evaluates the collection expression once before entering the loop.
- The loop variable is scoped to the block; it does not affect variables outside the loop.
