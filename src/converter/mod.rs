mod tools;

use crate::cli::Config;
use crate::formats::{get_category, Category};
use std::path::Path;


pub fn convert(new_path: &Path, old_ext: &str, new_ext: &str, config: &Config) -> Result<(), String> {
    let old_cat = get_category(old_ext)
        .ok_or_else(|| format!("Unsupported source format: {old_ext}"))?;
    let new_cat = get_category(new_ext)
        .ok_or_else(|| format!("Unsupported target format: {new_ext}"))?;

    let file_dir = new_path.parent().unwrap_or(Path::new("."));
    let stem = new_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Could not read file name")?;

    let temp_dir = config.resolve_temp_dir(file_dir);
    ensure_dir(temp_dir)?;

    let tmp_filename = if temp_dir == file_dir {
        format!(".converty_tmp_{stem}.{old_ext}")
    } else {
        format!("{stem}.{old_ext}")
    };
    let tmp_path = temp_dir.join(&tmp_filename);

    std::fs::rename(new_path, &tmp_path)
        .map_err(|e| format!("Failed to move file to temp location: {e}"))?;

    let result = run_conversion(&tmp_path, new_path, &old_cat, &new_cat, config);

    match result {
        Ok(()) => {
            if config.is_save_origin() {
                let save_dir = config.resolve_save_dir(file_dir);
                ensure_dir(save_dir)?;
                let origin_path = save_dir.join(format!("{stem}.{old_ext}"));
                std::fs::rename(&tmp_path, &origin_path)
                    .map_err(|e| format!("Failed to save original file: {e}"))?;
                log::info!(
                    "Done: {} → {} (original saved: {})",
                    old_ext,
                    new_path.display(),
                    origin_path.display()
                );
            } else {
                let _ = std::fs::remove_file(&tmp_path);
                log::info!("Done: {}", new_path.display());
            }
            Ok(())
        }
        Err(e) => {
            let restore = file_dir.join(format!("{stem}.{old_ext}"));
            log::error!("Conversion failed, restoring original: {e}");
            let _ = std::fs::rename(&tmp_path, &restore);
            Err(e)
        }
    }
}

fn run_conversion(input: &Path, output: &Path, old_cat: &Category, new_cat: &Category, config: &Config) -> Result<(), String> {
    let use_ffmpeg = matches!(
        (old_cat, new_cat),
        (Category::Video, _) | (_, Category::Video) | (Category::Gif, _)
    );

    if use_ffmpeg {
        return tools::run_ffmpeg(input, output, new_cat, config);
    }

    match tools::run_image_conversion(input, output) {
        Ok(()) => Ok(()),
        Err(img_err) => {
            log::warn!("image conversion failed ({img_err}), retrying with ffmpeg...");
            let _ = std::fs::remove_file(output);
            tools::run_ffmpeg(input, output, new_cat, config)
        }
    }
}

fn ensure_dir(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {e}", dir.display()))?;
    }
    Ok(())
}
