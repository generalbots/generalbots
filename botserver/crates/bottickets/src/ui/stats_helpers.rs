use super::*;

/// Mean resolution time in hours across the tickets that carry a
/// `resolved_at` timestamp. Tickets still open are ignored, so the average is
/// never diluted by unresolved rows; `None` when nothing was resolved yet.
pub(super) fn average_resolution_hours(tickets: &[SupportTicket]) -> Option<f64> {
    let mut total_hours = 0.0;
    let mut resolved_count = 0_usize;
    for ticket in tickets {
        if let Some(resolved_at) = ticket.resolved_at {
            let minutes = (resolved_at - ticket.created_at).num_minutes();
            total_hours += minutes as f64 / 60.0;
            resolved_count += 1;
        }
    }
    if resolved_count == 0 {
        None
    } else {
        Some(total_hours / resolved_count as f64)
    }
}

/// Renders a resolution span: hours below one day, days from one day up.
/// Sub-hour spans keep their fraction (a 30-minute resolution is "0.5h", not
/// "0.0h"), which is why the average is computed in minutes, not whole hours.
pub(super) fn format_resolution_span(hours: f64) -> String {
    if hours < 24.0 {
        format!("{hours:.1}h")
    } else {
        format!("{:.1}d", hours / 24.0)
    }
}
