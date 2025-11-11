use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
};
pub struct StreamProgress {
    pub progress: f64,
    pub status: String,
}
pub fn render_progress_bar(progress: &StreamProgress) -> Gauge {
    let color = if progress.progress >= 1.0 {
        Color::Green
    } else {
        Color::Blue
    };
    Gauge::default()
        .block(
            Block::default()
                .title(format!("Stream Progress: {}", progress.status))
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(color))
        .percent((progress.progress * 100.0) as u16)
}
