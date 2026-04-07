mod handler;

use crate::cli::Config;
use handler::{Ignoring, Pending, handle_event};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::time::Duration;

pub fn watch(home_dir: &Path, config: Config) -> notify::Result<()> {
    let (tx, rx) = mpsc::channel();
    let ignoring: Ignoring = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
    let config = Arc::new(config);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(200)),
    )?;

    watch_dir_skip_errors(&mut watcher, home_dir);
    log::info!("Watching: {}", home_dir.display());
    println!("Converty started. Watching: {}", home_dir.display());
    println!("Press Ctrl+C to stop");

    for res in rx {
        match res {
            Ok(event) => handle_event(event, &ignoring, &pending, Arc::clone(&config)),
            Err(e) => log::error!("Watcher error: {e}"),
        }
    }

    Ok(())
}

fn watch_dir_skip_errors(watcher: &mut RecommendedWatcher, dir: &Path) {
    if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
        log::warn!("Cannot watch: {} - {e}", dir.display());
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().map(|t| t.is_symlink()).unwrap_or(false) {
            continue;
        }
        if path.is_dir() {
            watch_dir_skip_errors(watcher, &path);
        }
    }
}
