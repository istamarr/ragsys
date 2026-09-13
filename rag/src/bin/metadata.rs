// changelog

fn main() {
    println!("Metadata generated");
}
// Changelog.md
// Whats new
// Describe

// [Version] date time
// Update link,pathdir from commit,metadata to commit,metadata

// Features
// Features 1 changed something
// Features 2 changed something

// ________

// # Changelog

// ## [1.2.0] - 2025-03-15 14:30:00

// ### Changed
// - Update link, pathdir from commit, metadata to commit, metadata

// ### Added
// - **Features 1:** Added real-time sync with cloud storage
// - **Features 2:** Introduced dark mode toggle in user settings


// ________


// # Changelog

// ## [1.2.0] - 2025-03-15 14:30:00

// ### Diubah
// - Perbarui tautan, pathdir dari commit,metadata ke commit,metadata

// ### Ditambahkan
// - **Fitur 1:** Menambahkan sinkronisasi real-time dengan penyimpanan cloud
// - **Fitur 2:** Menambakan tombol mode gelap di pengaturan pengguna

// Rust code for it fro. Metadara and log os changing

// _______

// _______



// use std::process::Command;
// use std::fs;
// use chrono::{Local, DateTime};
// use serde_json::json;

// /// Get last N git commits (metadata)
// fn get_git_commits(n: usize) -> Vec<(String, String, String)> {
//     let output = Command::new("git")
//         .args(["log", format!("-{}", n).as_str(), "--pretty=format:%h|%an|%s"])
//         .output()
//         .expect("git command failed");

//     String::from_utf8(output.stdout)
//         .unwrap()
//         .lines()
//         .map(|line| {
//             let parts: Vec<&str> = line.split('|').collect();
//             (
//                 parts[0].to_string(), // hash
//                 parts[1].to_string(), // author
//                 parts[2].to_string(), // subject
//             )
//         })
//         .collect()
// }

// /// Read OS change logs (example: /var/log/syslog or custom file)
// fn read_os_changes(log_path: &str) -> Vec<String> {
//     if let Ok(content) = fs::read_to_string(log_path) {
//         content.lines().take(5).map(|l| l.to_string()).collect()
//     } else {
//         vec!["No OS change log found".to_string()]
//     }
// }

// /// Generate changelog in your format
// fn generate_changelog(commits: Vec<(String, String, String)>, os_logs: Vec<String>) -> String {
//     let now: DateTime<Local> = Local::now();
//     let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
//     let version = "1.2.0"; // could be auto-incremented

//     let mut changelog = format!("# Changelog\n\n## [{}] - {}\n\n", version, date_str);
//     changelog.push_str("### Changed\n");
//     changelog.push_str("- Update link, pathdir from commit, metadata to commit, metadata\n\n");

//     changelog.push_str("### Features\n");
//     for (i, (hash, author, subject)) in commits.iter().enumerate() {
//         changelog.push_str(&format!("- **Feature {}:** {} ({} by {})\n", i+1, subject, hash, author));
//     }

//     changelog.push_str("\n### OS Changes\n");
//     for log in os_logs.iter().take(3) {
//         changelog.push_str(&format!("- {}\n", log));
//     }

//     changelog
// }

// fn main() {
//     let commits = get_git_commits(2);
//     let os_logs = read_os_changes("/var/log/syslog"); // adjust path for your OS
//     let changelog = generate_changelog(commits, os_logs);
//     println!("{}", changelog);
// }


// ______

// [dependencies]
// chrono = "0.4"
// serde_json = "1.0"
