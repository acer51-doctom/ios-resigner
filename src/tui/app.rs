use crate::core::{device, sideload};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend, widgets::{Block, Borders, Paragraph}, Layout, Constraint, Direction, Terminal,
};
use std::io;

pub async fn run_tui(ipa_path: String) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut status = String::from("Press 'S' to Start Resign. Press 'Q' to Quit.");
    let mut days_remaining = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(f.size());

            let header = Paragraph::new(format!("Days Remaining: {}", days_remaining))
                .block(Block::default().title("iOS Resigner TUI").borders(Borders::ALL));
            
            let body = Paragraph::new(status.clone())
                .block(Block::default().title("Console").borders(Borders::ALL));

            f.render_widget(header, chunks[0]);
            f.render_widget(body, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('s') => {
                        status = "Pairing device... Make sure to tap Trust!".to_string();
                        // Note: In a real TUI, you'd prompt for Apple ID/Pass here.
                        // Hardcoded for structural example.
                        match device::get_and_pair_device() {
                            Ok(dev) => {
                                status = format!("Paired {}. Signing...", dev.name);
                                match sideload::resign_and_install("user@icloud.com", "password", &ipa_path, &dev.udid).await {
                                    Ok(days) => {
                                        days_remaining = days;
                                        status = "Successfully installed!".to_string();
                                    },
                                    Err(e) => status = format!("Sign Error: {}", e),
                                }
                            },
                            Err(e) => status = format!("Device Error: {}", e),
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}