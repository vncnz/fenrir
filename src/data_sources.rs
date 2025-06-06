use bytesize::ByteSize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::mpsc::{Sender, Receiver, channel};

use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

macro_rules! extract_json {
    ($obj:expr => {
        $( $path:literal => $var:ident : $method:ident ),+ $(,)?
    }) => {{
        $(
            let $var = {
                let mut current = Some($obj);
                for key in $path.split('.') {
                    current = current.and_then(|v| v.get(key));
                }
                current.and_then(|v| v.$method())
            };
        )+

        if let ($(Some($var)),+) = ($($var),+) {
            Some(($( $var ),+))
        } else {
            None
        }
    }};
}



pub fn read_ratatoskr (sender: Sender<Paragraph>) {
    if let Ok(contents) = fs::read_to_string("/tmp/ratatoskr.json") {
        let res: Result<Value, serde_json::Error> = serde_json::from_str(&contents);
        if let Ok(data) = res {
                if let Some((tm, memory_percent, ts, swap_percent, mem_color, swap_color, wea_temp, wea_symb, wea_icon)) = extract_json!(&data => {
                    "ram.total_memory" => tm: as_u64,
                    "ram.mem_percent" => memory_percent: as_u64,
                    "ram.total_swap" => ts: as_u64,
                    "ram.swap_percent" => swap_percent: as_u64,
                    "ram.mem_color" => mem_color: as_str,
                    "ram.swap_color" => swap_color: as_str,
                    "weather.temp" => wea_temp: as_i64,
                    "weather.temp_unit" => wea_symb: as_str,
                    "weather.icon" => wea_icon: as_str
                }) {
                // let line = format!("{tm} {um} {ts} {us}");

                let tmh = ByteSize::b(tm).display().iec().to_string();
                let tsh = ByteSize::b(ts).display().iec().to_string();

                let paragraph = Paragraph::new(Line::from(vec![
                    Span::styled(format!("[MEM] {:.0}%", memory_percent), Style::default().fg(hex_to_color(mem_color).unwrap())),
                    Span::styled(format!("(󰍛): {:.0}% of {}", swap_percent, tsh), Style::default().fg(hex_to_color(swap_color).unwrap())),
                    Span::raw(format!("{}{}{}", wea_icon, wea_temp, wea_symb))
                ]));

                sender.send(paragraph).expect("Send error");
            } else {
                println!("File opened, wrong format");
            }
        } else {
            // File exists but contains shit
            println!("File exists but contains shit");
        }
    } else {
        // No file
        println!("No file");
    }
}