use crate::app::{load_app_entries, AppEntry};
use crate::data::FenrirSocket;
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
use std::{io, time::Instant};
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

pub fn run_ui(show_icons: bool, t0: Instant) -> io::Result<()> {
    let mut t1: Option<Instant> = None;
    let mut t2: Option<Instant> = None;
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
        // let mut counter = 0;
        loop {
            /* if counter % 2 == 0 */ { read_ratatoskr( sender.clone()); }
            // counter += 1;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    let mut apps_entries: Vec<AppEntry> = vec![];
    let mut sock = FenrirSocket::new("/tmp/ratatoskr.sock");

    loop {
        if let Ok(data) = receiver.try_recv() {
            sysinfo = data.clone();
        }

        sock.poll_messages();

        if let Ok(data) = sock.rx.try_recv() {
            // println!("Received: {:?}", data);
            /* if data.resource == "battery" {
                if let Some(bat) = &data.data {
                    // {"capacity": Number(177228.0), "color": String("#55FF00"), "eta": Number(380.0978088378906), "icon": String("\u{f0079}"), "percentage": Number(100), "state": String("Discharging"), "warn": Number(0.0), "watt": Number(7.76800012588501)}
                    // let old_eta = app.battery_eta;
                    // let old_state = app.battery_recharging;
                    app.battery_eta = bat["eta"].as_f64();
                    app.battery_recharging = match bat["state"].as_str().unwrap() {
                        "Discharging" => Some(false),
                        "Charging" => Some(true),
                        _ => None
                    };
                    app.request_redraw();
                    // eprintln!("{:?}", bat);
                    // eprintln!("battery {:?} {:?}", app.battery_recharging, app.battery_eta);
                }
            }

            if data.resource == "ratatoskr" {
                let new_ratatoskr_status = data.warning < 0.5;
                if app.ratatoskr_connected != new_ratatoskr_status {
                    app.ratatoskr_connected = new_ratatoskr_status;
                    app.request_redraw();
                }
            } else if data.warning < 0.3 {
                if app.remove_icon(&data.resource) {
                    app.request_redraw();
                }
            }
            else {
                let mut icon = "";
                if data.resource == "loadavg" { icon = "󰬢"; }
                else if data.resource == "ram" { icon = "󰘚"; }
                else if data.resource == "temperature" { icon = &data.icon; }
                else if data.resource == "network" { icon = &data.icon; }
                else if data.resource == "disk" { icon = "󰋊"; }
                // weather
                // volume
                // disk
                // display

                if icon != "" {
                    app.remove_icon(&data.resource);
                    app.add_icon(&data.resource, icon, get_color_gradient(data.warning), data.warning);
                    app.request_redraw();
                }
            } */
        }

        let filtered: Vec<_> = apps_entries.iter()
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

            if t1 == None {
                t1 = Some(Instant::now());
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

        if apps_entries.len() == 0 {
            apps_entries = load_app_entries().unwrap_or_default();
            if t2 == None {
                t2 = Some(Instant::now());
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

    println!("󰹉 Window realized at {:?}", t1.unwrap() - t0);
    println!("󱡠 App list visible at {:?}", t2.unwrap() - t0);

    Ok(())
}