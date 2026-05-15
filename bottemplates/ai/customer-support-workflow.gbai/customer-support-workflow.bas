' Example: Customer Support Workflow with Enhanced Orchestration
' This demonstrates the new ORCHESTRATE WORKFLOW, event system, and bot memory sharing

USE KB "support-policies"
USE TOOL "check-order"
USE TOOL "process-refund"

' Set up event handlers
ON EVENT "approval_received" DO
  TALK "Manager approval received, processing refund..."
END ON

ON EVENT "timeout_occurred" DO
  TALK "Approval timeout, escalating to director..."
END ON

' Main workflow orchestration
ORCHESTRATE WORKFLOW "customer-complaint-resolution"

STEP 1: BOT "classifier" "analyze complaint"
  ' Classifier bot analyzes the complaint and sets variables

STEP 2: BOT "order-checker" "validate order details"
  ' Order checker validates the order and warranty status

' Conditional logic based on order value
IF order_amount > 100 THEN
  STEP 3: HUMAN APPROVAL FROM "manager@company.com"
    TIMEOUT 1800  ' 30 minutes
    ON TIMEOUT: ESCALATE TO "director@company.com"
  
  ' Wait for approval event
  WAIT FOR EVENT "approval_received" TIMEOUT 3600
END IF

STEP 4: PARALLEL
  BRANCH A: BOT "refund-processor" "process refund"
  BRANCH B: BOT "inventory-updater" "update stock levels"
END PARALLEL

STEP 5: BOT "follow-up" "schedule customer check-in"
  DELAY 86400  ' 24 hours later

' Share successful resolution patterns with other support bots
BOT SHARE MEMORY "successful_resolution_method" WITH "support-bot-2"
BOT SHARE MEMORY "customer_satisfaction_score" WITH "support-bot-3"

' Sync knowledge from master support bot
BOT SYNC MEMORY FROM "master-support-bot"

' Publish completion event for analytics
PUBLISH EVENT "workflow_completed"

TALK "Customer complaint resolved successfully!"
