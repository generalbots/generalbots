# HUMAN APPROVAL Keywords

Pause bot execution until a human reviews and approves, rejects, or modifies a pending action.

## Keywords

| Keyword | Purpose |
|---------|---------|
| `REQUEST APPROVAL` | Submit action for human review |
| `WAIT FOR APPROVAL` | Pause until approval received |
| `CHECK APPROVAL` | Check approval status without blocking |

## REQUEST APPROVAL

```basic
approval_id = REQUEST APPROVAL "Transfer $5000 to vendor account"
```

With metadata:

```basic
approval_id = REQUEST APPROVAL "Delete customer records", "compliance-team", "high"
```

## WAIT FOR APPROVAL

```basic
approval_id = REQUEST APPROVAL "Publish marketing campaign"
result = WAIT FOR APPROVAL approval_id, 3600  ' Wait up to 1 hour

IF result.status = "approved" THEN
    TALK "Campaign published!"
ELSE
    TALK "Campaign rejected: " + result.reason
END IF
```

## CHECK APPROVAL

Non-blocking status check:

```basic
status = CHECK APPROVAL approval_id

TALK "Current status: " + status.state
```

## Approval States

| State | Description |
|-------|-------------|
| `pending` | Awaiting human review |
| `approved` | Action approved |
| `rejected` | Action denied |
| `modified` | Approved with changes |
| `expired` | Timeout reached |

## Example: Financial Approval Workflow

```basic
' Large transaction approval
amount = 10000
approval_id = REQUEST APPROVAL "Wire transfer: $" + amount, "finance-team", "critical"

TALK "Your transfer request has been submitted for approval."
TALK "You'll be notified when reviewed."

result = WAIT FOR APPROVAL approval_id, 86400  ' 24 hour timeout

SWITCH result.status
    CASE "approved"
        TALK "Transfer approved by " + result.approver
    CASE "rejected"
        TALK "Transfer denied: " + result.reason
    CASE "expired"
        TALK "Request expired. Please resubmit."
END SWITCH
```

## Configuration

Add to `config.csv`:

```csv
approval-timeout-default,3600
approval-notify-channel,slack
approval-escalation-hours,4
```

## See Also

- [WAIT](./keyword-wait.md)
- [SEND MAIL](./keyword-send-mail.md)