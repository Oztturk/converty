use crate::cli::Config;
use crate::converter::convert;
use crate::formats::get_category;
use notify::{EventKind, event::ModifyKind, event::RenameMode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub type Ignoring = Arc<Mutex<std::collections::HashSet<PathBuf>>>;
pub type Pending = Arc<Mutex<HashMap<u64, (PathBuf, String)>>>;

pub fn handle_event(
    event: notify::Event,
    ignoring: &Ignoring,
    pending: &Pending,
    config: Arc<Config>,
) {
    if let EventKind::Modify(ModifyKind::Name(mode)) = &event.kind {
        match mode {
            RenameMode::From => {
                if let Some(old_path) = event.paths.first() {
                    if is_tmp(old_path) {
                        return;
                    }
                    if let Some(old_ext) = get_ext(old_path) {
                        if get_category(&old_ext).is_some() {
                            let cookie = event_cookie(&event, old_path);
                            pending.lock().unwrap().insert(cookie, (old_path.clone(), old_ext));
                        }
                    }
                }
            }
            RenameMode::To => {
                let new_path = match event.paths.first() {
                    Some(p) => p.clone(),
                    None => return,
                };

                if ignoring.lock().unwrap().contains(&new_path) || is_tmp(&new_path) {
                    return;
                }

                let new_ext = match get_ext(&new_path) {
                    Some(e) => e,
                    None => return,
                };

                if get_category(&new_ext).is_none() {
                    return;
                }

                let cookie = event_cookie(&event, &new_path);
                let (old_path, old_ext) = match pending.lock().unwrap().remove(&cookie) {
                    Some(v) => v,
                    None => return,
                };

                if old_ext == new_ext {
                    return;
                }

                spawn_conversion(new_path, old_path, old_ext, new_ext, ignoring, config);
            }
            RenameMode::Both => {
                if event.paths.len() < 2 {
                    return;
                }
                let old_path = &event.paths[0];
                let new_path = event.paths[1].clone();

                if is_tmp(old_path) || is_tmp(&new_path) {
                    return;
                }
                if ignoring.lock().unwrap().contains(&new_path) {
                    return;
                }

                let old_ext = match get_ext(old_path) {
                    Some(e) => e,
                    None => return,
                };
                let new_ext = match get_ext(&new_path) {
                    Some(e) => e,
                    None => return,
                };

                if old_ext == new_ext {
                    return;
                }
                if get_category(&old_ext).is_none() || get_category(&new_ext).is_none() {
                    return;
                }

                spawn_conversion(new_path, old_path.clone(), old_ext, new_ext, ignoring, config);
            }
            _ => {}
        }
    }
}

fn spawn_conversion(
    new_path: PathBuf,
    old_path: PathBuf,
    old_ext: String,
    new_ext: String,
    ignoring: &Ignoring,
    config: Arc<Config>,
) {
    log::info!(
        "{} → {} ({} → {})",
        old_path.display(),
        new_path.display(),
        old_ext,
        new_ext
    );
    println!("Converting: {} → {}", old_path.display(), new_path.display());

    let ignoring_clone = Arc::clone(ignoring);
    ignoring_clone.lock().unwrap().insert(new_path.clone());

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        if let Err(e) = convert(&new_path, &old_ext, &new_ext, &config) {
            eprintln!("Error: {e}");
        }
        ignoring_clone.lock().unwrap().remove(&new_path);
    });
}

pub fn is_tmp(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with(".converty_tmp_"))
        .unwrap_or(false)
}

pub fn get_ext(path: &Path) -> Option<String> {
    path.extension()?.to_str().map(|s| s.to_lowercase())
}

fn event_cookie(event: &notify::Event, path: &Path) -> u64 {
    event
        .attrs
        .info()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| hash_path(path))
}

fn hash_path(path: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    h.finish()
}
