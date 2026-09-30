//! Fenrir: A TUI App Launcher in Rust with icon support (Kitty only)

// kitty -e ~/.config/niri/fenrir

mod app;
// mod sysinfo;
mod data;
mod ui;
mod utils;
mod data_sources;

// use crate::app::AppEntry;
use crate::ui::run_ui;
use std::env;
use std::error::Error;

use std::time::Instant;
use clap::{crate_name, crate_version, Parser};

#[derive(Parser, Debug)]
#[command(disable_version_flag = true)]
struct Args {

    /// Print version
    #[arg(short, long)]
    version: bool,
    
    /// Force not to use icons
    #[arg(
        short,
        long,
        num_args=0..=1,
        default_missing_value = "true",
        // action = clap::ArgAction::SetTrue
    )]
    icons: Option<bool>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let t0 = Instant::now();
    /* let args: Vec<String> = env::args().collect();

    let print_version = args.contains(&"--version".to_string());
    if print_version {
        println!("{} {}", crate_name!(), crate_version!());
        std::process::exit(0);
    }

    let force_icons = args.contains(&"--force-icons".to_string());
    let no_icons = args.contains(&"--no-icons".to_string());

    let show_icons = if force_icons {
        true
    } else if no_icons {
        false
    } else {
        std::env::var("KITTY_WINDOW_ID").is_ok()
    };
    */
    let args = Args::parse();

    if args.version {
        println!("{} {}", crate_name!(), crate_version!());
        std::process::exit(0);
    }

    let show_icons = match args.icons {
        Some(enabled) => enabled,
        None => std::env::var("KITTY_WINDOW_ID").is_ok()
    };

    // let apps_empty: Vec<AppEntry> = vec![];
    // let apps = app::load_app_entries()?;
    run_ui(show_icons, t0)?;
    Ok(())
}