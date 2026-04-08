mod cli;
mod converter;
mod formats;
mod watcher;

use clap::Parser;
use std::process::Command;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = cli::Config::parse();

    check_dependency("ffmpeg");

    if let Some(ref p) = config.temp_path {
        println!("Temp directory: {}", p.display());
    }
    if config.is_save_origin() {
        match &config.save_path {
            Some(p) => println!("Originals will be saved to: {}", p.display()),
            None => println!("Originals will be kept in the same directory"),
        }
    }

    let home = dirs::home_dir().expect("Could not find home directory");
    if let Err(e) = watcher::watch(&home, config) {
        eprintln!("Failed to start watcher: {e}");
        std::process::exit(1);
    }
}

fn check_dependency(name: &str) {
    let ok = Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !ok {
        eprintln!("Error: '{name}' not found. Please install it:");
        match name {
            "ffmpeg" => eprintln!("  sudo pacman -S ffmpeg"),
            _ => {}
        }
        std::process::exit(1);
    }
}
