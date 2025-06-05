//! Fenrir: A TUI App Launcher in Rust with icon support (Kitty only)

// kitty -e ~/.config/niri/fenrir

mod app;
// mod sysinfo;
mod ui;

use crate::app::AppEntry;
use crate::ui::run_ui;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let force_icons = args.contains(&"--force-icons".to_string());
    let no_icons = args.contains(&"--no-icons".to_string());

    let show_icons = if force_icons {
        true
    } else if no_icons {
        false
    } else {
        std::env::var("KITTY_WINDOW_ID").is_ok()
    };

    let apps = app::load_app_entries()?;
    run_ui(apps, show_icons)?;
    Ok(())
}