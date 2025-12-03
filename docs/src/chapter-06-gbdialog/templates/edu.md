# Education Template

The education template provides a comprehensive educational institution assistant that helps students and staff with enrollment, course management, schedules, grades, tuition information, and academic support.

## Topic: Educational Institution Assistant

This template is perfect for:
- Universities and colleges
- Online learning platforms
- Training centers
- K-12 schools
- Corporate learning management

## The Code

```basic
ADD TOOL "enrollment"
ADD TOOL "course-info"
ADD TOOL "schedule"
ADD TOOL "grades"
ADD TOOL "tuition"
ADD TOOL "support"

USE KB "edu.gbkb"

CLEAR SUGGESTIONS

ADD SUGGESTION "enroll" AS "Enroll in a course"
ADD SUGGESTION "courses" AS "View available courses"
ADD SUGGESTION "schedule" AS "My class schedule"
ADD SUGGESTION "grades" AS "Check my grades"
ADD SUGGESTION "tuition" AS "Payment information"
ADD SUGGESTION "help" AS "Academic support"

SET CONTEXT "education" AS "You are an educational institution assistant helping with enrollment, courses, schedules, grades, and academic support. Be helpful and guide students through processes clearly."

BEGIN TALK
**Education Assistant**

Welcome! I can help you with:
• Course enrollment and registration
• Available courses and programs
• Class schedules and calendars
• Grades and transcripts
• Tuition and payment info
• Academic support and advising

Select an option or ask me anything.
END TALK

BEGIN SYSTEM PROMPT
You are an AI assistant for an educational institution.

Be friendly and professional.
Provide clear, accurate assistance.
Reduce administrative workload by handling common inquiries.
Help with enrollment and registration.
Provide course information and prerequisites.
Answer admissions questions.
Guide through registration process.
Explain academic policies.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the education template works in real-world scenarios.

### Dialog 1: Course Enrollment

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🎓</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Education Assistant</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Education Assistant</div>
      <p><strong>Education Assistant</strong></p>
      <p>Welcome! I can help you with:</p>
      <p>• Course enrollment and registration<br>• Available courses and programs<br>• Class schedules and calendars<br>• Grades and transcripts</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I want to enroll in a course</p>
      <div class="wa-time">10:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Great! I'd be happy to help you enroll in a course. 📚</p>
      <p>What subject area are you interested in?</p>
      <p>• Computer Science<br>• Business Administration<br>• Languages<br>• Mathematics<br>• Sciences</p>
      <p>Or tell me the specific course name if you know it.</p>
      <div class="wa-time">10:01</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Computer Science</p>
      <div class="wa-time">10:02 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📖 <strong>Available Computer Science Courses:</strong></p>
      <p>1. CS101 - Introduction to Programming<br>   📅 Mon/Wed 9:00 AM | 3 credits | Open</p>
      <p>2. CS201 - Data Structures<br>   📅 Tue/Thu 2:00 PM | 4 credits | Open</p>
      <p>3. CS301 - Algorithms<br>   📅 Mon/Wed 1:00 PM | 4 credits | Waitlist</p>
      <p>Which course would you like to enroll in?</p>
      <div class="wa-time">10:02</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>CS101</p>
      <div class="wa-time">10:03 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Enrollment Confirmed!</strong></p>
      <p>📚 <strong>Course:</strong> CS101 - Introduction to Programming<br>📅 <strong>Schedule:</strong> Mon/Wed 9:00-10:30 AM<br>🏫 <strong>Room:</strong> Building A, Room 204<br>👨‍🏫 <strong>Instructor:</strong> Prof. Johnson</p>
      <p>You'll receive a confirmation email shortly. Your first class is next Monday!</p>
      <div class="wa-time">10:03</div>
    </div>
  </div>
</div>

### Dialog 2: Check Grades

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🎓</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Education Assistant</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What are my grades?</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 <strong>Your Current Grades - Fall 2024</strong></p>
      <p>┌─────────────────────────┐<br>│ CS101 Programming    A  │<br>│ MATH201 Calculus     B+ │<br>│ ENG102 Composition   A- │<br>│ PHYS101 Physics      B  │<br>└─────────────────────────┘</p>
      <p><strong>Term GPA:</strong> 3.45<br><strong>Cumulative GPA:</strong> 3.52</p>
      <p>Great work! You're on the Dean's List! 🌟</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
</div>

### Dialog 3: Class Schedule

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🎓</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Education Assistant</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show my class schedule</p>
      <div class="wa-time">08:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📅 <strong>Your Weekly Schedule</strong></p>
      <p><strong>Monday:</strong><br>• 9:00 AM - CS101 (Room A204)<br>• 2:00 PM - MATH201 (Room B105)</p>
      <p><strong>Tuesday:</strong><br>• 10:30 AM - ENG102 (Room C301)<br>• 2:00 PM - PHYS101 (Lab D102)</p>
      <p><strong>Wednesday:</strong><br>• 9:00 AM - CS101 (Room A204)<br>• 2:00 PM - MATH201 (Room B105)</p>
      <p><strong>Thursday:</strong><br>• 10:30 AM - ENG102 (Room C301)<br>• 2:00 PM - PHYS101 (Room B203)</p>
      <p><strong>Friday:</strong><br>• Office Hours available</p>
      <div class="wa-time">08:00</div>
    </div>
  </div>
</div>

### Dialog 4: Tuition Payment

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🎓</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Education Assistant</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>How much do I owe for tuition?</p>
      <div class="wa-time">11:20 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>💰 <strong>Tuition Summary - Fall 2024</strong></p>
      <p><strong>Charges:</strong><br>• Tuition (14 credits): $7,000.00<br>• Technology Fee: $150.00<br>• Activity Fee: $75.00<br>───────────────<br><strong>Total:</strong> $7,225.00</p>
      <p><strong>Payments/Aid:</strong><br>• Scholarship: -$3,000.00<br>• Payment (Aug 15): -$2,000.00<br>───────────────<br><strong>Balance Due:</strong> $2,225.00</p>
      <p>📅 <strong>Due Date:</strong> October 15, 2024</p>
      <p>Would you like to make a payment or set up a payment plan?</p>
      <div class="wa-time">11:20</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Set up a payment plan</p>
      <div class="wa-time">11:21 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 <strong>Payment Plan Options</strong></p>
      <p><strong>Option 1:</strong> 2 Monthly Payments<br>• $1,112.50 on Oct 15<br>• $1,112.50 on Nov 15<br>• No fee</p>
      <p><strong>Option 2:</strong> 4 Monthly Payments<br>• $556.25 per month (Oct-Jan)<br>• $25 setup fee</p>
      <p>Which option would you prefer?</p>
      <div class="wa-time">11:21</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register enrollment and academic tools |
| `USE KB` | Load educational knowledge base |
| `ADD SUGGESTION` | Create quick action buttons |
| `SET CONTEXT` | Define educational assistant behavior |
| `BEGIN TALK` | Welcome message block |
| `BEGIN SYSTEM PROMPT` | AI behavior instructions |

## Template Structure

```
edu.gbai/
├── edu.gbdialog/
│   ├── start.bas           # Main entry point
│   └── enrollment.bas      # Enrollment workflow
├── edu.gbdata/
│   └── (data tables)       # Student/course data
├── edu.gbot/
│   └── config.csv          # Bot configuration
└── edu.gbkb/
    └── academic-policies.md # Knowledge base
```

## Enrollment Tool: enrollment.bas

```basic
PARAM student_id AS STRING DESCRIPTION "Student ID number"
PARAM course_code AS STRING LIKE "CS101" DESCRIPTION "Course code to enroll in"

DESCRIPTION "Enroll a student in a course after checking prerequisites and availability"

' Verify student exists
student = FIND "students.csv", "id = '" + student_id + "'"
IF NOT student THEN
    TALK "Student ID not found. Please verify your ID."
    RETURN NULL
END IF

' Get course information
course = FIND "courses.csv", "code = '" + course_code + "'"
IF NOT course THEN
    TALK "Course " + course_code + " not found."
    RETURN NULL
END IF

' Check if already enrolled
existing = FIND "enrollments.csv", "student_id = '" + student_id + "' AND course_code = '" + course_code + "'"
IF existing THEN
    TALK "You're already enrolled in " + course_code + "."
    RETURN NULL
END IF

' Check prerequisites
IF course.prerequisite <> "" THEN
    prereq = FIND "enrollments.csv", "student_id = '" + student_id + "' AND course_code = '" + course.prerequisite + "' AND grade >= 'C'"
    IF NOT prereq THEN
        TALK "You need to complete " + course.prerequisite + " before enrolling in " + course_code + "."
        RETURN NULL
    END IF
END IF

' Check availability
enrolled_count = COUNT("enrollments.csv", "course_code = '" + course_code + "' AND term = 'Fall2024'")
IF enrolled_count >= course.capacity THEN
    TALK "This course is full. Would you like to join the waitlist?"
    HEAR waitlist_choice
    IF LOWER(waitlist_choice) = "yes" THEN
        WITH waitlist_entry
            student_id = student_id
            course_code = course_code
            timestamp = NOW()
        END WITH
        SAVE "waitlist.csv", waitlist_entry
        TALK "You've been added to the waitlist. We'll notify you if a spot opens."
    END IF
    RETURN NULL
END IF

' Create enrollment
WITH enrollment
    id = GUID()
    student_id = student_id
    course_code = course_code
    term = "Fall2024"
    enrollment_date = NOW()
    status = "enrolled"
END WITH

SAVE "enrollments.csv", enrollment

' Send confirmation email
SEND MAIL student.email, "Enrollment Confirmed: " + course_code, 
    "You have been enrolled in " + course.name + ".\n" +
    "Schedule: " + course.schedule + "\n" +
    "Room: " + course.room + "\n" +
    "Instructor: " + course.instructor

TALK "✅ You're enrolled in " + course.name + "!"
TALK "📅 Schedule: " + course.schedule
TALK "🏫 Room: " + course.room

RETURN enrollment.id
```

## Grades Tool: grades.bas

```basic
PARAM student_id AS STRING DESCRIPTION "Student ID number"
PARAM term AS STRING LIKE "Fall2024" DESCRIPTION "Academic term" OPTIONAL

DESCRIPTION "Retrieve student grades for current or specified term"

IF NOT term THEN
    term = "Fall2024"  ' Current term
END IF

' Get student info
student = FIND "students.csv", "id = '" + student_id + "'"
IF NOT student THEN
    TALK "Student not found."
    RETURN NULL
END IF

' Get enrollments with grades
enrollments = FIND "enrollments.csv", "student_id = '" + student_id + "' AND term = '" + term + "'"

IF UBOUND(enrollments) = 0 THEN
    TALK "No courses found for " + term + "."
    RETURN NULL
END IF

TALK "📊 **Grades for " + student.name + " - " + term + "**"
TALK ""

total_points = 0
total_credits = 0

FOR EACH enrollment IN enrollments
    course = FIND "courses.csv", "code = '" + enrollment.course_code + "'"
    
    grade_display = enrollment.grade
    IF grade_display = "" THEN
        grade_display = "In Progress"
    END IF
    
    TALK "• " + enrollment.course_code + " - " + course.name + ": **" + grade_display + "**"
    
    IF enrollment.grade <> "" THEN
        grade_points = GRADE_TO_POINTS(enrollment.grade)
        total_points = total_points + (grade_points * course.credits)
        total_credits = total_credits + course.credits
    END IF
NEXT

IF total_credits > 0 THEN
    gpa = total_points / total_credits
    TALK ""
    TALK "**Term GPA:** " + FORMAT(gpa, "#.00")
    
    IF gpa >= 3.5 THEN
        TALK "🌟 Dean's List!"
    END IF
END IF

RETURN enrollments
```

## Customization Ideas

### Add Course Recommendations

```basic
ADD TOOL "recommend-courses"

' Based on major and completed courses
completed = FIND "enrollments.csv", "student_id = '" + student_id + "' AND grade >= 'C'"
major = student.major

' Find next required courses
requirements = FIND "degree_requirements.csv", "major = '" + major + "'"

recommended = []
FOR EACH req IN requirements
    already_done = FILTER(completed, "course_code = '" + req.course_code + "'")
    IF UBOUND(already_done) = 0 THEN
        ' Check if prerequisites met
        IF req.prerequisite = "" OR HAS_COMPLETED(student_id, req.prerequisite) THEN
            PUSH recommended, req
        END IF
    END IF
NEXT

TALK "Based on your progress, I recommend these courses for next term:"
FOR EACH course IN FIRST(recommended, 5)
    TALK "• " + course.course_code + " - " + course.name
NEXT
```

### Add Academic Calendar Integration

```basic
ADD TOOL "important-dates"

dates = FIND "academic_calendar.csv", "date >= '" + NOW() + "' AND date <= '" + DATEADD(NOW(), 30, 'days') + "'"

TALK "📅 **Upcoming Important Dates:**"
FOR EACH date IN dates
    TALK "• " + FORMAT(date.date, "MMM DD") + ": " + date.event
NEXT
```

### Add Advisor Scheduling

```basic
ADD TOOL "book-advisor"

PARAM preferred_date AS DATE DESCRIPTION "Preferred date for appointment"

advisor = FIND "advisors.csv", "department = '" + student.major + "'"
available = FIND "advisor_slots.csv", "advisor_id = '" + advisor.id + "' AND date = '" + preferred_date + "' AND booked = false"

IF UBOUND(available) > 0 THEN
    TALK "Available times on " + FORMAT(preferred_date, "MMM DD") + ":"
    FOR EACH slot IN available
        ADD SUGGESTION slot.time AS slot.time
    NEXT
    HEAR selected_time
    
    ' Book the appointment
    UPDATE "advisor_slots" SET booked = true WHERE id = slot.id
    
    TALK "✅ Appointment booked with " + advisor.name + " on " + FORMAT(preferred_date, "MMM DD") + " at " + selected_time
    SEND MAIL student.email, "Advisor Appointment Confirmed", "Your meeting with " + advisor.name + " is scheduled."
END IF
```

### Add Document Requests

```basic
ADD TOOL "request-transcript"

PARAM delivery_method AS STRING LIKE "email" DESCRIPTION "Delivery: email, mail, or pickup"

' Check for holds
holds = FIND "student_holds.csv", "student_id = '" + student_id + "' AND resolved = false"
IF UBOUND(holds) > 0 THEN
    TALK "⚠️ There's a hold on your account. Please resolve it before requesting transcripts."
    TALK "Hold reason: " + holds[1].reason
    RETURN NULL
END IF

' Create transcript request
WITH request
    id = GUID()
    student_id = student_id
    type = "official_transcript"
    delivery = delivery_method
    status = "processing"
    request_date = NOW()
    fee = 10.00
END WITH

SAVE "document_requests.csv", request

TALK "✅ Transcript request submitted!"
TALK "📋 Request #: " + request.id
TALK "💰 Fee: $10.00 (added to your account)"
TALK "📬 Delivery: " + delivery_method
TALK "⏱️ Processing time: 3-5 business days"
```

## Related Templates

- [start.bas](./start.md) - Basic greeting patterns
- [enrollment.bas](./enrollment.md) - Detailed enrollment workflow
- [auth.bas](./auth.md) - Student authentication

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