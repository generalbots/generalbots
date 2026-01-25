' E-commerce Order Processing Workflow
USE KB "order-policies"
USE TOOL "validate-payment"
USE TOOL "reserve-inventory"
USE TOOL "send-confirmation"

ORCHESTRATE WORKFLOW "order-processing"

STEP 1: BOT "fraud-detector" "analyze transaction"
  ' AI-powered fraud detection

STEP 2: BOT "inventory-checker" "verify availability"
  ' Check stock levels and reserve items

IF fraud_score > 0.8 THEN
  STEP 3: HUMAN APPROVAL FROM "security@store.com"
    TIMEOUT 900  ' 15 minutes for high-risk orders
    ON TIMEOUT: REJECT ORDER
END IF

IF payment_method = "credit_card" THEN
  STEP 4: BOT "payment-processor" "charge card"
ELSE
  STEP 4: BOT "payment-processor" "process alternative"
END IF

STEP 5: PARALLEL
  BRANCH A: BOT "shipping-optimizer" "select carrier"
  BRANCH B: BOT "inventory-updater" "update stock"
  BRANCH C: BOT "notification-sender" "send confirmation"
END PARALLEL

' Share successful processing patterns
BOT SHARE MEMORY "fraud_indicators" WITH "fraud-detector-backup"
BOT SHARE MEMORY "shipping_preferences" WITH "logistics-bot"

' Publish completion event
PUBLISH EVENT "order_processed"

TALK "Order processed successfully!"
