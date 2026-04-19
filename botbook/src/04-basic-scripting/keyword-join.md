# JOIN

Combines elements of an array into a single string with a specified separator.

## Syntax

```basic
result = JOIN(array, separator)
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `array` | Array | The array of elements to join |
| `separator` | String | The delimiter to place between elements |

## Description

`JOIN` concatenates all elements of an array into a single string, inserting the specified separator between each element. This is commonly used for creating comma-separated lists, building display strings, or formatting data for output.

## Examples

### Basic Usage

```basic
names = ["Alice", "Bob", "Charlie"]
result = JOIN(names, ", ")
TALK result
' Output: "Alice, Bob, Charlie"
```

### Creating Hashtag Lists

```basic
tags = ["GeneralBots", "AI", "Automation", "NoCode"]
hashtags = JOIN(tags, " #")
hashtags = "#" + hashtags
TALK hashtags
' Output: "#GeneralBots #AI #Automation #NoCode"
```

### Building File Paths

```basic
parts = ["documents", "reports", "2025", "sales.pdf"]
path = JOIN(parts, "/")
TALK "File: " + path
' Output: "File: documents/reports/2025/sales.pdf"
```

### Email Recipients

```basic
recipients = ["john@example.com", "jane@example.com", "bob@example.com"]
to_list = JOIN(recipients, "; ")
SEND MAIL to_list, "Team Update", "Please review the attached report."
```

### Display Lists

```basic
items = FIND "products", "category=electronics"
product_names = []

FOR EACH item IN items
    product_names = APPEND(product_names, item.name)
NEXT

TALK "Available products: " + JOIN(product_names, ", ")
```

## Return Value

Returns a string containing all array elements concatenated with the separator.

- If the array is empty, returns an empty string
- If the array has one element, returns that element as a string
- Null values in the array are converted to empty strings

## Sample Conversation

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What hashtags should I use for my post about AI automation?</p>
      <div class="wa-time">15:20</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Here are some great hashtags for your AI automation post:</p>
      <p></p>
      <p>#AI #Automation #MachineLearning #NoCode #Productivity #TechInnovation #DigitalTransformation</p>
      <p></p>
      <p>Copy and paste them directly into your post!</p>
      <div class="wa-time">15:20</div>
    </div>
  </div>
</div>

## Common Separators

| Separator | Use Case |
|-----------|----------|
| `", "` | Readable comma-separated lists |
| `","` | CSV data |
| `"\n"` | Multi-line output |
| `" "` | Space-separated words |
| `" \| "` | Table columns |
| `"/"` | File paths |
| `"; "` | Email recipients |

## See Also

- [SPLIT](./keyword-split.md) - Split a string into an array (opposite of JOIN)
- [FOR EACH](./keyword-for-each.md) - Iterate over arrays
- [FILTER](./keyword-filter.md) - Filter arrays before joining

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:500px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-message{margin-bottom:10px}
.wa-message.user{text-align:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;display:inline-block;text-align:left}
.wa-message.bot .wa-bubble{background-color:#fff;display:inline-block}
.wa-bubble{padding:8px 12px;border-radius:8px;box-shadow:0 1px .5px rgba(0,0,0,.13);max-width:85%}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
</style>