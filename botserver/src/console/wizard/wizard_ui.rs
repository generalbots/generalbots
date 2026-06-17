use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use super::wizard_core::{ComponentChoice, StartupWizard};

impl StartupWizard {
    pub(super) fn show_welcome(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        let _ = self; // kept for API consistency
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let banner = r"
    ╔══════════════════════════════════════════════════════════════════╗
    ║                                                                  ║
    ║     ██████╗ ███████╗███╗   ██╗███████╗██████╗  █████╗ ██╗       ║
    ║    ██╔════╝ ██╔════╝████╗  ██║██╔════╝██╔══██╗██╔══██╗██║       ║
    ║    ██║  ███╗█████╗  ██╔██╗ ██║█████╗  ██████╔╝███████║██║       ║
    ║    ██║   ██║██╔══╝  ██║╚██╗██║██╔══╝  ██╔══██╗██╔══██║██║       ║
    ║    ╚██████╔╝███████╗██║ ╚████║███████╗██║  ██║██║  ██║███████╗  ║
    ║     ╚═════╝ ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝  ║
    ║                      ██████╗  ██████╗ ████████╗███████╗          ║
    ║                      ██╔══██╗██╔═══██╗╚══██╔══╝██╔════╝          ║
    ║                      ██████╔╝██║   ██║   ██║   ███████╗          ║
    ║                      ██╔══██╗██║   ██║   ██║   ╚════██║          ║
    ║                      ██████╔╝╚██████╔╝   ██║   ███████║          ║
    ║                      ╚═════╝  ╚═════╝    ╚═╝   ╚══════╝          ║
    ║                                                                  ║
    ╚══════════════════════════════════════════════════════════════════╝
";

        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print(banner),
            ResetColor
        )?;

        execute!(
            stdout,
            cursor::MoveTo(20, 18),
            SetForegroundColor(Color::Cyan),
            Print(format!(
                "Welcome to {} Setup Wizard v{}",
                "General Bots",
                botlib::version::BOTSERVER_VERSION
            )),
            ResetColor
        )?;

        execute!(
            stdout,
            cursor::MoveTo(20, 20),
            Print("This wizard will help you configure your bot server."),
            cursor::MoveTo(20, 21),
            Print("You can re-run this wizard anytime with: "),
            SetForegroundColor(Color::Yellow),
            Print("botserver --wizard"),
            ResetColor
        )?;

        execute!(
            stdout,
            cursor::MoveTo(20, 24),
            SetForegroundColor(Color::DarkGrey),
            Print("Press ENTER to continue..."),
            ResetColor
        )?;

        stdout.flush()?;
        Ok(())
    }

    pub(super) fn show_step_header(&self, stdout: &mut io::Stdout, title: &str) -> io::Result<()> {
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let progress = format!("Step {}/{}: {}", self.current_step, self.total_steps, title);
        let bar_width = 50;
        let filled = (self.current_step * bar_width) / self.total_steps;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("╔"),
            Print("═".repeat(bar_width + 2)),
            Print("╗\n"),
            Print("║ "),
            SetForegroundColor(Color::Green),
            Print("█".repeat(filled)),
            SetForegroundColor(Color::DarkGrey),
            Print("░".repeat(bar_width - filled)),
            SetForegroundColor(Color::Cyan),
            Print(" ║\n"),
            Print("╚"),
            Print("═".repeat(bar_width + 2)),
            Print("╝"),
            ResetColor
        )?;

        execute!(
            stdout,
            cursor::MoveTo(0, 4),
            SetForegroundColor(Color::White),
            Print(format!("  {}\n", progress)),
            ResetColor,
            Print("\n")
        )?;

        stdout.flush()?;
        Ok(())
    }

    pub(super) fn select_option<T: Clone>(
        &self,
        stdout: &mut io::Stdout,
        options: &[(&str, &str, T)],
        default: usize,
    ) -> io::Result<usize> {
        let _ = self; // kept for API consistency
        let mut selected = default;
        let start_row = 10;

        loop {
            for (i, (name, desc, _)) in options.iter().enumerate() {
                execute!(stdout, cursor::MoveTo(4, start_row + i as u16))?;

                if i == selected {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Green),
                        Print("> "),
                        Print(format!("{:<25}", name)),
                        SetForegroundColor(Color::DarkGrey),
                        Print(format!(" {}", desc)),
                        ResetColor
                    )?;
                } else {
                    execute!(
                        stdout,
                        Print("  "),
                        Print(format!("{:<25}", name)),
                        SetForegroundColor(Color::DarkGrey),
                        Print(format!(" {}", desc)),
                        ResetColor
                    )?;
                }
            }

            stdout.flush()?;

            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up => {
                        selected = selected.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if selected < options.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
                    }
                    _ => {}
                }
            }
        }

        Ok(selected)
    }

    pub(super) fn multi_select(
        &self,
        stdout: &mut io::Stdout,
        options: &[(ComponentChoice, bool, bool)],
    ) -> io::Result<Vec<ComponentChoice>> {
        let _ = self; // kept for API consistency
        let mut selected: Vec<bool> = options.iter().map(|(_, s, _)| *s).collect();
        let mut cursor = 0;
        let start_row = 10;

        loop {
            for (i, (component, _, can_toggle)) in options.iter().enumerate() {
                execute!(stdout, cursor::MoveTo(4, start_row + i as u16))?;

                let checkbox = if selected[i] { "[*]" } else { "[ ]" };
                let prefix = if i == cursor { ">" } else { " " };

                if !can_toggle {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::DarkGrey),
                        Print(format!("{} {} {} (required)", prefix, checkbox, component)),
                        ResetColor
                    )?;
                } else if i == cursor {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Green),
                        Print(format!("{} {} {}", prefix, checkbox, component)),
                        ResetColor
                    )?;
                } else {
                    execute!(
                        stdout,
                        Print(format!("{} {} {}", prefix, checkbox, component)),
                    )?;
                }
            }

            execute!(
                stdout,
                cursor::MoveTo(4, start_row + options.len() as u16 + 2),
                SetForegroundColor(Color::DarkGrey),
                Print("Use ↑↓ to navigate, SPACE to toggle, ENTER to confirm"),
                ResetColor
            )?;

            stdout.flush()?;

            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up => {
                        cursor = cursor.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if cursor < options.len() - 1 {
                            cursor += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if options[cursor].2 {
                            selected[cursor] = !selected[cursor];
                        }
                    }
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        return Err(io::Error::new(
                            io::ErrorKind::Interrupted,
                            "Cancelled",
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(options
            .iter()
            .enumerate()
            .filter(|(i, _)| selected[*i])
            .map(|(_, (c, _, _))| c.clone())
            .collect())
    }

    pub(super) fn wait_for_enter(&self) -> io::Result<()> {
        let _ = self; // kept for API consistency
        loop {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                if code == KeyCode::Enter {
                    break;
                }
            }
        }
        Ok(())
    }
}
