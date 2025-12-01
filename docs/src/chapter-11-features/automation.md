# Automation

This chapter explains how BotServer enables bots to perform scheduled and event-driven tasks without requiring direct user interaction. Through automation capabilities, bots can proactively maintain content, process data, and respond to system events, transforming them from reactive assistants into active participants in organizational workflows.

## Automation Fundamentals

BotServer implements automation through two complementary mechanisms. The SET SCHEDULE keyword enables cron-based task scheduling, allowing scripts to execute at predetermined times. Event triggers through the ON keyword enable scripts to respond to database changes and system events. Together, these mechanisms support a wide range of automation scenarios from simple periodic tasks to complex event-driven workflows.

Understanding when to use automation is as important as understanding how. Automated tasks run without an active user session, which means they cannot directly communicate with users through TALK statements. Instead, automated scripts typically gather and process information, storing results in bot memory where users can access it during their next interaction.

## Scheduling Tasks with SET SCHEDULE

The SET SCHEDULE keyword registers a script for periodic execution using standard cron expressions. When the BASIC compiler encounters this keyword, it extracts the schedule specification and creates an entry in the system_automations table. A background service monitors this table and executes scripts when their scheduled times arrive.

Cron expressions follow the standard Unix format with five fields representing minutes, hours, day of month, month, and day of week. The expression `0 9 * * *` means "at minute 0 of hour 9, every day of every month, regardless of day of week"—in other words, daily at 9:00 AM. The expression `*/30 * * * *` means "every 30 minutes" by using the step syntax. More complex patterns like `0 9 * * 1-5` specify "weekdays at 9 AM" by restricting the day of week field to Monday through Friday.

Scheduled scripts execute with full bot context and permissions, but without an associated user session. This means they can access bot memory, call external APIs, read and write files, and perform data processing. However, they cannot use TALK to send messages since there's no user to receive them. Results should be stored in bot memory for later retrieval or sent through other channels like email.

## Practical Scheduling Examples

A daily report generation script illustrates common automation patterns. The script specifies its schedule, retrieves data from the previous day, processes it using LLM analysis, and stores the result in bot memory. When users later ask about the daily report, the bot can retrieve and present this pre-computed summary without delay.

Content update automation keeps information fresh without manual intervention. A news aggregation script might run every six hours, fetching latest headlines, summarizing them, and caching the result. Users interacting with the bot receive current information even if nobody has explicitly updated the content.

Maintenance tasks handle housekeeping that shouldn't require human attention. Cleanup scripts can run during low-activity periods to archive old data, remove temporary files, or perform consistency checks. These tasks keep the system healthy without consuming resources during peak usage times.

Data synchronization scripts bridge external systems with bot knowledge. A script might periodically fetch updates from a CRM, inventory system, or other business application, ensuring the bot's responses reflect current organizational reality.

## Event-Driven Automation

The ON keyword creates triggers that fire when specific events occur rather than at scheduled times. Currently, the system supports database event triggers that respond to table modifications. When the specified event occurs, the associated code block executes.

Event triggers complement scheduled tasks by enabling immediate response to changes rather than waiting for the next scheduled run. While a scheduled task might check for new registrations hourly, an event trigger fires immediately when a registration occurs, enabling real-time automation workflows.

The system stores triggers in the same system_automations table as scheduled tasks, distinguished by their trigger kind. Each trigger specifies its target (the table or resource being monitored), parameters controlling its behavior, and an activation flag allowing temporary disabling without deletion.

## The System Automations Table

The system_automations table serves as the central registry for all automation rules. Each record contains a unique identifier, the bot that owns the automation, the kind of trigger (scheduled or event-driven), the cron expression for scheduled tasks, parameters such as script names, an active flag, and a timestamp tracking the last execution.

This centralized storage allows the background scheduler to efficiently query upcoming tasks across all bots. It also enables administrative monitoring of automation activity and troubleshooting of failed executions.

## Automation Lifecycle Management

Understanding how automations are created, executed, modified, and removed helps administrators manage bot deployments effectively.

During script compilation, the preprocessor detects SET SCHEDULE statements and extracts their cron expressions. The system creates or updates corresponding entries in the system_automations table. If a script previously had a schedule that was removed, the old automation entry is deleted.

When execution time arrives, the scheduler loads the bot's context, executes the BASIC script, updates the last_triggered timestamp, and logs the execution result. Any errors during execution are captured and logged but don't affect other scheduled tasks.

Modifying a schedule requires only changing the SET SCHEDULE line in the script. The next compilation updates the database entry automatically. This approach keeps schedule definitions with their associated code rather than requiring separate configuration management.

Deleting a bot cascades to remove all its automations, preventing orphaned schedules that would fail at execution time.

## Best Practices for Automation

Effective automation requires thoughtful design decisions. Scheduling frequency should match actual needs—running a task every minute when hourly would suffice wastes resources and can mask problems. Consider what would happen if a task takes longer than its scheduling interval, as overlapping executions can cause unexpected behavior.

Error handling in automated scripts is particularly important because no user is present to observe failures. Scripts should catch exceptions, log meaningful error messages, and degrade gracefully when dependencies are unavailable. Consider storing error states in bot memory so users can be informed of issues during their next interaction.

Scripts should be tested manually before enabling scheduling. Running a script interactively verifies that it works correctly and helps identify issues that might not be apparent from logs alone.

Bot memory serves as the bridge between automated tasks and user interactions. Automated scripts store their results in bot memory, making that information available to all users. This pattern works well for information that benefits from pre-computation, like summarized reports or aggregated statistics.

External credentials should never be hardcoded in scripts. Use bot memory to store API keys and other secrets, retrieving them at runtime. This practice improves security and simplifies credential rotation.

## Understanding Limitations

Several constraints affect automation design decisions. The minimum scheduling granularity is one minute, as the cron format doesn't support sub-minute precision. Tasks requiring more frequent execution need alternative approaches.

Each scheduled execution has timeout limits to prevent runaway tasks from consuming resources indefinitely. Long-running processes should be designed to complete within these limits or broken into smaller pieces.

The system doesn't provide automatic retry on failure. If a scheduled task fails, it simply waits for the next scheduled time. Scripts needing retry behavior must implement it internally.

Only one instance of a scheduled script runs at a time. If execution takes longer than the scheduling interval, subsequent invocations are skipped rather than queued. This prevents resource exhaustion but means some scheduled times may be missed.

There's no dependency management between scheduled tasks. If one task must complete before another begins, scripts must coordinate through bot memory or other synchronization mechanisms.

## Monitoring Automated Tasks

Observing automation behavior helps identify problems and optimize performance. Active schedules can be queried directly from the system_automations table, filtered by bot and trigger kind. The last_triggered timestamp shows when each automation last executed successfully.

Execution logging captures both successful runs and failures at appropriate log levels. Monitoring these logs reveals patterns like consistently slow executions or recurring errors that might not be apparent from individual runs.

Debug logging at lower levels captures schedule changes during compilation, helping trace unexpected automation behavior to its source. Enabling debug logging temporarily can help diagnose why a schedule isn't executing as expected.

## Debugging Automation Issues

When automated tasks don't behave as expected, systematic investigation identifies the cause. Common issues include invalid cron expressions that never match, scripts that work interactively but fail without a user session, external resources that are unavailable when the script runs, and permission issues that only manifest in the automation context.

Verifying the cron expression syntax ensures the schedule means what you intend. Online cron expression validators can help confirm that expressions match expected execution times.

Testing scripts manually with explicit handling for the missing user session helps identify code that incorrectly assumes user context. Any TALK statements will fail in automated context, and scripts must work correctly without user input.

Checking external resource availability at scheduled times reveals dependencies that might not be available around the clock. Business APIs often have maintenance windows, and network conditions vary throughout the day.

Reviewing permissions ensures the bot has access to all resources the automated script needs. Permissions that work for interactive users might not apply to automated execution contexts.

## Security Considerations

Automated tasks execute with the bot's full permissions, making them powerful but requiring careful design. Scripts can access any data the bot can access, call any API the bot is authorized to use, and store results in any location the bot can write.

This power means automated scripts should be reviewed carefully before deployment. Malicious or buggy automation could exfiltrate data, overwhelm external services, or fill storage with garbage. Limiting automation privileges isn't possible in the current system, so careful script review is the primary safeguard.

Rate limiting applies to automated tasks just as it does to interactive use. Aggressive scheduling that exceeds API limits will be throttled, potentially causing tasks to fail or take longer than expected.

Monitoring for runaway automation helps catch scripts that behave differently than expected. Unusual resource consumption, excessive API calls, or unexpected storage growth might indicate automation problems requiring intervention.

## Summary

BotServer's automation capabilities transform bots from reactive assistants into proactive system participants. Through SET SCHEDULE and event triggers, bots can maintain fresh content, process data regularly, and respond to system events without user interaction. Understanding the automation lifecycle, limitations, and best practices enables effective use of these powerful capabilities while avoiding common pitfalls. Automation extends bot value by handling routine tasks automatically, freeing users to focus on work that requires human judgment.