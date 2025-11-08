use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};
use crate::nvidia::get_system_metrics;

pub struct BotUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl BotUI {
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn render_progress(&mut self, current_tokens: usize, max_context_size: usize) -> io::Result<()> {
        let metrics = get_system_metrics(current_tokens, max_context_size).unwrap_or_default();
        let gpu_usage = metrics.gpu_usage.unwrap_or(0.0);
        let cpu_usage = metrics.cpu_usage;
        let token_ratio = current_tokens as f64 / max_context_size.max(1) as f64;

        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(f.area());

            let gpu_gauge = Gauge::default()
                .block(Block::default().title("GPU Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .ratio(gpu_usage as f64 / 100.0)
                .label(format!("{:.1}%", gpu_usage));

            let cpu_gauge = Gauge::default()
                .block(Block::default().title("CPU Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .ratio(cpu_usage as f64 / 100.0)
                .label(format!("{:.1}%", cpu_usage));

            let token_gauge = Gauge::default()
                .block(Block::default().title("Token Progress").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .ratio(token_ratio)
                .label(format!("{}/{}", current_tokens, max_context_size));

            f.render_widget(gpu_gauge, chunks[0]);
            f.render_widget(cpu_gauge, chunks[1]);
            f.render_widget(token_gauge, chunks[2]);
        })?;
        Ok(())
    }

    pub fn render_warning(&mut self, message: &str) -> io::Result<()> {
        self.terminal.draw(|f| {
            let block = Block::default()
                .title("⚠️ NVIDIA Warning")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));
            let paragraph = Paragraph::new(message)
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .block(block);
            f.render_widget(paragraph, f.area());
        })?;
        Ok(())
    }
}
