' Marketing Campaign Automation
USE KB "brand-guidelines"
USE TOOL "social-media-post"
USE TOOL "email-sender"
USE TOOL "analytics-tracker"

ORCHESTRATE WORKFLOW "marketing-campaign"

STEP 1: BOT "audience-segmenter" "analyze target demographics"
  ' AI-powered audience analysis

STEP 2: BOT "content-creator" "generate campaign materials"
  ' Multi-modal content generation

' Smart LLM routing for different content types
email_content = LLM "Create engaging email subject line" 
  WITH OPTIMIZE FOR "cost"

social_content = LLM "Create viral social media post" 
  WITH OPTIMIZE FOR "quality"
  WITH MAX_LATENCY 5000

STEP 3: PARALLEL
  BRANCH A: BOT "email-scheduler" "send email campaign"
  BRANCH B: BOT "social-scheduler" "post to social media"
  BRANCH C: BOT "ad-manager" "launch paid ads"
END PARALLEL

' Wait for initial results
WAIT FOR EVENT "campaign_metrics_ready" TIMEOUT 7200

STEP 4: BOT "performance-analyzer" "analyze results"

IF engagement_rate < 0.02 THEN
  STEP 5: BOT "optimizer" "adjust campaign parameters"
  PUBLISH EVENT "campaign_optimized"
END IF

' Share successful campaign patterns
BOT SHARE MEMORY "high_engagement_content" WITH "content-creator-v2"
BOT SHARE MEMORY "optimal_timing" WITH "scheduler-bots"

PUBLISH EVENT "campaign_complete"

TALK "Marketing campaign launched and optimized!"
