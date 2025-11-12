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
     fn render_warning(&mut self, message: &str) -> io::Result<()> {
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
