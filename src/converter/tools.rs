use crate::formats::Category;
use std::path::Path;
use std::process::Command;

pub fn run_magick(input: &Path, output: &Path) -> Result<(), String> {
    let status = Command::new("magick").arg(input).arg(output).status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("magick exited with code: {:?}", s.code())),
        Err(e) => Err(format!("Failed to run magick: {e}")),
    }
}

pub fn run_ffmpeg(input: &Path, output: &Path, new_cat: &Category) -> Result<(), String> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-v", "warning", "-y", "-i"]).arg(input);

    if matches!(new_cat, Category::Photo) {
        cmd.args(["-frames:v", "1", "-update", "1"]);
    }

    cmd.arg(output);

    match cmd.status() {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("ffmpeg exited with code: {:?}", s.code())),
        Err(e) => Err(format!("Failed to run ffmpeg: {e}")),
    }
}
