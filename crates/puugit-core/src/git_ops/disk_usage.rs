use std::path::PathBuf;
use std::sync::mpsc;

pub fn calc_subscription_sizes(paths: Vec<PathBuf>) -> mpsc::Receiver<(PathBuf, u64)> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for path in paths {
            let size = dir_size(&path);
            if tx.send((path, size)).is_err() {
                break;
            }
        }
    });
    rx
}

fn dir_size(path: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
