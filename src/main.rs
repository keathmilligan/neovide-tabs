#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg(target_os = "windows")]

mod config;
mod process;
mod window;

use anyhow::Result;
use config::Config;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check for debug commands
    if args.len() >= 2 {
        match args[1].as_str() {
            "list-windows" => {
                let search = if args.len() >= 3 { &args[2] } else { "neovide" };
                process::debug_list_windows(search);
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                println!("neovide-tabs - A tabbed wrapper for Neovide");
                println!();
                println!("Usage:");
                println!("  neovide-tabs                    Run the application");
                println!(
                    "  neovide-tabs list-windows [name]  List windows matching name (default: neovide)"
                );
                println!("  neovide-tabs help               Show this help");
                return Ok(());
            }
            _ => {}
        }
    }

    // Load configuration
    let config = Config::load();

    // Check if Neovide is available before creating the window
    if process::NeovideProcess::check_neovide_available().is_err() {
        window::show_neovide_not_found_error();
        std::process::exit(1);
    }

    // Register window class with configured background color
    window::register_window_class(config.background_color)?;

    // Create main window
    let _hwnd = window::create_window()?;

    // Run message loop
    window::run_message_loop()?;

    Ok(())
}
