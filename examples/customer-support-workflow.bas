' Example Workflow: Customer Support Process
USE KB "support-policies"
USE TOOL "check-order"
USE TOOL "process-refund"

ORCHESTRATE WORKFLOW "customer-support"
  STEP 1: BOT "classifier" "analyze complaint"
  STEP 2: BOT "order-checker" "validate order"
  
  IF order_amount > 100 THEN
    STEP 3: HUMAN APPROVAL FROM "manager@company.com"
      TIMEOUT 1800
  END IF
  
  STEP 4: BOT "refund-processor" "process refund"
  
  BOT SHARE MEMORY "resolution_method" WITH "support-team"
  PUBLISH EVENT "case_resolved"
END WORKFLOW

TALK "Support case processed successfully!"
