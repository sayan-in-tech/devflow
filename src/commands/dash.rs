use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, time::Duration};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessesToUpdate, RefreshKind, System};

pub async fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything()),
    );

    loop {
        sys.refresh_memory();
        sys.refresh_cpu_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let cpu = sys.global_cpu_usage();
        let mem = sys.used_memory();
        let procs = sys.processes().len();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(3),
                ])
                .split(f.area());

            let head = Paragraph::new("devflow dash (press q to quit)")
                .block(Block::default().borders(Borders::ALL).title("Status"));
            let health = Paragraph::new(format!(
                "CPU: {:.1}% | MEM: {} KB | PROCS: {}",
                cpu, mem, procs
            ))
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title("System"));
            let body = Paragraph::new(
                "services: local\nports: use devflow port --watch\ntests: use devflow watch",
            )
            .block(Block::default().borders(Borders::ALL).title("Workspace"));
            f.render_widget(head, chunks[0]);
            f.render_widget(health, chunks[1]);
            f.render_widget(body, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(400))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
