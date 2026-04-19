notification-title-new-message = New Message
notification-title-task-due = Task Due
notification-title-task-assigned = Task Assigned
notification-title-task-completed = Task Completed
notification-title-meeting-reminder = Meeting Reminder
notification-title-meeting-started = Meeting Started
notification-title-file-shared = File Shared
notification-title-file-uploaded = File Uploaded
notification-title-comment-added = New Comment
notification-title-mention = You were mentioned
notification-title-system = System Notification
notification-title-security = Security Alert
notification-title-update = Update Available
notification-title-error = Error Occurred
notification-title-success = Success
notification-title-warning = Warning
notification-title-info = Information

notification-message-new = You have a new message from { $sender }
notification-message-unread = You have { $count ->
    [one] { $count } unread message
   *[other] { $count } unread messages
}
notification-task-due-soon = Task "{ $task }" is due in { $time }
notification-task-due-today = Task "{ $task }" is due today
notification-task-due-overdue = Task "{ $task }" is overdue by { $time }
notification-task-assigned-to-you = You have been assigned to task "{ $task }"
notification-task-assigned-by = { $assigner } assigned you to "{ $task }"
notification-task-completed-by = { $user } completed task "{ $task }"
notification-task-status-changed = Task "{ $task }" status changed to { $status }

notification-meeting-in-minutes = Meeting "{ $meeting }" starts in { $minutes } minutes
notification-meeting-starting-now = Meeting "{ $meeting }" is starting now
notification-meeting-cancelled = Meeting "{ $meeting }" has been cancelled
notification-meeting-rescheduled = Meeting "{ $meeting }" has been rescheduled to { $datetime }
notification-meeting-invite = { $inviter } invited you to "{ $meeting }"
notification-meeting-response = { $user } { $response } your meeting invite

notification-file-shared-with-you = { $sharer } shared "{ $filename }" with you
notification-file-uploaded-by = { $uploader } uploaded "{ $filename }"
notification-file-modified = "{ $filename }" was modified by { $user }
notification-file-deleted = "{ $filename }" was deleted by { $user }
notification-file-download-ready = Your file "{ $filename }" is ready for download
notification-file-upload-complete = Upload of "{ $filename }" completed successfully
notification-file-upload-failed = Upload of "{ $filename }" failed

notification-comment-on-task = { $user } commented on task "{ $task }"
notification-comment-on-file = { $user } commented on "{ $filename }"
notification-comment-reply = { $user } replied to your comment
notification-mention-in-comment = { $user } mentioned you in a comment
notification-mention-in-chat = { $user } mentioned you in { $channel }

notification-login-new-device = New login detected from { $device } in { $location }
notification-login-failed = Failed login attempt on your account
notification-password-changed = Your password was changed successfully
notification-password-expiring = Your password will expire in { $days } days
notification-session-expired = Your session has expired
notification-account-locked = Your account has been locked
notification-two-factor-enabled = Two-factor authentication has been enabled
notification-two-factor-disabled = Two-factor authentication has been disabled

notification-subscription-expiring = Your subscription expires in { $days } days
notification-subscription-expired = Your subscription has expired
notification-subscription-renewed = Your subscription has been renewed until { $date }
notification-payment-successful = Payment of { $amount } was successful
notification-payment-failed = Payment of { $amount } failed
notification-invoice-ready = Your invoice for { $period } is ready

notification-bot-response = { $bot } responded to your query
notification-bot-error = { $bot } encountered an error
notification-bot-offline = { $bot } is currently offline
notification-bot-online = { $bot } is now online
notification-bot-updated = { $bot } has been updated

notification-system-maintenance = System maintenance scheduled for { $datetime }
notification-system-update = System update available: { $version }
notification-system-restored = System has been restored
notification-system-degraded = System is experiencing degraded performance

notification-action-view = View
notification-action-dismiss = Dismiss
notification-action-mark-read = Mark as read
notification-action-mark-all-read = Mark all as read
notification-action-settings = Notification settings
notification-action-reply = Reply
notification-action-open = Open
notification-action-join = Join
notification-action-accept = Accept
notification-action-decline = Decline

notification-time-just-now = Just now
notification-time-minutes = { $count ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
notification-time-hours = { $count ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
notification-time-days = { $count ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
notification-time-weeks = { $count ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}

notification-preference-all = All notifications
notification-preference-important = Important only
notification-preference-none = None
notification-preference-email = Email notifications
notification-preference-push = Push notifications
notification-preference-in-app = In-app notifications
notification-preference-sound = Sound enabled
notification-preference-vibration = Vibration enabled

notification-empty = No notifications
notification-empty-description = You're all caught up!
notification-load-more = Load more
notification-clear-all = Clear all notifications
notification-filter-all = All
notification-filter-unread = Unread
notification-filter-mentions = Mentions
notification-filter-tasks = Tasks
notification-filter-messages = Messages
notification-filter-system = System
