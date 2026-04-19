# Observability

This chapter describes the observability infrastructure that General Bots provides for monitoring system health, collecting logs, and tracking metrics. The observability system operates automatically without requiring code changes, giving administrators visibility into platform behavior and helping identify issues before they impact users.

## Understanding the Observability System

General Bots implements observability through an integrated pipeline that collects, parses, routes, and stores operational data from all system components. The pipeline reads log files from the centralized logs directory within the botserver-stack folder, extracts structured information including log levels, timestamps, and messages, routes different types of data to appropriate destinations such as alerts for errors and storage for metrics, and enriches entries with contextual information like hostnames and service names.

This automated approach means administrators don't need to instrument code or configure complex logging frameworks. The system captures operational data from all components using consistent formats and routes it to useful destinations without manual intervention.

## Log Directory Organization

The logging system organizes output by component within the `./botserver-stack/logs/` directory. System logs from the main botserver application appear in the system subdirectory. Storage service operations are captured in the drive subdirectory. Database activity from PostgreSQL goes to the tables subdirectory. The cache subdirectory contains logs from the caching layer. LLM server interactions are recorded in the llm subdirectory.

Additional services have their own logging locations. Email service logs appear in the email subdirectory. Identity and authentication events are captured in the directory subdirectory. Vector database operations go to the vectordb subdirectory. Video meeting activities are logged in the meet subdirectory.

This organization makes it straightforward to investigate issues in specific components without wading through unrelated log entries.

## Installation and Configuration

The observability component installs automatically during the bootstrap process, ensuring that monitoring begins from the first system start. Administrators who need to install it separately can use the botserver install command with the observability parameter.

Configuration for the observability pipeline resides in the monitoring configuration file within the botserver-stack conf directory. This Vector configuration file controls how logs are collected, parsed, transformed, and routed to their destinations.

## Log Format Conventions

botserver generates logs in a standard format that includes the timestamp in ISO 8601 format with millisecond precision, the log level indicating severity, the module path identifying the code location, and the message describing what occurred. This structured format enables automated parsing while remaining human-readable for direct inspection.

The pipeline parses these logs automatically, extracting fields for indexing and routing. Errors are identified by level and routed to alerting systems while informational messages flow to long-term storage for historical analysis.

## Metrics Collection

The platform exposes operational metrics through a Prometheus-compatible endpoint at `/api/metrics`, enabling integration with standard monitoring infrastructure. Available metrics track log event counts by severity level, error totals broken down by service, currently active session counts, total messages processed since startup, and LLM response latency measurements.

These metrics enable administrators to understand system behavior over time, identify trends that might indicate developing problems, and verify that the platform operates within expected parameters. The Prometheus format ensures compatibility with common visualization and alerting tools.

## Alerting Configuration

The observability system can send alerts automatically when error conditions occur. Webhook alerts POST event data to the admin alerts API endpoint, enabling integration with custom alerting systems. Slack integration sends notifications to configured channels when properly configured. Email alerts reach administrators directly when SMTP settings are provided.

Alert thresholds are configurable through the bot's config.csv file. The CPU threshold setting triggers alerts when processor utilization exceeds the specified percentage. Memory threshold configuration works similarly for RAM usage. Response time thresholds flag slow operations that might indicate performance degradation.

Tuning these thresholds for your environment prevents alert fatigue from false positives while ensuring genuine issues receive attention.

## Dashboard Visualization

A pre-built Grafana dashboard template is available in the templates directory, providing immediate visualization of key metrics. The dashboard includes panels for active sessions showing current load, messages per minute indicating throughput, error rates highlighting problems, and LLM latency percentiles revealing AI response performance.

Importing this dashboard into a Grafana instance connected to your metrics storage creates an operational overview suitable for operations teams and helps during incident investigation.

## Log Level Configuration

The logging system supports four severity levels that control which messages are captured and the volume of output generated.

Error level captures failures that require attention, such as database connection losses or file permission problems. Warning level records unexpected conditions that were handled but might indicate developing issues. Info level logs normal operations and key events, providing a record of system activity without excessive detail. Debug level includes detailed flow information useful during development and troubleshooting but too verbose for normal production operation.

The log level setting in config.csv controls the minimum severity that produces output. Setting it to info captures everything except debug messages, providing operational visibility without overwhelming log storage.

## Troubleshooting Common Issues

When logs aren't being collected as expected, several common causes should be investigated. First, verify that the observability service is running and hasn't crashed or been stopped. Second, check that the log directory permissions allow the collection process to read the files. Third, review the observability service's own logs for errors that might explain the collection failure.

High log volume can overwhelm storage and make analysis difficult. Raising the log level from debug to info significantly reduces volume by eliminating detailed trace messages. Configuring retention policies in the metrics storage prevents unbounded growth. Filtering debug-level logs before they reach long-term storage reduces costs while preserving important operational data.

## Operational Guidelines

Effective observability requires attention to both technical configuration and operational practices. Log content should never include sensitive data like passwords, tokens, or personally identifiable information, as logs often flow to systems with broader access than the application itself.

Using appropriate log levels keeps signal-to-noise ratios manageable. Reserve error level for actual failures requiring investigation. Use info level for normal operations that help understand system behavior. Avoid overusing warning level, which loses meaning when applied too broadly.

Monitoring should focus on trends rather than just instantaneous values. Gradual increases in error rates or response times often indicate developing problems before they become critical failures. Alert configuration should consider baseline behavior and flag deviations rather than simple threshold crossings.

Establishing observability early in deployment ensures that baseline data exists when problems occur. Trying to instrument a system during an incident rarely produces useful results.

## Related Documentation

For additional context on operating General Bots at scale, the Scaling and Load Balancing chapter explains how observability integrates with clustered deployments. The Infrastructure Design chapter provides the full architectural overview showing how observability fits into the complete system. The Monitoring Dashboard section describes the built-in monitoring interface available through the administrative UI.