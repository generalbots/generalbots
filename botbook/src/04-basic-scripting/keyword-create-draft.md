# CREATE DRAFT Keyword

**Syntax**

```
CREATE DRAFT "to-address", "subject", "reply-text"
```

**Parameters**

- `"to-address"` – Email address of the recipient.
- `"subject"` – Subject line for the draft email.
- `"reply-text"` – Body content for the draft. If a previous email exists in the user's mailbox to the same address, its content is appended after a separator.

**Description**

`CREATE DRAFT` composes an email draft and saves it to the user's mailbox. It first checks whether a prior email has been sent to the same recipient using the `GET_LATEST_SENT_TO` helper. If such an email exists, its body (converted to HTML line breaks) is appended to the new reply text, separated by `<br><hr><br>`. The combined content is then stored as a draft via the email service configured in the application (`save_email_draft`). The keyword returns a success message or an error string.

**Example**

```basic
CREATE DRAFT "john.doe@example.com", "Project Update", "Here is the latest status..."
TALK "Draft created and saved."
```

If an earlier email to `john.doe@example.com` exists, the draft will contain the new reply followed by the previous email content, allowing the user to continue the conversation seamlessly.
