# WAIT Keyword

**Syntax**

```
WAIT seconds
```

**Parameters**

- `seconds` – Number of seconds to pause execution. Can be an integer or floating‑point value.

**Description**

`WAIT` suspends the script for the specified duration. The keyword validates that the argument is a non‑negative number, caps the wait time at 300 seconds (5 minutes) to prevent excessively long pauses, and then sleeps the current thread for the requested period.

During the wait, the engine does not process other commands; the dialog is effectively paused.

**Example**

```basic
TALK "Processing your request..."
WAIT 2
TALK "Done."
```

The script will wait two seconds between the two `TALK` statements.

**Implementation Notes**

- The keyword uses `std::thread::sleep` with a `Duration` derived from the provided seconds.
- Negative values result in a runtime error.
- The maximum allowed wait time is 300 seconds; values above this are truncated to the limit.
- The keyword returns a string indicating the actual wait time (e.g., `"Waited 2 seconds"`).
