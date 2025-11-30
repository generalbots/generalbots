' Sales Pipeline Template - Start Script
' Manages sales deals, stages, and pipeline analytics

' ============================================================================
' SETUP TOOLS - Register available sales pipeline tools
' ============================================================================

ADD TOOL "create-deal"
ADD TOOL "update-stage"
ADD TOOL "list-deals"
ADD TOOL "deal-details"
ADD TOOL "pipeline-report"
ADD TOOL "forecast-revenue"

' ============================================================================
' SETUP KNOWLEDGE BASE
' ============================================================================

USE KB "sales-pipeline.gbkb"

' ============================================================================
' SET CONTEXT FOR AI
' ============================================================================

SET CONTEXT "sales pipeline" AS "You are a sales assistant helping manage the sales pipeline. You can help with: creating new deals, updating deal stages, viewing pipeline status, generating sales forecasts, and analyzing win/loss rates. Always be encouraging and help sales reps close more deals."

' ============================================================================
' SETUP SUGGESTIONS
' ============================================================================

CLEAR SUGGESTIONS

ADD SUGGESTION "newdeal" AS "Create a new deal"
ADD SUGGESTION "pipeline" AS "Show my pipeline"
ADD SUGGESTION "update" AS "Update a deal stage"
ADD SUGGESTION "forecast" AS "View sales forecast"
ADD SUGGESTION "report" AS "Generate pipeline report"

' ============================================================================
' WELCOME MESSAGE
' ============================================================================

BEGIN TALK
    💼 **Sales Pipeline Manager**

    Welcome! I'm your sales assistant for managing deals and pipeline.

    **What I can help you with:**
    • ➕ Create new deals and opportunities
    • 📊 View and manage your pipeline
    • 🔄 Update deal stages (Lead → Qualified → Proposal → Negotiation → Closed)
    • 📈 Generate sales forecasts
    • 📋 Pipeline analytics and reports
    • 🏆 Track win/loss rates

    Just tell me what you need or select an option below!
END TALK

' ============================================================================
' SYSTEM PROMPT FOR AI INTERACTIONS
' ============================================================================

BEGIN SYSTEM PROMPT
    You are a sales pipeline assistant. Your responsibilities include:

    1. Helping sales reps create and manage deals
    2. Tracking deal progression through pipeline stages
    3. Providing pipeline visibility and forecasts
    4. Analyzing sales performance metrics
    5. Suggesting next best actions for deals

    Pipeline stages in order:
    - Lead: Initial contact, not yet qualified
    - Qualified: Confirmed budget, authority, need, timeline
    - Proposal: Quote or proposal sent
    - Negotiation: Active discussions on terms
    - Closed Won: Deal successfully closed
    - Closed Lost: Deal lost to competitor or no decision

    Always encourage sales reps and provide actionable insights.
    When updating deals, confirm the changes before saving.
    Use deal values in currency format when displaying amounts.
END SYSTEM PROMPT
