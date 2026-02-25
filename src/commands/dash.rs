use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table},
    Terminal,
};
use std::{io, time::Duration};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessesToUpdate, RefreshKind, System};

const LOGO: &str = r"  ██████╗ ███████╗██╗   ██╗███████╗██╗      ██████╗ ██╗    ██╗
  ██╔══██╗██╔════╝██║   ██║██╔════╝██║     ██╔═══██╗██║    ██║
  ██║  ██║█████╗  ██║   ██║█████╗  ██║     ██║   ██║██║ █╗ ██║
  ██║  ██║██╔══╝  ╚██╗ ██╔╝██╔══╝  ██║     ██║   ██║██║███╗██║
  ██████╔╝███████╗ ╚████╔╝ ██║     ███████╗╚██████╔╝╚███╔███╔╝
  ╚═════╝ ╚══════╝  ╚═══╝  ╚═╝     ╚══════╝ ╚═════╝  ╚══╝╚══╝";

const SPIN: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

fn gauge_color(pct: f64) -> Color {
    if pct > 80.0 {
        Color::Red
    } else if pct > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn format_uptime(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if d > 0 {
        format!("{}d {:02}h {:02}m {:02}s", d, h, m, s)
    } else if h > 0 {
        format!("{:02}h {:02}m {:02}s", h, m, s)
    } else {
        format!("{:02}m {:02}s", m, s)
    }
}

fn format_bytes(bytes: u64) -> String {
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gb >= 1.0 {
        format!("{:.2} GB", gb)
    } else {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("{:.0} MB", mb)
    }
}

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

    let mut cpu_history: Vec<u64> = Vec::new();
    let history_max: usize = 200;
    let mut tick: usize = 0;

    // Static system info (fetched once)
    let sys_name = System::name().unwrap_or_else(|| "Unknown".into());
    let host_name = System::host_name().unwrap_or_else(|| "Unknown".into());
    let os_version = System::os_version().unwrap_or_else(|| "N/A".into());

    let cyan = Style::default().fg(Color::Cyan);

    loop {
        sys.refresh_memory();
        sys.refresh_cpu_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let cpu = sys.global_cpu_usage();
        let mem_used = sys.used_memory();
        let mem_total = sys.total_memory();
        let swap_used = sys.used_swap();
        let swap_total = sys.total_swap();
        let uptime = System::uptime();

        // Collect owned process data to avoid borrow issues in the draw closure
        let mut proc_data: Vec<(String, String, f32, u64)> = sys
            .processes()
            .values()
            .map(|p| {
                (
                    format!("{}", p.pid()),
                    p.name().to_string_lossy().to_string(),
                    p.cpu_usage(),
                    p.memory(),
                )
            })
            .collect();
        proc_data.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        proc_data.truncate(50);

        let proc_count = sys.processes().len();

        // Per-core CPU data
        let core_data: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let core_count = core_data.len();

        // Memory percentages
        let mem_pct = if mem_total > 0 {
            (mem_used as f64 / mem_total as f64) * 100.0
        } else {
            0.0
        };
        let swap_pct = if swap_total > 0 {
            (swap_used as f64 / swap_total as f64) * 100.0
        } else {
            0.0
        };

        // CPU history for sparkline
        cpu_history.push(cpu as u64);
        if cpu_history.len() > history_max {
            cpu_history.remove(0);
        }

        tick += 1;
        let spinner = SPIN[tick % SPIN.len()];

        terminal.draw(|f| {
            let area = f.area();

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(10), // banner
                    Constraint::Length(3),  // gauges row
                    Constraint::Min(8),    // middle section
                    Constraint::Length(5), // sparkline
                    Constraint::Length(3), // footer
                ])
                .split(area);

            // ══════════════════════════════════════════════════════
            //  BANNER — ASCII art logo + system info subtitle
            // ══════════════════════════════════════════════════════
            let mut banner_lines: Vec<Line> = LOGO
                .lines()
                .map(|l| {
                    Line::from(Span::styled(
                        l.to_string(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                })
                .collect();
            banner_lines.push(Line::from(""));
            banner_lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{} ", spinner),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    "ONLINE",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "v0.1.0",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} {}", &sys_name, &os_version),
                    Style::default().fg(Color::White),
                ),
                Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                Span::styled(host_name.clone(), Style::default().fg(Color::White)),
                Span::styled(
                    "  \u{2502}  \u{23f1} ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format_uptime(uptime),
                    Style::default().fg(Color::Yellow),
                ),
            ]));

            let banner = Paragraph::new(banner_lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(cyan)
                    .title(Span::styled(
                        " \u{25c8} DEVFLOW COMMAND CENTER \u{25c8} ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
            );
            f.render_widget(banner, main_layout[0]);

            // ══════════════════════════════════════════════════════
            //  GAUGES ROW — CPU / MEM / SWAP / PROCS
            // ══════════════════════════════════════════════════════
            let gauge_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ])
                .split(main_layout[1]);

            let cpu_gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(cyan)
                        .title(Span::styled(
                            " \u{26a1} CPU ",
                            Style::default().fg(Color::Cyan),
                        )),
                )
                .gauge_style(
                    Style::default()
                        .fg(gauge_color(cpu as f64))
                        .bg(Color::DarkGray),
                )
                .ratio((cpu as f64 / 100.0).min(1.0))
                .label(Span::styled(
                    format!("{:.1}%", cpu),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            f.render_widget(cpu_gauge, gauge_layout[0]);

            let mem_label = format!(
                "{} / {}",
                format_bytes(mem_used),
                format_bytes(mem_total)
            );
            let mem_gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(cyan)
                        .title(Span::styled(
                            " \u{25c8} MEM ",
                            Style::default().fg(Color::Cyan),
                        )),
                )
                .gauge_style(
                    Style::default()
                        .fg(gauge_color(mem_pct))
                        .bg(Color::DarkGray),
                )
                .ratio((mem_pct / 100.0).min(1.0))
                .label(Span::styled(
                    mem_label,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            f.render_widget(mem_gauge, gauge_layout[1]);

            let swap_label = format!(
                "{} / {}",
                format_bytes(swap_used),
                format_bytes(swap_total)
            );
            let swap_gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(cyan)
                        .title(Span::styled(
                            " \u{25c8} SWAP ",
                            Style::default().fg(Color::Cyan),
                        )),
                )
                .gauge_style(
                    Style::default()
                        .fg(gauge_color(swap_pct))
                        .bg(Color::DarkGray),
                )
                .ratio((swap_pct / 100.0).min(1.0))
                .label(Span::styled(
                    swap_label,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            f.render_widget(swap_gauge, gauge_layout[2]);

            let procs_widget = Paragraph::new(Line::from(vec![Span::styled(
                format!("{}", proc_count),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(cyan)
                    .title(Span::styled(
                        " \u{25b6} PROCS ",
                        Style::default().fg(Color::Cyan),
                    )),
            );
            f.render_widget(procs_widget, gauge_layout[3]);

            // ══════════════════════════════════════════════════════
            //  MIDDLE — CPU Cores (left) + Top Processes (right)
            // ══════════════════════════════════════════════════════
            let mid_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(main_layout[2]);

            // ─── Left: CPU Core Bars ───
            let bar_area_w = mid_layout[0].width.saturating_sub(2) as usize;
            let bar_w = bar_area_w.saturating_sub(16).max(5);

            let core_lines: Vec<Line> = core_data
                .iter()
                .enumerate()
                .map(|(i, &usage)| {
                    let filled = ((usage / 100.0) * bar_w as f32) as usize;
                    let empty = bar_w.saturating_sub(filled);
                    let color = gauge_color(usage as f64);
                    Line::from(vec![
                        Span::styled(
                            format!(" C{:02} ", i),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled("\u{2588}".repeat(filled), Style::default().fg(color)),
                        Span::styled(
                            "\u{2591}".repeat(empty),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(format!(" {:>5.1}%", usage), Style::default().fg(color)),
                    ])
                })
                .collect();

            let cores_widget = Paragraph::new(core_lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(cyan)
                    .title(Span::styled(
                        " \u{25b6} CPU CORES ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
            );
            f.render_widget(cores_widget, mid_layout[0]);

            // ─── Right: Top Processes Table ───
            let header = Row::new(vec![
                Cell::from("  PID").style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("PROCESS").style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("CPU%").style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("MEM").style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
            .style(Style::default().bg(Color::DarkGray));

            let proc_rows: Vec<Row> = proc_data
                .iter()
                .map(|(pid, name, cpu_u, mem)| {
                    let color = if *cpu_u > 50.0 {
                        Color::Red
                    } else if *cpu_u > 20.0 {
                        Color::Yellow
                    } else {
                        Color::White
                    };
                    Row::new(vec![
                        Cell::from(format!("  {}", pid))
                            .style(Style::default().fg(Color::DarkGray)),
                        Cell::from(name.clone()).style(Style::default().fg(Color::White)),
                        Cell::from(format!("{:.1}", cpu_u)).style(Style::default().fg(color)),
                        Cell::from(format_bytes(*mem))
                            .style(Style::default().fg(Color::DarkGray)),
                    ])
                })
                .collect();

            let widths = [
                Constraint::Length(8),
                Constraint::Min(16),
                Constraint::Length(8),
                Constraint::Length(12),
            ];

            let proc_table = Table::new(proc_rows, widths).header(header).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(cyan)
                    .title(Span::styled(
                        " \u{25b6} TOP PROCESSES ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
            );
            f.render_widget(proc_table, mid_layout[1]);

            // ══════════════════════════════════════════════════════
            //  SPARKLINE — CPU usage history
            // ══════════════════════════════════════════════════════
            let spark = Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(cyan)
                        .title(Span::styled(
                            " \u{25b6} CPU HISTORY ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )),
                )
                .data(&cpu_history)
                .max(100)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(spark, main_layout[3]);

            // ══════════════════════════════════════════════════════
            //  FOOTER — keybindings + monitoring status
            // ══════════════════════════════════════════════════════
            let footer = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", spinner),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    "MONITORING",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "[q]",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Quit  ", Style::default().fg(Color::White)),
                Span::styled("\u{2502}  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} cores", core_count),
                    Style::default().fg(Color::White),
                ),
                Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} procs", proc_count),
                    Style::default().fg(Color::White),
                ),
                Span::styled("  \u{2502}  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "\u{27f3} 400ms",
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(cyan),
            );
            f.render_widget(footer, main_layout[4]);
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
