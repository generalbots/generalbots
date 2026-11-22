# Designer 🟡 BETA

Designer is an AI assistant that appears alongside your application. Use natural conversation to modify any aspect of your app without writing code.

![Designer Workflow](../assets/05-multi-agent/designer-workflow.svg)

---

## How It Works

Designer understands the context of your application. When you make a request, it knows which app you are viewing, which page is active, and what tables exist.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Change the schedule button to blue</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Updating schedule.html:</p>
      <p>.btn-schedule { background-color: #3498db; }</p>
      <p>Button color changed to blue.</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
</div>

---

## What Designer Can Modify

| Category | Examples |
|----------|----------|
| **Styles** | Colors, layout, fonts, spacing |
| **HTML Pages** | Forms, lists, buttons, elements |
| **Database** | Add fields, create tables |
| **Tools** | Voice commands, chat triggers |
| **Schedulers** | Automated tasks, reports |

---

## Editing Styles

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Make the colors more modern, blue tones</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Updating styles.css:</p>
      <p>Primary color: #2563eb</p>
      <p>Secondary color: #3b82f6</p>
      <p>Background: #f8fafc</p>
      <p>Visual style updated.</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
</div>

---

## Adding Form Fields

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Add an insurance field to the patient form</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Making two changes:</p>
      <p><strong>1. tables.bas</strong> - new field:</p>
      <p>insurance AS STRING</p>
      <p><strong>2. patients.html</strong> - new input added</p>
      <p>Insurance field added to form.</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
</div>

---

## Creating Voice Commands

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create a command to see today's appointments</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Creating .gbdialog/tools/todays-appointments.bas</p>
      <p>Triggers: "today's appointments", "daily schedule"</p>
      <p>Command created. Say "today's appointments" to use it.</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
</div>

---

## Creating Schedulers

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Every day at 9am send reminders to today's patients</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Creating .gbdialog/schedulers/daily-reminder.bas</p>
      <p>Schedule: Every day at 9:00 AM</p>
      <p>Action: Send reminder to patients with appointments today</p>
      <p>Scheduler configured.</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>
</div>

---

## Adding Database Fields

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Add SSN field to the patients table</p>
      <div class="wa-time">10:45</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Updating tables.bas:</p>
      <p>Adding: ssn AS STRING UNIQUE INDEX</p>
      <p>Database synchronized.</p>
      <p>Field available in all apps.</p>
      <div class="wa-time">10:45</div>
    </div>
  </div>
</div>

---

## Context Awareness

Designer tracks your current location and correlates follow-up requests.

| Context | How It Is Used |
|---------|----------------|
| **Current app** | Edits apply to that app's files |
| **Current page** | Defaults edits to that file |
| **Recent changes** | Correlates follow-up requests |
| **Available tables** | Knows your data structure |
| **Existing tools** | Knows voice commands already created |

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Change the button to green</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Changing button in schedule.html to green.</p>
      <p>Button updated.</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>And add a notes field</p>
      <div class="wa-time">14:31</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Adding notes field to the same form.</p>
      <p>Field added.</p>
      <div class="wa-time">14:31</div>
    </div>
  </div>
</div>

---

## Handling Sensitive Information

When Designer needs API keys or credentials:

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Integrate with Stripe for payments</p>
      <div class="wa-time">15:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>To integrate with Stripe, I need the API Key.</p>
      <p>Do you want to provide it now or later?</p>
      <div class="wa-time">15:00</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Later</p>
      <div class="wa-time">15:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Saved as pending item.</p>
      <p>Pending: Stripe API Key</p>
      <p>You can provide it in Settings, Pending Items.</p>
      <p>Payment structure created. When you provide the key, it will work automatically.</p>
      <div class="wa-time">15:01</div>
    </div>
  </div>
</div>

---

## Chat Reachability and Current Parity

Designer-style editing is not a separate chat protocol. Editor operations are
reached through the same declarative command surface as every other suite app:
the model emits a `{"__api_call__": {...}}` block, discovers a command with
`api.find`, and executes it with `api.exec` or a curated verb name
(`botserver/src/core/bot/api_catalog/commands_list/mod.rs:53-62`). There is no
dedicated BASIC keyword for undo, redo, confirm or track-changes: editor verbs
live in the API command catalog, not in `botbasic_*`.

The curated catalog is a set of `AppCommand` entries
(`botserver/src/apps/commands/app_command.rs:3-24`). Each entry carries an app
id, name, label, summary, **named parameters with free-text descriptions**, a
deep link and a single `admin_only` boolean. Verb-to-route bindings live in
`botserver/src/apps/commands/verb_endpoints/table/office_endpoints.rs`;
composite Docs and Sheet dispatchers live in
`botserver/src/core/bot/api_catalog/docs_verbs.rs` and
`botserver/src/core/bot/api_catalog/sheet_verbs.rs`.

Chat-reachable editor verb families today, taken from those bindings and
catalog entries:

| App | Representative verbs |
|-----|----------------------|
| Docs | `docs.list`, `docs.create`, `docs.update`, `docs.get`, `docs.save`, `docs.delete`, `docs.search`, `docs.outline`, `docs.collaborators`, `docs.ai`, `docs.footnote.add`, `docs.comment.add`, `docs.track_changes.enable`, `docs.compare`, `docs.toc.generate` |
| Sheet | `sheet.create`, `sheet.cells.set`, `sheet.get`, `sheet.import`, `sheet.export`, `sheet.chart.create`, `sheet.conditional_format.apply`, `sheet.pivot.create`, `sheet.delete` |
| Slides | `slides.create`, `slides.get`, `slides.slide.add`, `slides.slide.delete`, `slides.slide.duplicate`, `slides.slide.notes`, `slides.element.add`, `slides.media.add`, `slides.element.delete`, `slides.theme.set`, `slides.export` |
| Canvas | `canvas.list`, `canvas.get`, `canvas.create` |

`sheet.cells.set` additionally accepts `styles`, `formats`, `conditional` and
`chart`/`charts` and applies them to the same sheet in one call
(`sheet_verbs.rs:103-115` and `:203-246`).

Full parity with typed operations, scopes, confirmation and undo is **not**
claimed. Issue #2448 tracks the gap and its acceptance criteria were still
unverified at the last triage. Concretely, the following are **not**
implemented in the catalog today:

| Requirement (#2448) | Current state in source |
|---------------------|-------------------------|
| Typed request/response JSON schema per verb | `AppCommand.params` is `&[(&str, &str)]` — names and free-text descriptions only (`app_command.rs:14-15`); there is no schema field. |
| Scopes read/write/share/admin, resolved from role and enforced per verb | Only `admin_only: bool` (`app_command.rs:22-23`). The consent classifier derives read/delete/update from the verb suffix, not a per-verb scope (`consent_class.rs:6-13`). |
| Preview-then-confirm with a diff summary for destructive verbs | The consent gate emits a permission card carrying `{command, params}` detail (`consent_gate.rs:134-154`); no diff summary is generated. |
| One command undoes the last agent batch | `/api/editor/undo` exists (`endpoint_inventory_editor_email.rs:10`, `crates/boteditor/src/routes.rs:15`) but its stack is recreated on every request (`editor_registry()` in `crates/boteditor/src/handlers/mod.rs:70-72`), so it is not a persistent cross-request or agent-batch undo. |
| Idempotency keys for batch plans | No idempotency field exists in the command catalog. |
| Parity CI test (toolbar action mapped to a catalog verb) | No `data-verb` mapping test was found in the source tree. |

A mutating command can still prompt for permission: the `__api_call__` handler
runs a consent gate that classifies the action by verb suffix and returns a
pending request or a denial (`api_call/handle.rs:130-184`). This is a permission
prompt, not the preview-with-diff flow the parity epic requires.

---

## Limitations

| Limitation | Reason |
|------------|--------|
| Cannot delete entire app | Safety protection |
| Cannot access external systems without credentials | Security |
| Cannot modify system files | Isolation |
| Cannot execute arbitrary code | Security |

---

## Available Keywords

Designer uses GB BASIC keywords to implement changes:

| Category | Keywords |
|----------|----------|
| **Communication** | TALK, HEAR, SEND MAIL, SEND TEMPLATE |
| **Data** | FIND, SAVE, UPDATE, DELETE |
| **Automation** | SET SCHEDULE, ON CHANGE, ON EMAIL |
| **AI** | LLM, USE KB |
| **Web** | SCRAPE, GET, POST, RSS |
| **Memory** | SET BOT MEMORY, GET BOT MEMORY |
| **Control** | IF/THEN/ELSE, FOR EACH/NEXT, WHILE/WEND |

---

## Tips for Better Results

### Be Specific

| Less Effective | More Effective |
|----------------|----------------|
| "improve the form" | "add email validation to the form" |
| "change the colors" | "change the main button to green" |
| "fix this" | "increase the font size in the table" |

### Combine Requests

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Add SSN field to patient form with mask and validation, and show it in the list too</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Making 3 changes:</p>
      <p>1. tables.bas - new SSN field</p>
      <p>2. patients.html - input with mask XXX-XX-XXXX</p>
      <p>3. patients.html - SSN column in the table</p>
      <p>All changes applied.</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>
</div>

---

## Next Steps

- [App Generation](./app-generation.md) — How apps are created
- [Task Workflow](./workflow.md) — How tasks are processed
- [Data Model](./data-model.md) — TABLE keyword reference