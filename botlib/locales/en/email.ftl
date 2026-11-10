# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = Hello { $name },
email-signoff = Best regards,
email-team = General Bots Team
email-welcome-subject = Welcome to General Bots Cloud — { $plan } Plan
email-welcome-body = Welcome to General Bots Cloud! Your { $plan } plan is now active. You can start creating bots and managing your workspace at { $url }
email-invoice-subject = Invoice #{ $invoice } — General Bots Cloud
email-invoice-body = A new invoice has been created. Invoice: { $invoice }. Amount: { $amount } { $currency }. View the details at { $url }
email-payment-success-subject = Payment Confirmed — General Bots Cloud
email-payment-success-body = Your payment of { $amount } { $currency } has been confirmed. Plan: { $plan }. Thank you for your subscription!
email-subscription-activated-subject = Subscription Activated — General Bots Cloud
email-subscription-activated-body = Your { $plan } subscription is now active. You can manage it at { $url }
email-recurring-charge-subject = Recurring Charge — Invoice #{ $invoice }
email-recurring-charge-body = Your recurring subscription has been charged. Invoice: { $invoice }. Amount: { $amount } { $currency }. View the details at { $url }
email-payment-failed-subject = Payment Failed — Action Required
email-payment-failed-body = Your payment of { $amount } { $currency } has failed. Invoice: { $invoice }. Please update your payment method at { $url }
email-subscription-cancelled-subject = Subscription Cancelled — General Bots Cloud
email-subscription-cancelled-body = Your { $plan } subscription has been cancelled. If you would like to reactivate it, visit { $url }
email-grace-warning-subject = Payment Overdue — Service Suspension Warning
email-grace-warning-body = Your payment is overdue. Your service will be suspended in { $days } days. Please update your payment method immediately at { $url }
email-invitation-subject = Invitation to join { $org }
email-invitation-body = You have been invited to join { $org }. Accept your invitation at { $url }. This invitation expires at { $expires }.
email-quota-limit-reached = You have reached your { $metric } quota limit.
email-quota-threshold = You have used { $current }% of your { $metric } quota (threshold: { $threshold }%).
email-metric-messages = messages
email-metric-storage = storage
email-metric-api-calls = API calls
email-metric-bots = bots
email-metric-users = users
email-metric-kb-documents = knowledge base documents
email-metric-apps = apps
email-metric-emails-per-day = emails per day
email-metric-builds-per-day = builds per day
email-metric-uploads-per-hour = uploads per hour

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } shared "{ $file }" with you
email-share-download = Download { $file }
email-share-link-expires = This link expires on { $date }. You can revoke it at any time.
email-share-link-no-expiry = This link does not expire. You can revoke it at any time.
email-mention-subject = { $author } mentioned you in a comment
email-mention-body = { $author } mentioned you on { $resource_type } ({ $resource_id }). "{ $preview }" Open the app to view and reply.
