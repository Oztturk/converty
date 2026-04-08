use crate::cli::Config;
use crate::formats::Category;
use std::path::Path;
use std::process::Command;

pub fn run_image_conversion(input: &Path, output: &Path) -> Result<(), String> {
    let img = image::open(input).map_err(|e| format!("Failed to open image: {e}"))?;
    img.save(output).map_err(|e| format!("Failed to save image: {e}"))?;
    Ok(())
}

pub fn run_ffmpeg(input: &Path, output: &Path, new_cat: &Category, config: &Config) -> Result<(), String> {
    if matches!(new_cat, Category::Gif) {
        return run_ffmpeg_gif(input, output, config);
    }

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

fn run_ffmpeg_gif(input: &Path, output: &Path, config: &Config) -> Result<(), String> {
    let c = config.compress;

    let fps = ((5.0 + 10.0 * c).round() as u32).max(1);

    let colors = ((8.0 + 248.0 * c).round() as u32).clamp(4, 256);

    let scale = c.max(0.25);
    let scale_filter = if scale < 1.0 {
        format!("scale=trunc(iw*{scale}/2)*2:trunc(ih*{scale}/2)*2,")
    } else {
        String::new()
    };

    let filter = format!(
        "[0:v] fps={fps},{scale_filter}split [a][b]; \
         [a] palettegen=max_colors={colors} [p]; \
         [b][p] paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle"
    );

    let status = Command::new("ffmpeg")
        .args(["-v", "warning", "-y", "-i"])
        .arg(input)
        .args(["-filter_complex", &filter])
        .arg(output)
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("ffmpeg exited with code: {:?}", s.code())),
        Err(e) => Err(format!("Failed to run ffmpeg: {e}")),
    }
}
