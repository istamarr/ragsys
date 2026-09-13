use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use anyhow::Result;
use chrono::{Local, Utc};
use regex::Regex;
use crate::domain::flow_code::*;
use crate::rag_agent::handler::flow_code::git::clone_repository;
//crates. rust


pub struct CodeAnalyzer {
    project_path: PathBuf,
    git_url: Option<String>,
}

impl CodeAnalyzer {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            git_url: None,
        }
    }

    pub fn with_git_url(mut self, git_url: String) -> Self {
        self.git_url = Some(git_url);
        self
    }

    pub async fn clone_repository(&self, target_dir: &Path) -> Result<()> {
        if let Some(git_url) = &self.git_url {
            clone_repository;
            clone_repository(git_url, target_dir).await?;
        }
        Ok(())
    }

    pub async fn analyze_project(&self) -> Result<ProjectAnalysis> {
        let project_name = self.project_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut total_files = 0;
        let mut total_lines = 0;
        let mut file_types: HashMap<String, usize> = HashMap::new();
        let mut issues_found = Vec::new();

        // Analyze files
        for entry in WalkDir::new(&self.project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Skip hidden files and common ignore patterns
            if self.should_skip_file(path) {
                continue;
            }

            total_files += 1;

            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                *file_types.entry(ext).or_insert(0) += 1;
            }

            // Analyze file content
            if let Ok(content) = fs::read_to_string(path) {
                let lines = content.lines().count();
                total_lines += lines;

                // Detect issues in this file
                let mut file_issues = self.analyze_file_content(path, &content);
                issues_found.append(&mut file_issues);
            }
        }

        let language = self.detect_primary_language(&file_types);
        let framework = self.detect_framework(&language);
        let dependencies = self.extract_dependencies();
        let structure_tree = self.generate_structure_tree()?;

        Ok(ProjectAnalysis {
            name: project_name,
            root_path: self.project_path.to_string_lossy().to_string(),
            language,
            framework,
            total_files,
            total_lines,
            file_types,
            issues_found,
            dependencies,
            structure_tree,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    fn should_skip_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Skip hidden files and directories
        if path_str.contains("/.") || path_str.contains("\\.") {
            return true;
        }

        // Skip common build/cache directories
        let skip_dirs = [
            "target", "node_modules", "__pycache__", ".git", ".svn",
            "build", "dist", "out", ".next", ".nuxt", "vendor",
            ".vscode", ".idea", "coverage", ".nyc_output"
        ];

        for skip_dir in &skip_dirs {
            if path_str.contains(&format!("/{}/", skip_dir)) ||
                path_str.contains(&format!("\\{}\\", skip_dir)) {
                return true;
            }
        }

        // Skip large files (> 1MB
        if let Ok(metadata) = path.metadata() {
         //Selection If its .bin.safetensor.dll.gguf
            if metadata.len() > 1_048_576 {
                return true;
            }else{
                 //call fn to detect ekstensi, check purpose
             }
        }

        false
    }

    fn analyze_file_content(&self, file_path: &Path, content: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let file_name = file_path
            .strip_prefix(&self.project_path)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let extension = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            self.check_long_lines(line, line_num, &file_name, &mut issues);
            self.check_todo_comments(line, line_num, &file_name, &mut issues);

            match extension.as_str() {
                "rs" => self.check_rust_issues(line, line_num, &file_name, &mut issues),
                "py" => self.check_python_issues(line, line_num, &file_name, &mut issues),
                "js" | "ts" | "jsx" | "tsx" => self.check_javascript_issues(line, line_num, &file_name, &mut issues),
                "java" => self.check_java_issues(line, line_num, &file_name, &mut issues),
                "c" | "cpp" | "cc" | "cxx" => self.check_c_cpp_issues(line, line_num, &file_name, &mut issues),
                "go" => self.check_go_issues(line, line_num, &file_name, &mut issues),
                "php" => self.check_php_issues(line, line_num, &file_name, &mut issues),
                "rb" => self.check_ruby_issues(line, line_num, &file_name, &mut issues),
                "cs" => self.check_csharp_issues(line, line_num, &file_name, &mut issues),
                _ => {}
            }
        }

        issues
    }

    fn check_long_lines(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        if line.len() > 120 {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: format!("Line too long ({} characters)", line.len()),
                suggestion: Some("Consider breaking this line into multiple lines".to_string()),
            });
        }
    }

    fn check_todo_comments(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        let todo_regex = Regex::new(r"(?i)(TODO|FIXME|HACK|XXX|BUG)").unwrap();
        if todo_regex.is_match(line) {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Medium".to_string(),
                message: "TODO/FIXME comment found".to_string(),
                suggestion: Some("Consider addressing this TODO item or creating a proper issue".to_string()),
            });
        }
    }

    fn check_rust_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        // Check for unwrap() usage
        if line.contains(".unwrap()") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Medium".to_string(),
                message: "Use of unwrap() can cause panic".to_string(),
                suggestion: Some("Use match, if let, or expect() with a descriptive message".to_string()),
            });
        }

        if line.trim().starts_with("unsafe") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "High".to_string(),
                message: "Unsafe code block detected".to_string(),
                suggestion: Some("Ensure unsafe code is necessary and properly documented".to_string()),
            });
        }

        if line.contains("println!") && !file_name.contains("main.rs") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "println! macro used outside main".to_string(),
                suggestion: Some("Consider using proper logging (log, tracing) instead".to_string()),
            });
        }
    }

    fn check_python_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        if line.trim() == "except:" {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Medium".to_string(),
                message: "Bare except clause".to_string(),
                suggestion: Some("Specify the exception type or use 'except Exception:'".to_string()),
            });
        }

        if line.contains("print(") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "Print statement found".to_string(),
                suggestion: Some("Consider using logging instead of print statements".to_string()),
            });
        }
    }

    fn check_javascript_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        // Check for console.log
        if line.contains("console.log") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "Console.log statement found".to_string(),
                suggestion: Some("Remove console.log or use proper logging".to_string()),
            });
        }

        if line.contains(" == ") && !line.contains(" === ") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Medium".to_string(),
                message: "Use of loose equality (==)".to_string(),
                suggestion: Some("Use strict equality (===) instead".to_string()),
            });
        }
    }

    fn check_java_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        if line.contains("System.out.println") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "System.out.println found".to_string(),
                suggestion: Some("Use proper logging framework instead".to_string()),
            });
        }
    }

    fn check_c_cpp_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        if line.contains("printf(") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "printf statement found".to_string(),
                suggestion: Some("Consider using safer alternatives or proper logging".to_string()),
            });
        }
    }

    fn check_go_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        if line.contains("fmt.Println") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "fmt.Println found".to_string(),
                suggestion: Some("Use proper logging instead of fmt.Println".to_string()),
            });
        }
    }

    fn check_php_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        if line.contains("var_dump") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "var_dump found".to_string(),
                suggestion: Some("Remove var_dump or use proper logging".to_string()),
            });
        }
    }

    fn check_ruby_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        if line.contains("puts ") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "puts statement found".to_string(),
                suggestion: Some("Use proper logging instead of puts".to_string()),
            });
        }
    }

    fn check_csharp_issues(&self, line: &str, line_num: usize, file_name: &str, issues: &mut Vec<CodeIssue>) {
        // Check for Console.WriteLine
        if line.contains("Console.WriteLine") {
            issues.push(CodeIssue {
                file: file_name.to_string(),
                line: line_num,
                severity: "Low".to_string(),
                message: "Console.WriteLine found".to_string(),
                suggestion: Some("Use proper logging framework instead".to_string()),
            });
        }
    }

    fn detect_primary_language(&self, file_types: &HashMap<String, usize>) -> String {
        let mut max_count = 0;
        let mut primary_language = "Unknown".to_string();

        let language_map = [
            ("java", "Java"),("py", "Python"),("rs", "Rust"),
            ("js", "JavaScript"),("ts", "TypeScript"),
            ("jsx", "JavaScript (React)"),("tsx", "TypeScript (React)"),
            ("c", "C"),("cpp", "C++"),("cc", "C++"),("cxx", "C++"),
            ("go", "Go"), ("php", "PHP"),("rb", "Ruby"),("cs", "C#"),
        ];

        for (ext, lang) in &language_map {
            if let Some(&count) = file_types.get(*ext) {
                if count > max_count {
                    max_count = count;
                    primary_language = lang.to_string();
                }
            }
        }

        primary_language
    }

    fn detect_framework(&self, language: &str) -> Option<String> {
        // Check for framework-specific files
        let cargo_toml = self.project_path.join("Cargo.toml");
        let package_json = self.project_path.join("package.json");
        let requirements_txt = self.project_path.join("requirements.txt");
        let pyproject_toml = self.project_path.join("pyproject.toml");

        match language {
            "Rust" => {
                if let Ok(content) = fs::read_to_string(&cargo_toml) {
                    if content.contains("axum") {
                        return Some("Axum".to_string());
                    } else if content.contains("actix-web") {
                        return Some("Actix-web".to_string());
                    } else if content.contains("warp") {
                        return Some("Warp".to_string());
                    } else if content.contains("rocket") {
                        return Some("Rocket".to_string());
                    }
                }
            }
            "JavaScript" | "TypeScript" | "JavaScript (React)" | "TypeScript (React)" => {
                if let Ok(content) = fs::read_to_string(&package_json) {
                    if content.contains("\"react\"") {
                        return Some("React".to_string());
                    } else if content.contains("\"vue\"") {
                        return Some("Vue.js".to_string());
                    } else if content.contains("\"express\"") {
                        return Some("Express.js".to_string());
                    } else if content.contains("\"next\"") {
                        return Some("Next.js".to_string());
                    }
                }
            }
            "Python" => {
                if let Ok(content) = fs::read_to_string(&requirements_txt) {
                    if content.contains("fastapi") {
                        return Some("FastAPI".to_string());
                    } else if content.contains("django") {
                        return Some("Django".to_string());
                    } else if content.contains("flask") {
                        return Some("Flask".to_string());
                    }
                }
                if let Ok(content) = fs::read_to_string(&pyproject_toml) {
                    if content.contains("fastapi") {
                        return Some("FastAPI".to_string());
                    } else if content.contains("django") {
                        return Some("Django".to_string());
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn extract_dependencies(&self) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Rust dependencies
        let cargo_toml = self.project_path.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&cargo_toml) {
            for line in content.lines() {
                if line.contains(" = ") && !line.starts_with('[') && !line.starts_with('#') {
                    if let Some(dep_name) = line.split('=').next() {
                        let clean_name = dep_name.trim().trim_matches('"');
                        if !clean_name.is_empty() {
                            dependencies.push(clean_name.to_string());
                        }
                    }
                }
            }
        }

        // Node.js dependencies
        let package_json = self.project_path.join("package.json");
        if let Ok(content) = fs::read_to_string(&package_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                    for key in deps.keys() {
                        dependencies.push(key.clone());
                    }
                }
                if let Some(dev_deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
                    for key in dev_deps.keys() {
                        dependencies.push(format!("{} (dev)", key));
                    }
                }
            }
        }

        // Python dependencies
        let requirements_txt = self.project_path.join("requirements.txt");
        if let Ok(content) = fs::read_to_string(&requirements_txt) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    if let Some(dep_name) = line.split(&['=', '>', '<', '!', '~'][..]).next() {
                        dependencies.push(dep_name.trim().to_string());
                    }
                }
            }
        }

        dependencies
    }

    fn generate_structure_tree(&self) -> Result<String> {
        let mut tree = String::new();
        tree.push_str(&format!("{}/\n",
                               self.project_path.file_name().unwrap_or_default().to_string_lossy()));

        self.build_tree_recursive(&self.project_path, "", &mut tree, 0, 3)?;
        Ok(tree)
    }

    fn build_tree_recursive(&self, dir: &Path, prefix: &str, tree: &mut String, depth: usize, max_depth: usize) -> Result<()> {
        if depth >= max_depth {
            return Ok(());
        }

        let mut entries: Vec<_> = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| !self.should_skip_file(&e.path()))
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for (i, entry) in entries.iter().enumerate() {
            let is_last = i == entries.len() - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let change_entry = entry.clone();
            // let name =  change_entry.file_name().to_string_lossy();

            tree.push_str(&format!("{}{}{}\n", prefix, connector, change_entry.file_name().to_string_lossy()));

            if change_entry.file_type().map_or(false, |ft| ft.is_dir()) {
                let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                self.build_tree_recursive(&change_entry.path(), &new_prefix, tree, depth + 1, max_depth)?;
            }
        }

        Ok(())
    }
}
