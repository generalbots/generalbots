# Analytics Dashboard Guide

## Overview

The Analytics Dashboard provides real-time insights into your knowledge base performance, document statistics, and system health metrics.

## Key Metrics

### Document Statistics
- **Total Documents**: Number of documents indexed in your knowledge base
- **Total Vectors**: Number of vector embeddings created for semantic search
- **Storage Used**: Total storage consumption in megabytes
- **Collections**: Number of document collections/categories

### Activity Metrics
- **Documents This Week**: New documents added in the current week
- **Documents This Month**: New documents added in the current month
- **Growth Rate**: Percentage change compared to historical average

### System Health
- **Health Percentage**: Overall system status (100% = all systems operational)
- **Last Update**: Timestamp of the most recent statistics refresh

## How to Use

### Viewing Overview
Ask for an overview to see all key metrics at a glance:
- "Show overview"
- "What's the storage usage?"
- "How many documents do we have?"

### Checking Activity
Monitor recent changes and growth trends:
- "Show recent activity"
- "What's the growth trend?"
- "How many documents were added this week?"

### System Health
Check if all systems are running properly:
- "Show system health"
- "What's the collection status?"
- "Is everything working?"

## Understanding the Data

### Growth Rate Interpretation
- **Positive rate (📈)**: Knowledge base is growing faster than average
- **Negative rate (📉)**: Growth has slowed compared to average
- **Zero rate**: Stable growth matching historical patterns

### Health Status
- **100%**: All systems operational
- **Below 100%**: Some components may need attention
- **Check specific warnings** for details on affected systems

## Scheduled Updates

Statistics are automatically refreshed by the `update-stats` scheduled job. By default, this runs:
- Every hour for activity metrics
- Daily for comprehensive statistics
- On-demand when requested

## Frequently Asked Questions

**Q: Why do I see "Statistics are being computed"?**
A: The analytics system is initializing or refreshing data. Wait a few minutes and try again.

**Q: How accurate are the metrics?**
A: Metrics are updated regularly and reflect the state at the last refresh time shown in "Last Update".

**Q: Can I export analytics data?**
A: Yes, use the "Export Stats" tool to download metrics in various formats.

**Q: What affects system health?**
A: Health reflects database connectivity, vector store status, storage availability, and background job status.

## Best Practices

1. **Regular Monitoring**: Check the dashboard weekly to track growth trends
2. **Storage Planning**: Monitor storage usage to plan capacity needs
3. **Activity Tracking**: Use activity metrics to measure content team productivity
4. **Health Alerts**: Address health issues promptly when below 100%