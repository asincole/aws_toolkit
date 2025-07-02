use std::path::{Path, PathBuf};

pub fn create_unique_filepath(dir: &Path, filename: &str) -> PathBuf {
    let mut path = dir.join(filename);

    if !path.exists() {
        return path;
    }

    let (stem, extension) = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => (
            s.to_string(),
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string(),
        ),
        None => (filename.to_string(), String::new()),
    };

    let mut counter = 1;
    loop {
        let new_filename = if extension.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}).{}", stem, counter, extension)
        };

        path = dir.join(&new_filename);
        if !path.exists() {
            break;
        }
        counter += 1;
    }

    path
}
