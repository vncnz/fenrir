use crate::app::{load_app_entries, AppEntry};
use crate::data::{FenrirSocket, PartialMsg};
// use crate::data_sources::read_ratatoskr;
use crate::utils::{get_color_gradient, log_to_file};

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
use regex::Regex;
use std::collections::HashMap;

// use chrono::Local;


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

pub fn update_span (paragraphs: &mut HashMap<String, Span>, data: PartialMsg) {
    // Extract the right paragraph or create a new one
    // Update it with updated data
    // return it

    // let par = Span::styled(format!(" [WLAN {}%]", signal.unwrap()), Style::default().fg(hex_to_color(color.unwrap()).unwrap()));
    // paragraphs.insert(data.resource, par);
    let res = data.resource.as_str();
    let mut span: Option<Span> = None;
    let wcolor = get_color_gradient(data.warning);
    let color = Color::Rgb(wcolor.0, wcolor.1, wcolor.2);
    match res {
        "loadavg" => {
            if let Some(info) = &data.data {
                span = Some(Span::styled(format!("[AVG {} {} {}] ",info["m1"], info["m5"], info["m15"]), Style::default().fg(color)));
            }
        },
        "ram" => {
            if let Some(info) = &data.data {
                span = Some(Span::styled(format!("[MEM {}% / SWP {}%] ", info["mem_percent"], info["swap_percent"]), Style::default().fg(color)));
            }
        },
        "disk" => {
            if let Some(info) = &data.data {
                span = Some(Span::styled(format!("[DSK {}%] ", info["used_percent"]), Style::default().fg(color)));
            }
        },
        "network" => {
            if let Some(info) = &data.data {
                if info["conn_type"] == "ethernet" {
                    span = Some(Span::raw("[ETH] "));
                } else {
                    span = Some(Span::styled(format!("[WLAN {}%] [IP {}] [NET {}] ", info["signal"], info["ip"].as_str().unwrap(), info["ssid"].as_str().unwrap()), Style::default().fg(color)));
                }
            }
        },
        "temperature" => {
            if let Some(info) = &data.data {
                span = Some(Span::styled(format!("[TEMP {:.0}°C] ", info["value"].as_f64().unwrap_or(0.0)), Style::default().fg(color)));
            }
        },
        "volume" => {
            if let Some(info) = &data.data {
                span = Some(Span::styled(format!("[VOL {:.0}%] ", info["value"].as_f64().unwrap_or(0.0)), Style::default().fg(color)));
            }
        },
        "battery" => {
            if let Some(info) = &data.data {
                let bat_symb = match info["state"].as_str() {
                    Some("Charging") => { "󱐋" },
                    Some("Discharging") => { "󰯆" },
                    _ => { "" }
                };
                let eta = info["eta"].as_f64().unwrap_or_default().round() as i8;
                let h = eta / 60;
                let m = eta % 60;
                span = Some(Span::styled(format!("[BAT {:.0}%] [{} {}h{}m]", info["percentage"].as_f64().unwrap_or(0.0), bat_symb, h, m), Style::default().fg(color)));
            }
        },
        "ratatoskr" => {
            if data.warning == 1.0 { span = Some(Span::styled(format!("Ratatoskr disconnected"), Style::default().fg(color))); }
        },
        "display" => {},
        "weather" => {
            // {"icon": "", "text": "Fog", "temp": 8, "temp_real": 9, "temp_unit": "°C", "day": "0", "icon_name": "fog.svg", "sunrise": "07:15", "sunset": "16:48", "sunrise_mins": 435, "sunset_mins": 1008, "daylight": 34385.75, "locality": "Desenzano Del Garda", "humidity": 99}
            if let Some(info) = &data.data {
                span = Some(Span::styled(format!("[{} {}] ", info["icon"], info["text"]), Style::default().fg(color)));
            }
        },
        _ => {
            span = Some(Span::styled(format!("[{}] ", data.resource), Style::default().fg(color)));
        }
    }
    /*if let Some(info) = &data.data {
        app.battery_eta = info["eta"].as_f64();
    }*/
    
    if paragraphs.contains_key(&data.resource) {
        paragraphs.remove(&data.resource);
    }
    if let Some(s) = span {
        paragraphs.insert(data.resource, s);
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
    // let mut sysinfo = Paragraph::default();

    /* let (sender, receiver) = channel::<Paragraph>();
    std::thread::spawn(move || {
        // let mut counter = 0;
        loop {
            /* if counter % 2 == 0 */ { read_ratatoskr( sender.clone()); }
            // counter += 1;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }); */

    let mut apps_entries: Vec<AppEntry> = vec![];
    let mut sock = FenrirSocket::new("/tmp/ratatoskr.sock");
    let mut spans: HashMap<String, Span> = HashMap::new();

    // let mut draws: i64 = 0;
    // let mut loops: i64 = 0;
    // let mut recv: String = "".into();

    loop {
        // loops += 1;
        /* if let Ok(data) = receiver.try_recv() {
            sysinfo = data.clone();
        } */

        sock.poll_messages();

        if let Ok(data) = sock.rx.try_recv() {
            // log_to_file(format!("Received: {} {:?}", data.resource, data));
            // recv.push(data.resource.chars().nth(0).unwrap());
            update_span(&mut spans, data);
        }

        let filtered: Vec<_> = apps_entries.iter()
            .filter(|a| a.name.to_lowercase().contains(&filter.to_lowercase()))
            .collect();

        let tsize = terminal.size().unwrap();
        terminal.draw(|f| {
            // draws += 1;
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(f.area());


            if spans.len() == 0 {
                f.render_widget(Span::raw("No sys information"), chunks[0]);
            } else {
                // f.render_widget(Span::raw(format!("{} redraws    {} loops    {} spans    {} recv", draws, loops, spans.len(), recv)), chunks[0]);
                // f.render_widget(Span::raw(format!("{} spans", spans.len())), chunks[0]);
                if spans.contains_key("ratatoskr") {
                    f.render_widget(Span::raw("Ratatoskr disconnected"), chunks[0]);
                } else {
                    let keys = ["loadavg", "ram", "disk", "temperature", "volume", "weather", "display"];
                    // let v: Vec<Span> = spans.values().cloned().collect();
                    let v: Vec<Span> = keys
                        .iter()
                        .filter_map(|k| spans.get(*k).cloned())
                        .collect();
                    f.render_widget(Paragraph::new(Line::from(v)), chunks[0]);
                }
            }
            let mut second_row: Vec<Span> = vec![];
            if spans.contains_key("network") {
                // f.render_widget(Paragraph::new(spans.get("network").cloned().unwrap_or_default()), chunks[1]);
                second_row.push(spans.get("network").cloned().unwrap_or_default());
            }
            if spans.contains_key("battery") {
                second_row.push(spans.get("battery").cloned().unwrap_or_default());
            }
            if second_row.len() > 0 {
                f.render_widget(Paragraph::new(Line::from(second_row)), chunks[1]);
            } else {
                f.render_widget(Paragraph::new(""), chunks[1]);
            }

            let input = Paragraph::new(format!("Filter: {}", filter));
            f.render_widget(input, chunks[3]);

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
            f.render_stateful_widget(list, chunks[4], &mut state);

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
                        } else {
                            last_icon_path = None;
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