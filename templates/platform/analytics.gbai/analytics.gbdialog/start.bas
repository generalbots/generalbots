' =============================================================================
' Analytics Bot - Platform Metrics and Reporting Dialog
' General Bots Template for Platform Analytics
' =============================================================================
' This template provides analytics capabilities for:
' - Platform usage metrics
' - Performance monitoring
' - Custom report generation
' - Multi-agent analytics queries
' =============================================================================

TALK "Welcome to the Analytics Center. I can help you understand your platform metrics and generate reports."

TALK "What would you like to analyze?"
TALK "1. Platform Overview - Key metrics summary"
TALK "2. Message Analytics - Conversation statistics"
TALK "3. User Analytics - Active users and sessions"
TALK "4. Performance Metrics - Response times and throughput"
TALK "5. LLM Usage - Token consumption and costs"
TALK "6. Storage Analytics - Disk usage and file statistics"
TALK "7. Error Analysis - Error patterns and trends"
TALK "8. Generate Custom Report"

HEAR choice AS INTEGER

SELECT CASE choice
    CASE 1
        CALL "platform-overview.bas"

    CASE 2
        CALL "message-analytics.bas"

    CASE 3
        CALL "user-analytics.bas"

    CASE 4
        CALL "performance-metrics.bas"

    CASE 5
        CALL "llm-usage.bas"

    CASE 6
        CALL "storage-analytics.bas"

    CASE 7
        CALL "error-analysis.bas"

    CASE 8
        CALL "custom-report.bas"

    CASE ELSE
        SET CONTEXT "You are an analytics assistant. Help the user understand platform metrics. Available data: messages, sessions, response_time, llm_tokens, storage, errors. Answer questions about trends, patterns, and performance."

        HEAR query AS TEXT

        response = LLM "Analyze this analytics query and provide insights: " + query
        TALK response
END SELECT
