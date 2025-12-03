# HR Employees Template

The HR Employees template provides a comprehensive employee management system that helps HR teams manage employee records, organizational structure, and personnel information through a conversational interface.

## Topic: Employee Management & HR Directory

This template is perfect for:
- HR departments
- People operations teams
- Employee self-service portals
- Organizational management
- Employee directory services

## The Code

```basic
ADD TOOL "add-employee"
ADD TOOL "update-employee"
ADD TOOL "search-employee"
ADD TOOL "employee-directory"
ADD TOOL "org-chart"
ADD TOOL "emergency-contacts"

USE KB "employees.gbkb"

SET CONTEXT "employee management" AS "You are an HR assistant helping manage employee information. Help with adding new employees, updating records, searching the directory, viewing org charts, and managing emergency contacts. Maintain confidentiality of employee data."

CLEAR SUGGESTIONS

ADD SUGGESTION "directory" AS "Employee directory"
ADD SUGGESTION "add" AS "Add new employee"
ADD SUGGESTION "search" AS "Search employee"
ADD SUGGESTION "org" AS "Organization chart"
ADD SUGGESTION "emergency" AS "Emergency contacts"

BEGIN TALK
**Employee Management System**

I can help you with:
• View employee directory
• Add new employees
• Search for employees
• View organization chart
• Manage emergency contacts
• Generate employee reports

Select an option or tell me what you need.
END TALK

BEGIN SYSTEM PROMPT
You are an HR assistant for the Employee Management System.

Confirm sensitive operations before executing.
Never expose salaries or personal IDs without authorization.
Use professional and helpful language.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the HR Employees template works in real-world scenarios.

### Dialog 1: Search Employee

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">👥</div>
    <div class="wa-header-info">
      <div class="wa-header-name">HR Assistant</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">HR Assistant</div>
      <p><strong>Employee Management System</strong></p>
      <p>I can help you with:</p>
      <p>• View employee directory<br>• Add new employees<br>• Search for employees<br>• View organization chart</p>
      <p>Select an option or tell me what you need.</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Find John from Engineering</p>
      <div class="wa-time">09:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔍 Found 2 employees matching "John" in Engineering:</p>
      <p></p>
      <p><strong>1. John Smith</strong></p>
      <p>📧 john.smith@company.com</p>
      <p>📞 Ext. 4521</p>
      <p>💼 Senior Software Engineer</p>
      <p>🏢 Engineering > Backend Team</p>
      <p></p>
      <p><strong>2. John Davis</strong></p>
      <p>📧 john.davis@company.com</p>
      <p>📞 Ext. 4532</p>
      <p>💼 Engineering Manager</p>
      <p>🏢 Engineering > Platform Team</p>
      <p></p>
      <p>Would you like more details on either employee?</p>
      <div class="wa-time">09:01</div>
    </div>
  </div>
</div>

### Dialog 2: Add New Employee

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">👥</div>
    <div class="wa-header-info">
      <div class="wa-header-name">HR Assistant</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Add a new employee</p>
      <div class="wa-time">10:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📝 <strong>New Employee Onboarding</strong></p>
      <p>I'll help you add a new employee to the system.</p>
      <p>What is the employee's <strong>full name</strong>?</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Maria Santos</p>
      <div class="wa-time">10:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>What is Maria's <strong>job title</strong>?</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Product Designer</p>
      <div class="wa-time">10:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Which <strong>department</strong> will Maria join?</p>
      <p>• Engineering</p>
      <p>• Product</p>
      <p>• Design</p>
      <p>• Marketing</p>
      <p>• Sales</p>
      <p>• HR</p>
      <p>• Finance</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Design</p>
      <div class="wa-time">10:32 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Who will be Maria's <strong>manager</strong>?</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Sarah Johnson</p>
      <div class="wa-time">10:32 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>What is Maria's <strong>start date</strong>?</p>
      <div class="wa-time">10:32</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>February 1, 2025</p>
      <div class="wa-time">10:33 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 <strong>Please confirm new employee details:</strong></p>
      <p></p>
      <p>👤 <strong>Name:</strong> Maria Santos</p>
      <p>💼 <strong>Title:</strong> Product Designer</p>
      <p>🏢 <strong>Department:</strong> Design</p>
      <p>👔 <strong>Manager:</strong> Sarah Johnson</p>
      <p>📅 <strong>Start Date:</strong> February 1, 2025</p>
      <p></p>
      <p>Type <strong>CONFIRM</strong> to add this employee.</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>CONFIRM</p>
      <div class="wa-time">10:34 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Employee Added Successfully!</strong></p>
      <p></p>
      <p>🆔 Employee ID: EMP-2025-0147</p>
      <p>📧 Email: maria.santos@company.com</p>
      <p>📞 Extension: 4587</p>
      <p></p>
      <p>📧 Welcome email sent to Maria</p>
      <p>📋 IT ticket created for equipment setup</p>
      <p>📅 Onboarding scheduled with HR</p>
      <div class="wa-time">10:34</div>
    </div>
  </div>
</div>

### Dialog 3: View Organization Chart

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">👥</div>
    <div class="wa-header-info">
      <div class="wa-header-name">HR Assistant</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show me the org chart for Engineering</p>
      <div class="wa-time">14:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🏢 <strong>Engineering Organization</strong></p>
      <p></p>
      <p>👔 <strong>CTO</strong> - Michael Chen</p>
      <p>├── 👤 VP Engineering - Lisa Wang</p>
      <p>│   ├── 👤 John Davis (Platform Team - 8 reports)</p>
      <p>│   ├── 👤 Anna Miller (Backend Team - 6 reports)</p>
      <p>│   └── 👤 Tom Wilson (Frontend Team - 5 reports)</p>
      <p>└── 👤 Director of DevOps - James Brown</p>
      <p>    ├── 👤 DevOps Lead - Chris Lee (4 reports)</p>
      <p>    └── 👤 SRE Lead - Pat Garcia (3 reports)</p>
      <p></p>
      <p><strong>Total:</strong> 45 employees in Engineering</p>
      <p></p>
      <p>Would you like details on a specific team?</p>
      <div class="wa-time">14:00</div>
    </div>
  </div>
</div>

### Dialog 4: Emergency Contacts

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">👥</div>
    <div class="wa-header-info">
      <div class="wa-header-name">HR Assistant</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Get emergency contact for John Smith</p>
      <div class="wa-time">16:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔐 <strong>Authorization Required</strong></p>
      <p>Emergency contact information is confidential.</p>
      <p>Please confirm this is for an emergency or authorized HR purpose.</p>
      <p>Reason for access:</p>
      <div class="wa-time">16:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>John had an accident at work and needs medical assistance</p>
      <div class="wa-time">16:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🚨 <strong>Emergency Contact for John Smith</strong></p>
      <p></p>
      <p>👤 <strong>Primary:</strong> Jane Smith (Spouse)</p>
      <p>📞 Phone: (555) 123-4567</p>
      <p></p>
      <p>👤 <strong>Secondary:</strong> Robert Smith (Father)</p>
      <p>📞 Phone: (555) 987-6543</p>
      <p></p>
      <p>⚠️ Access logged for compliance.</p>
      <p>📋 Incident report form: <a href="#">Click here</a></p>
      <div class="wa-time">16:31</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register employee management tools |
| `USE KB` | Load HR knowledge base |
| `SET CONTEXT` | Define HR assistant behavior |
| `ADD SUGGESTION` | Create quick action buttons |
| `BEGIN TALK` | Welcome message block |
| `BEGIN SYSTEM PROMPT` | Confidentiality and behavior rules |

## Template Structure

```
employees.gbai/
├── employees.gbdialog/
│   ├── start.bas              # Main entry point
│   ├── add-employee.bas       # New employee onboarding
│   ├── update-employee.bas    # Update employee records
│   ├── search-employee.bas    # Employee search
│   ├── employee-directory.bas # Full directory view
│   ├── org-chart.bas          # Organization structure
│   └── emergency-contacts.bas # Emergency contact access
├── employees.gbdata/
│   └── employees.csv          # Employee database
├── employees.gbdrive/
│   └── templates/             # Document templates
├── employees.gbkb/
│   ├── hr-policies.md         # HR policies
│   └── org-structure.md       # Organization info
└── employees.gbot/
    └── config.csv             # Bot configuration
```

## Search Employee Tool: search-employee.bas

```basic
PARAM query AS STRING LIKE "John" DESCRIPTION "Name, department, or title to search for"
PARAM department AS STRING LIKE "Engineering" DESCRIPTION "Filter by department" OPTIONAL

DESCRIPTION "Search for employees by name, department, or title"

' Build search filter
filter = "name LIKE '%" + query + "%' OR title LIKE '%" + query + "%'"

IF department THEN
    filter = "(" + filter + ") AND department = '" + department + "'"
END IF

' Execute search
results = FIND "employees.csv", filter

IF UBOUND(results) = 0 THEN
    TALK "No employees found matching '" + query + "'"
    RETURN NULL
END IF

TALK "🔍 Found " + UBOUND(results) + " employee(s):"
TALK ""

FOR EACH emp IN results
    TALK "**" + emp.name + "**"
    TALK "📧 " + emp.email
    TALK "📞 Ext. " + emp.extension
    TALK "💼 " + emp.title
    TALK "🏢 " + emp.department
    TALK ""
NEXT

RETURN results
```

## Add Employee Tool: add-employee.bas

```basic
PARAM name AS STRING LIKE "John Smith" DESCRIPTION "Employee full name"
PARAM title AS STRING LIKE "Software Engineer" DESCRIPTION "Job title"
PARAM department AS STRING LIKE "Engineering" DESCRIPTION "Department name"
PARAM manager AS STRING LIKE "Jane Doe" DESCRIPTION "Manager's name"
PARAM start_date AS DATE LIKE "2025-02-01" DESCRIPTION "Start date"

DESCRIPTION "Add a new employee to the system"

' Generate employee ID
employeeId = "EMP-" + FORMAT(NOW(), "YYYY") + "-" + FORMAT(RANDOM(1000, 9999))

' Generate email
emailName = LOWER(REPLACE(name, " ", "."))
email = emailName + "@company.com"

' Assign extension
extension = FORMAT(RANDOM(4000, 4999))

' Find manager ID
managerRecord = FIND "employees.csv", "name = '" + manager + "'"
IF NOT managerRecord THEN
    TALK "⚠️ Manager '" + manager + "' not found. Please verify the name."
    RETURN NULL
END IF

' Create employee record
WITH employee
    id = employeeId
    name = name
    email = email
    extension = extension
    title = title
    department = department
    manager_id = managerRecord.id
    manager_name = manager
    start_date = start_date
    status = "active"
    created_at = NOW()
END WITH

' Save to database
SAVE "employees.csv", employee

' Send welcome email
SEND MAIL email, "Welcome to the Company!", 
    "Dear " + name + ",\n\n" +
    "Welcome to the team! Your employee ID is " + employeeId + ".\n" +
    "Your manager is " + manager + ".\n" +
    "Start date: " + FORMAT(start_date, "MMMM DD, YYYY") + "\n\n" +
    "HR will contact you with onboarding details.\n\n" +
    "Best regards,\nHR Team"

' Create IT ticket for equipment
CREATE_TASK "New Employee Setup - " + name, 
    "Please prepare workstation for new employee:\n" +
    "Name: " + name + "\n" +
    "Department: " + department + "\n" +
    "Start Date: " + FORMAT(start_date, "MMM DD, YYYY"),
    "it@company.com"

' Notify manager
SEND MAIL managerRecord.email, "New Team Member: " + name,
    "A new team member has been added:\n\n" +
    "Name: " + name + "\n" +
    "Title: " + title + "\n" +
    "Start Date: " + FORMAT(start_date, "MMM DD, YYYY") + "\n\n" +
    "Please prepare for their onboarding."

TALK "✅ Employee **" + name + "** added successfully!"
TALK "🆔 ID: " + employeeId
TALK "📧 Email: " + email
TALK "📞 Extension: " + extension

RETURN employee
```

## Org Chart Tool: org-chart.bas

```basic
PARAM department AS STRING LIKE "Engineering" DESCRIPTION "Department to show org chart for"
PARAM manager AS STRING DESCRIPTION "Show org chart under specific manager" OPTIONAL

DESCRIPTION "Display organization chart for a department or team"

IF manager THEN
    ' Get org chart under specific manager
    managerRecord = FIND "employees.csv", "name = '" + manager + "'"
    IF NOT managerRecord THEN
        TALK "Manager not found."
        RETURN NULL
    END IF
    
    reports = FIND "employees.csv", "manager_id = '" + managerRecord.id + "'"
    
    TALK "👔 **" + manager + "** - " + managerRecord.title
    FOR EACH emp IN reports
        subReports = COUNT("employees.csv", "manager_id = '" + emp.id + "'")
        IF subReports > 0 THEN
            TALK "├── 👤 " + emp.name + " (" + emp.title + " - " + subReports + " reports)"
        ELSE
            TALK "├── 👤 " + emp.name + " (" + emp.title + ")"
        END IF
    NEXT
ELSE
    ' Get department org chart
    deptHead = FIND "employees.csv", "department = '" + department + "' AND title LIKE '%Director%' OR title LIKE '%VP%'"
    
    IF NOT deptHead THEN
        deptHead = FIND "employees.csv", "department = '" + department + "' AND title LIKE '%Manager%'"
    END IF
    
    TALK "🏢 **" + department + " Organization**"
    TALK ""
    
    FOR EACH head IN deptHead
        TALK "👔 **" + head.title + "** - " + head.name
        
        reports = FIND "employees.csv", "manager_id = '" + head.id + "'"
        FOR EACH emp IN reports
            subCount = COUNT("employees.csv", "manager_id = '" + emp.id + "'")
            IF subCount > 0 THEN
                TALK "├── 👤 " + emp.name + " (" + subCount + " reports)"
            ELSE
                TALK "├── 👤 " + emp.name
            END IF
        NEXT
        TALK ""
    NEXT
END IF

totalCount = COUNT("employees.csv", "department = '" + department + "'")
TALK "**Total:** " + totalCount + " employees in " + department

RETURN department
```

## Customization Ideas

### Add Employee Self-Service

```basic
' Allow employees to update their own info
IF user_id = employee.id THEN
    TALK "What would you like to update?"
    ADD SUGGESTION "phone" AS "Phone number"
    ADD SUGGESTION "address" AS "Address"
    ADD SUGGESTION "emergency" AS "Emergency contacts"
    ADD SUGGESTION "photo" AS "Profile photo"
    
    HEAR updateChoice
    
    ' Only allow non-sensitive updates
    IF updateChoice = "phone" THEN
        TALK "Enter your new phone number:"
        HEAR newPhone
        UPDATE "employees.csv" SET phone = newPhone WHERE id = user_id
        TALK "✅ Phone number updated!"
    END IF
END IF
```

### Add Birthday Reminders

```basic
' Scheduled job for birthday notifications
SET SCHEDULE "0 9 * * *"  ' Run daily at 9 AM

today = FORMAT(NOW(), "MM-DD")
birthdays = FIND "employees.csv", "FORMAT(birth_date, 'MM-DD') = '" + today + "'"

FOR EACH emp IN birthdays
    ' Notify their team
    manager = FIND "employees.csv", "id = '" + emp.manager_id + "'"
    SEND MAIL manager.email, "🎂 Team Birthday Today!", 
        emp.name + " has a birthday today! Don't forget to wish them well."
    
    ' Send birthday message
    SEND MAIL emp.email, "🎂 Happy Birthday!", 
        "Dear " + emp.name + ",\n\nHappy Birthday from all of us!"
NEXT
```

### Add Anniversary Tracking

```basic
' Check for work anniversaries
today = FORMAT(NOW(), "MM-DD")
anniversaries = FIND "employees.csv", "FORMAT(start_date, 'MM-DD') = '" + today + "'"

FOR EACH emp IN anniversaries
    years = YEAR(NOW()) - YEAR(emp.start_date)
    IF years > 0 THEN
        SEND MAIL emp.email, "🎉 Happy Work Anniversary!",
            "Congratulations on " + years + " years with us!"
        
        ' Milestone recognition
        IF years = 5 OR years = 10 OR years = 15 OR years = 20 THEN
            CREATE_TASK "Milestone Recognition - " + emp.name,
                emp.name + " has completed " + years + " years. Please arrange recognition.",
                "hr@company.com"
        END IF
    END IF
NEXT
```

### Add Department Reports

```basic
ADD TOOL "department-report"

PARAM department AS STRING DESCRIPTION "Department to generate report for"

DESCRIPTION "Generate a department headcount and demographics report"

employees = FIND "employees.csv", "department = '" + department + "'"

totalCount = UBOUND(employees)
managerCount = 0
avgTenure = 0

FOR EACH emp IN employees
    IF INSTR(emp.title, "Manager") > 0 OR INSTR(emp.title, "Director") > 0 THEN
        managerCount = managerCount + 1
    END IF
    avgTenure = avgTenure + DATEDIFF(NOW(), emp.start_date, "years")
NEXT

avgTenure = avgTenure / totalCount

TALK "📊 **" + department + " Department Report**"
TALK ""
TALK "👥 Total Employees: " + totalCount
TALK "👔 Managers: " + managerCount
TALK "📅 Avg. Tenure: " + FORMAT(avgTenure, "#.#") + " years"
TALK ""
TALK "**By Level:**"
' ... additional breakdown
```

## Data Security

The employee management system includes several security features:

1. **Access Control**: Sensitive data requires authorization
2. **Audit Logging**: All access to confidential info is logged
3. **Data Masking**: Personal IDs and salaries are not exposed
4. **Emergency Override**: Emergency contacts accessible with justification

## Related Templates

- [helpdesk.bas](./helpdesk.md) - IT ticket integration
- [edu.bas](./edu.md) - Training and development
- [privacy.bas](./privacy.md) - Data protection compliance

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>