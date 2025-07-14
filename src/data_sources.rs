use serde_json::Value;
use std::fs;
use std::sync::mpsc::{Sender};

use ratatui::{
    style::{Color, Style},
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

/* macro_rules! extract_json {
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
} */
macro_rules! extract_json {
    ($data:expr => { $($path:literal => $method:ident),+ $(,)? }) => {
        (
            $(
                {
                    fn get_nested<'a>(data: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
                        path.split('.').fold(Some(data), |acc, key| acc?.get(key))
                    }
                    get_nested($data, $path).and_then(|v| v.$method())
                }
            ),+
        )
    };
}



use std::time::{SystemTime, UNIX_EPOCH};
pub fn read_ratatoskr (sender: Sender<Paragraph>) {
    if let Ok(contents) = fs::read_to_string("/tmp/ratatoskr.json") {
        let res: Result<Value, serde_json::Error> = serde_json::from_str(&contents);
        if let Ok(data) = res {
            let mut spans = Vec::<Span>::new();

            if let Some(written_at) = extract_json!(&data => {
                "written_at" => as_u64
            }) {
                let saved_time = UNIX_EPOCH + std::time::Duration::from_secs(written_at);
                let now = SystemTime::now();
                let diff = now.duration_since(saved_time).expect("Big bang is in the future?!");
                let secs = diff.as_secs();

                if secs > 2 {
                    spans.push(Span::styled(format!("[OLD {secs}s]"), Style::default().fg(Color::LightRed)));
                }
            }

            if let (Some(avg_m1), Some(avg_m5), Some(avg_m15), Some(avg_color)) = extract_json!(&data => {
                "loadavg.m1" => as_f64,
                "loadavg.m5" => as_f64,
                "loadavg.m15" => as_f64,
                "loadavg.color" => as_str
            }) {
                spans.push(Span::styled(format!("[AVG {avg_m1} {avg_m5} {avg_m15}]"), Style::default().fg(hex_to_color(avg_color).unwrap())));
            }

            // let tmh = ByteSize::b(tm).display().iec().to_string();
            if let (Some(memory_percent), Some(mem_color)) = extract_json!(&data => {
                // "ram.total_memory" => tm: as_u64,
                "ram.mem_percent" => as_u64,
                "ram.mem_color" => as_str
            }) {
                spans.push(Span::styled(format!(" [MEM {memory_percent}%]"), Style::default().fg(hex_to_color(mem_color).unwrap())));
            }

            if let (Some(swap_percent), Some(swap_color)) = extract_json!(&data => {
                "ram.swap_percent" => as_u64,
                "ram.swap_color" => as_str
            }) {
                spans.push(Span::styled(format!(" [SWP {swap_percent}%]"), Style::default().fg(hex_to_color(swap_color).unwrap())));
            }

            if let (Some(used_percent), Some(color)) = extract_json!(&data => {
                "disk.used_percent" => as_u64,
                "disk.color" => as_str
            }) {
                spans.push(Span::styled(format!(" [DSK {used_percent}%]"), Style::default().fg(hex_to_color(color).unwrap())));
            }

            if let (Some(temp), Some(color)) = extract_json!(&data => {
                "temperature.value" => as_f64,
                "temperature.color" => as_str
            }) {
                spans.push(Span::styled(format!(" [TEMP {:.0}%]", temp), Style::default().fg(hex_to_color(color).unwrap())));
            }

            if let (Some(value), Some(color)) = extract_json!(&data => {
                "volume.value" => as_u64,
                "volume.color" => as_str
            }) {
                if value > 0 { spans.push(Span::styled(format!(" [VOL {}%]", value), Style::default().fg(hex_to_color(color).unwrap()))); }
                else { spans.push(Span::styled(" [MUTED]", Style::default().fg(hex_to_color(color).unwrap()))); }
            }

            if let (Some(conn_type), signal, color) = extract_json!(&data => {
                "network.conn_type" => as_str,
                "network.signal" => as_f64,
                "network.color" => as_str
            }) {
                if conn_type == "ethernet" { spans.push(Span::raw(" [ETH]")); }
                else { spans.push(Span::styled(format!(" [WLAN {}%]", signal.unwrap()), Style::default().fg(hex_to_color(color.unwrap()).unwrap()))); }
            }

            if let (Some(wea_temp), Some(wea_symb), Some(_wea_icon), Some(wea_text)) = extract_json!(&data => {
                "weather.temp" => as_i64,
                "weather.temp_unit" => as_str,
                "weather.icon" => as_str,
                "weather.text" => as_str
            }) {
                spans.push(Span::raw(format!(" [{} {}{}]", wea_text, wea_temp, wea_symb)));
            }

            let paragraph = Paragraph::new(Line::from(spans));
            sender.send(paragraph).expect("Send error");
        } else {
            // File exists but contains shit
            // println!("File exists but contains shit");
            let paragraph = Paragraph::new(Span::styled("File exists but contains shit", Style::default().fg(Color::LightRed)));
        sender.send(paragraph).expect("Send error");
        }
    } else {
        // No file
        // println!("No file");
        let paragraph = Paragraph::new(Span::styled("No sysinfo file, maybe you missed to start Ratatoskr?", Style::default().fg(Color::LightMagenta)));
        sender.send(paragraph).expect("Send error");
    }
}