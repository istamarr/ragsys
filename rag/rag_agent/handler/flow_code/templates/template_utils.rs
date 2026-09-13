/*!
 * Template Utilities
 * System PIN Ai
 */
use std::fs;
use std::path::Path;
use anyhow::Result;

pub fn create_dirs(base_path: &Path, dirs: &[&str]) -> Result<()> {
    for dir in dirs {
        let dir_path = base_path.join(dir);
        fs::create_dir_all(&dir_path)?;
    }
    Ok(())
}

pub fn write_file(file_path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, content)?;
    Ok(())
}

pub fn replace_template_vars(content: &str, vars: &[(&str, &str)]) -> String {
    let mut result = content.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

pub fn format_project_name(name: &str) -> (String, String, String) {
    let snake_case = name.replace("-", "_").to_lowercase();
    let kebab_case = name.replace("_", "-").to_lowercase();
    let pascal_case = name
        .split(&['-', '_'][..])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>();

    (snake_case, kebab_case, pascal_case)
}
