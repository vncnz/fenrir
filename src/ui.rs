use crate::app::AppEntry;
use crate::data_sources::read_ratatoskr;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    style::{Style, Color},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use std::process::{Command, Stdio};
use std::fs::OpenOptions;
use std::sync::mpsc::{channel};
use regex::Regex;


pub fn launch_detached(app: &AppEntry) {
    // let exec = &app.exec;
    let re = Regex::new(r"%[UufFdDnNickvm]").unwrap();
    let exec = re.replace_all(&app.exec, "").to_string();

    // Log file in caso di errori
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/fenrir-launcher.log")
        .unwrap_or_else(|_| std::fs::File::create("/dev/null").unwrap());

    let result = Command::new("setsid")
        .arg("sh")
        .arg("-c")
        .arg(&exec)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file.try_clone().unwrap()))
        .stderr(Stdio::from(log_file))
        .spawn();

    if let Err(e) = result {
        eprintln!("Failed to launch '{}': {}", exec, e);
    }
}

pub fn run_ui(apps: Vec<AppEntry>, show_icons: bool) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut filter = String::new();
    let mut selected = 0;

    let mut last_icon_path: Option<std::path::PathBuf> = None;
    let mut sysinfo = Paragraph::default();

    let (sender, receiver) = channel::<Paragraph>();
    std::thread::spawn(move || {
        let mut counter = 0;
        loop {
            /* if counter % 2 == 0 */ { read_ratatoskr( sender.clone()); }
            counter += 1;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    loop {
        if let Ok(data) = receiver.try_recv() {
            sysinfo = data.clone();
        }

        let filtered: Vec<_> = apps.iter()
            .filter(|a| a.name.to_lowercase().contains(&filter.to_lowercase()))
            .collect();

        let tsize = terminal.size().unwrap();
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(f.area());

            f.render_widget(&sysinfo, chunks[0]);

            let input = Paragraph::new(format!("Filter: {}", filter));
            f.render_widget(input, chunks[1]);

            let items: Vec<_> = filtered.iter()
                .map(|a| ListItem::new(
                    // format!("{} {} - {} - {}", if a.terminal { "" } else { "" }, a.name, a.exec, a.comment)
                    Line::from(vec![
                        Span::styled(if a.terminal { "" } else { "" }, Style::default().fg(Color::Gray)),
                        Span::styled(format!(" {}", a.name), Style::default()),
                        Span::styled(format!(" {}", a.exec), Style::default().fg(Color::Yellow)),
                        Span::styled(format!(" {}", a.comment), Style::default().fg(Color::Rgb(128,128,128))),
                    ])
                ))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Applications"))
                .highlight_style(Style::default().bg(Color::Blue));

            let mut state = ratatui::widgets::ListState::default();
            state.select(Some(selected));
            f.render_stateful_widget(list, chunks[2], &mut state);

            // Icon rendering (Kitty required)
            let mut config = viuer::Config::default();
            config.x = tsize.width.saturating_sub(15) as u16 - 1;
            config.y = tsize.height.saturating_sub(7) as i16 - 1;
            config.width = Some(14);
            config.height = Some(6);

            if show_icons {
                if let Some(app) = filtered.get(selected) {
                    if last_icon_path != app.icon_path {
                        let black = image::DynamicImage::new_rgb8(96, 96); // 6x6 terminal cells ≈ 96x96 px
                                let _ = viuer::print(&black, &config);
                        if let Some(icon_path) = &app.icon_path {
                            if let Ok(img) = image::open(icon_path) {
                                let _ = viuer::print(&img, &config); // viuer::Config::default()
                                last_icon_path = app.icon_path.clone();
                            }
                        }
                    }
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char(c) => { filter.push(c); selected = 0; },
                    KeyCode::Backspace => { filter.pop(); },
                    KeyCode::Up => { if selected > 0 { selected -= 1; } },
                    KeyCode::Down => { if selected + 1 < filtered.len() { selected += 1; } },
                    KeyCode::Enter => {
                        if let Some(app) = filtered.get(selected) {
                            /* let _ = Command::new("sh")
                                .arg("-c")
                                .arg(&app.exec)
                                .spawn(); */
                            launch_detached(app);
                            std::thread::sleep(std::time::Duration::from_millis(600));

                            break;
                        }
                    },
                    KeyCode::Esc => break,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}