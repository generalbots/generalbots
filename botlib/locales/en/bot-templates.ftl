bot-greeting-default = Hello! How can I help you today?
bot-greeting-named = Hello, { $name }! How can I help you today?
bot-goodbye = Goodbye! Have a great day!
bot-help-prompt = I can help you with: { $topics }. What would you like to know?
bot-thank-you = Thank you for your message. How can I assist you today?
bot-echo-intro = Echo Bot: I will repeat everything you say. Type 'quit' to exit.
bot-you-said = You said: { $message }
bot-thinking = Let me think about that...
bot-processing = Processing your request...
bot-error-occurred = I'm sorry, something went wrong. Please try again.
bot-not-understood = I didn't understand that. Could you please rephrase?
bot-confirm-action = Are you sure you want to proceed?
bot-action-cancelled = Action cancelled.
bot-action-completed = Done!

bot-lead-welcome = Welcome! Let me help you get started.
bot-lead-ask-name = What's your name?
bot-lead-ask-email = And your email?
bot-lead-ask-company = What company are you from?
bot-lead-ask-phone = What's your phone number?
bot-lead-hot = Great! Our sales team will reach out shortly.
bot-lead-nurture = Thanks for your interest! We'll send you some resources.
bot-lead-score = Your lead score is { $score } out of 100.
bot-lead-saved = Your information has been saved successfully.

bot-schedule-created = Running scheduled task: { $name }
bot-schedule-next = Next run scheduled for { $datetime }
bot-schedule-cancelled = Schedule cancelled.
bot-schedule-paused = Schedule paused.
bot-schedule-resumed = Schedule resumed.

bot-monitor-alert = Alert: { $subject } has changed
bot-monitor-threshold = { $metric } has exceeded threshold: { $value }
bot-monitor-recovered = { $subject } has returned to normal.
bot-monitor-status = Current status: { $status }

bot-order-welcome = Welcome to our store! How can I help?
bot-order-track = Track my order
bot-order-browse = Browse products
bot-order-support = Contact support
bot-order-enter-id = Please enter your order number:
bot-order-status = Order status: { $status }
bot-order-shipped = Your order has been shipped! Tracking number: { $tracking }
bot-order-delivered = Your order has been delivered.
bot-order-processing = Your order is being processed.
bot-order-cancelled = Your order has been cancelled.
bot-order-ticket = Support ticket created: #{ $ticket }
bot-order-products-available = Here are our available products:
bot-order-product-item = { $name } - { $price }
bot-order-cart-added = Added { $product } to your cart.
bot-order-cart-total = Your cart total is { $total }.
bot-order-checkout = Proceeding to checkout...

bot-hr-welcome = HR Assistant here. How can I help?
bot-hr-request-leave = Request leave
bot-hr-check-balance = Check balance
bot-hr-view-policies = View policies
bot-hr-leave-type = What type of leave? (vacation/sick/personal)
bot-hr-start-date = Start date? (YYYY-MM-DD)
bot-hr-end-date = End date? (YYYY-MM-DD)
bot-hr-leave-submitted = Leave request submitted! Your manager will review it.
bot-hr-leave-approved = Your leave request has been approved.
bot-hr-leave-rejected = Your leave request has been rejected.
bot-hr-leave-pending = Your leave request is pending approval.
bot-hr-balance-title = Your leave balance:
bot-hr-vacation-days = Vacation: { $days } days
bot-hr-sick-days = Sick: { $days } days
bot-hr-personal-days = Personal: { $days } days
bot-hr-policy-found = Here's the policy information you requested:
bot-hr-policy-not-found = Policy not found. Please check the policy name.

bot-health-welcome = Welcome to our healthcare center. How can I help?
bot-health-book = Book appointment
bot-health-cancel = Cancel appointment
bot-health-view = View my appointments
bot-health-reschedule = Reschedule appointment
bot-health-type = What type of appointment? (general/specialist/lab)
bot-health-doctor = Which doctor would you prefer?
bot-health-date = What date works best for you?
bot-health-time = What time would you prefer?
bot-health-confirmed = Your appointment has been confirmed for { $datetime } with { $doctor }.
bot-health-cancelled = Your appointment has been cancelled.
bot-health-rescheduled = Your appointment has been rescheduled to { $datetime }.
bot-health-reminder = Reminder: You have an appointment on { $datetime }.
bot-health-no-appointments = You don't have any upcoming appointments.
bot-health-appointments-list = Your upcoming appointments:

bot-support-welcome = How can I help you today?
bot-support-describe = Please describe your issue:
bot-support-category = What category best describes your issue?
bot-support-priority = How urgent is this issue?
bot-support-ticket-created = Support ticket #{ $ticket } has been created.
bot-support-ticket-status = Ticket #{ $ticket } status: { $status }
bot-support-ticket-updated = Your ticket has been updated.
bot-support-ticket-resolved = Your ticket has been resolved. Please let us know if you need further assistance.
bot-support-transfer = Transferring you to a human agent...
bot-support-wait-time = Estimated wait time: { $minutes } minutes.
bot-support-agent-joined = Agent { $name } has joined the conversation.

bot-survey-intro = We'd love to hear your feedback!
bot-survey-question = { $question }
bot-survey-scale = On a scale of 1-10, how would you rate { $subject }?
bot-survey-open = Please share any additional comments:
bot-survey-thanks = Thank you for your feedback!
bot-survey-completed = Survey completed successfully.
bot-survey-skip = You can skip this question if you prefer.

bot-notification-new-message = You have a new message from { $sender }.
bot-notification-task-due = Task "{ $task }" is due { $when }.
bot-notification-reminder = Reminder: { $message }
bot-notification-update = Update: { $message }
bot-notification-alert = Alert: { $message }

bot-command-help = Available commands:
bot-command-unknown = Unknown command. Type 'help' for available commands.
bot-command-invalid = Invalid command syntax. Usage: { $usage }

bot-transfer-to-human = Transferring you to a human agent. Please wait...
bot-transfer-complete = You are now connected with { $agent }.
bot-transfer-unavailable = No agents are currently available. Please try again later.
bot-transfer-queue-position = You are number { $position } in the queue.

bot-auth-login-prompt = Please enter your credentials to continue.
bot-auth-login-success = You have been logged in successfully.
bot-auth-login-failed = Login failed. Please check your credentials.
bot-auth-logout-success = You have been logged out.
bot-auth-session-expired = Your session has expired. Please log in again.

bot-file-upload-prompt = Please upload your file.
bot-file-upload-success = File "{ $filename }" uploaded successfully.
bot-file-upload-failed = Failed to upload file. Please try again.
bot-file-download-ready = Your file is ready for download.
bot-file-processing = Processing your file...

bot-payment-amount = The total amount is { $amount }.
bot-payment-method = Please select a payment method.
bot-payment-processing = Processing your payment...
bot-payment-success = Payment successful! Transaction ID: { $transactionId }
bot-payment-failed = Payment failed. Please try again or use a different payment method.
bot-payment-refund = Your refund of { $amount } has been processed.

bot-subscription-active = Your subscription is active until { $endDate }.
bot-subscription-expired = Your subscription has expired.
bot-subscription-renew = Would you like to renew your subscription?
bot-subscription-upgraded = Your subscription has been upgraded to { $plan }.
bot-subscription-cancelled = Your subscription has been cancelled.

bot-feedback-positive = Thank you for your positive feedback!
bot-feedback-negative = We're sorry to hear that. How can we improve?
bot-feedback-rating = You rated this interaction { $rating } out of 5.
