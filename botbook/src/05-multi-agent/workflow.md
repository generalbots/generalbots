# Task Workflow

From request to working application, the task workflow handles every step automatically.

![Task Workflow](../assets/05-multi-agent/task-workflow.svg)

---

## The Three Steps

| Step | What Happens |
|------|--------------|
| 1. **Describe** | You tell the bot what you want in plain language |
| 2. **Plan** | System analyzes and creates an execution plan |
| 3. **Execute** | Steps run automatically, app is delivered |

---

## Example Workflow

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create a CRM for my store with customers, products, and sales</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Planning:</strong></p>
      <p>Step 1: Create database tables</p>
      <p>Step 2: Generate customer management UI</p>
      <p>Step 3: Generate product inventory UI</p>
      <p>Step 4: Generate sales tracking UI</p>
      <p>Step 5: Create voice commands</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Done:</strong></p>
      <p>customers table created</p>
      <p>products table created</p>
      <p>sales table created</p>
      <p>App available at /apps/crm</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>

---

## Error Handling

When something goes wrong, the system asks what to do.

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Error:</strong> Could not create chart component</p>
      <p>Options: Retry, Skip, Cancel</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Skip</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Skipped chart component. Continuing with remaining steps.</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>
</div>

---

## Approval Requests

Some actions require confirmation before executing.

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Approval required:</strong></p>
      <p>This action will send 50 emails to customers.</p>
      <p>Confirm?</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Yes</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>50 emails sent successfully.</p>
      <div class="wa-time">11:01</div>
    </div>
  </div>
</div>

---

## Actions Requiring Approval

| Action | Reason |
|--------|--------|
| Bulk email sends | Prevents accidental spam |
| Data deletion | Prevents data loss |
| External API calls | Cost and security |
| Schema changes | Database integrity |

---

## Next Steps

- [Designer Guide](./designer.md) — Edit apps through conversation
- [Examples](./examples.md) — Real-world applications