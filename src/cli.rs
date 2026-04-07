use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Clone)]
#[command(
    name = "converty",
    about = "Automatically converts media files when their extension is changed",
    long_about = "Watches for file renames and converts the file content to match the new \
                  extension. Uses ffmpeg and ImageMagick."
)]
pub struct Config {

    #[arg(long, default_value_t = false)]
    pub save_origin: bool,


    #[arg(long)]
    pub save_path: Option<PathBuf>,

    #[arg(long)]
    pub temp_path: Option<PathBuf>,
}

impl Config {
    pub fn is_save_origin(&self) -> bool {
        self.save_origin || self.save_path.is_some()
    }

    pub fn resolve_temp_dir<'a>(&'a self, file_dir: &'a std::path::Path) -> &'a std::path::Path {
        self.temp_path.as_deref().unwrap_or(file_dir)
    }

    pub fn resolve_save_dir<'a>(&'a self, file_dir: &'a std::path::Path) -> &'a std::path::Path {
        self.save_path.as_deref().unwrap_or(file_dir)
    }
}
