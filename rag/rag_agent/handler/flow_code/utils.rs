use std::path::Path;
use anyhow::Result;

pub fn should_ignore_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    if path_str.contains("/.") || path_str.contains("\\.") {
        return true;
    }

    let ignore_dirs = [
        "target", "node_modules", "__pycache__", ".git", ".svn",
        "build", "dist", "out", ".next", ".nuxt", "vendor",
        ".vscode", ".idea", "coverage", ".nyc_output", "logs"
    ];

    for ignore_dir in &ignore_dirs {
        if path_str.contains(&format!("/{}/", ignore_dir)) ||
            path_str.contains(&format!("\\{}\\", ignore_dir)) {
            return true;
        }
    }

    false
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

pub fn is_source_file(path: &Path) -> bool {
    if let Some(ext) = get_file_extension(path) {
        matches!(ext.as_str(),
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" |
            "cc" | "cxx" | "h" | "hpp" | "go" | "php" | "rb" | "cs" | "swift" |
            "kt" | "scala" | "clj" | "hs" | "ml" | "elm" | "dart" | "lua" | "r"
        )
    } else {
        false
    }
}

pub fn is_config_file(path: &Path) -> bool {
    let filename = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    matches!(filename,
        "Cargo.toml" | "package.json" | "requirements.txt" | "pyproject.toml" |
        "pom.xml" | "build.gradle" | "go.mod" | "composer.json" | "Gemfile" |
        ".env" | "config.yml" | "config.yaml" | "config.json" | "tsconfig.json"
    ) || get_file_extension(path).map_or(false, |ext| {
        matches!(ext.as_str(), "toml" | "yaml" | "yml" | "json" | "ini" | "conf")
    })
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub fn calculate_percentage(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}
