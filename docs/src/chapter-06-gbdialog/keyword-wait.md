# WAIT Keyword

The **WAIT** keyword pauses script execution for a specified duration.  
It is used to introduce delays between actions, synchronize processes, or control timing in automation flows.

---

## Syntax

```basic
WAIT seconds
```

---

## Parameters

- `seconds` — The number of seconds to pause execution.  
  Can be an integer or floating-point value.  
  The maximum allowed duration is 300 seconds (5 minutes).

---

## Description

`WAIT` suspends the script for the specified duration.  
During this time, the bot does not process other commands or messages.  
This keyword is useful for pacing interactions, waiting for external events, or throttling API calls.

If the provided value is invalid (negative or non-numeric), the command raises a runtime error.  
The system automatically caps the wait time to prevent excessively long pauses.

---

## Example

```basic
' Wait for 2 seconds before continuing
TALK "Processing your request..."
WAIT 2
TALK "Done!"
```

---

## Implementation Notes

- Implemented in Rust under `src/basic/mod.rs` and `src/shared/utils.rs`.  
- Uses `std::thread::sleep` with a `Duration` derived from the provided seconds.  
- The engine ensures that the wait does not exceed the configured timeout limit.  
- During the wait, no other BASIC commands are executed.

---

## Related Keywords

- [`SET SCHEDULE`](keyword-set-schedule.md) — Defines scheduled tasks for automation.  
- [`PRINT`](keyword-print.md) — Outputs messages or debugging information.  
- [`TALK`](keyword-talk.md) — Sends messages to the user.  
- [`HEAR`](keyword-hear.md) — Receives user input after a delay.

---

## Summary

`WAIT` is a simple but essential keyword for controlling timing in BASIC scripts.  
It allows developers to create natural pauses, synchronize workflows, and manage execution pacing effectively.
