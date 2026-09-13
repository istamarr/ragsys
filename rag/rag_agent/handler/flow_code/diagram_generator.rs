use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;

/// Diagram types supported
#[derive(Debug, Clone)]
pub enum DiagramType {
    Flowchart,      // Business flow
    ClassDiagram,   // Class/struct relationships
    SequenceDiagram, // Function call sequences
    ModuleDiagram,  // Module dependencies
}

/// Diagram generation result
#[derive(Debug, Clone)]
pub struct DiagramResult {
    pub diagram_type: String,
    pub mermaid_code: String,
    pub description: String,
    pub file_path: Option<String>,
    pub title: String,
}

/// Diagram generator for code projects
pub struct DiagramGenerator {
    project_path: PathBuf,
    project_name: String,
}

impl DiagramGenerator {
    pub fn new(project_path: PathBuf, project_name: String) -> Self {
        Self { project_path, project_name }
    }

    /// Generate all diagrams for the project
    pub fn generate_all_diagrams(&self) -> Result<Vec<DiagramResult>> {
        let mut diagrams = Vec::new();

        // Generate flowchart
        diagrams.push(self.generate_business_flowchart()?);
        
        // Generate class diagram
        diagrams.push(self.generate_class_diagram()?);
        
        // Generate module diagram
        diagrams.push(self.generate_module_diagram()?);
        
        // Generate sequence diagram
        diagrams.push(self.generate_sequence_diagram()?);

        Ok(diagrams)
    }

    /// Generate business flowchart from project structure
    pub fn generate_business_flowchart(&self) -> Result<DiagramResult> {
        let mut mermaid = String::new();
        mermaid.push_str("flowchart TD\n");
        mermaid.push_str(&format!("    A[{}] --> B{{Analyze Request}}\n", self.project_name));
        
        // Detect main entry points
        let entry_points = self.detect_entry_points();
        
        if !entry_points.is_empty() {
            mermaid.push_str("    B --> C[Route Handler]\n");
            
            for (i, entry) in entry_points.iter().enumerate().take(5) {
                let node_id = format!("D{}", i);
                mermaid.push_str(&format!("    C --> {}[{}]\n", node_id, entry));
            }
            
            mermaid.push_str("    C --> E[Process Logic]\n");
            mermaid.push_str("    E --> F{Validation}\n");
            mermaid.push_str("    F -->|Valid| G[Business Logic]\n");
            mermaid.push_str("    F -->|Invalid| H[Error Handler]\n");
            mermaid.push_str("    G --> I[(Database)]\n");
            mermaid.push_str("    I --> J[Response]\n");
            mermaid.push_str("    H --> J\n");
            mermaid.push_str("    J --> K[End]\n");
        } else {
            mermaid.push_str("    B --> C[Main Process]\n");
            mermaid.push_str("    C --> D[Execute Logic]\n");
            mermaid.push_str("    D --> E[Output Result]\n");
        }

        Ok(DiagramResult {
            diagram_type: "Business Flowchart".to_string(),
            mermaid_code: mermaid,
            description: format!("Business flow diagram for {}", self.project_name),
            file_path: None,
            title: "Business Flowchart".to_string(),
        })
    }

    /// Generate class/struct diagram
    pub fn generate_class_diagram(&self) -> Result<DiagramResult> {
        let mut mermaid = String::new();
        mermaid.push_str("classDiagram\n");

        let classes = self.extract_classes_and_structs();
        
        for (class_name, members) in classes.iter().take(15) {
            mermaid.push_str(&format!("    class {} {{\n", class_name));
            for member in members.iter().take(5) {
                mermaid.push_str(&format!("        {}\n", member));
            }
            mermaid.push_str("    }\n");
        }

        // Add relationships based on detected patterns
        let relationships = self.detect_class_relationships(&classes);
        for rel in relationships.iter().take(10) {
            mermaid.push_str(&format!("    {}\n", rel));
        }

        Ok(DiagramResult {
            diagram_type: "Class Diagram".to_string(),
            mermaid_code: mermaid,
            description: format!("Class/struct relationships in {}", self.project_name),
            file_path: None,
            title: "Class Diagram".to_string(),
        })
    }

    /// Generate module dependency diagram
    pub fn generate_module_diagram(&self) -> Result<DiagramResult> {
        let mut mermaid = String::new();
        mermaid.push_str("flowchart LR\n");
        mermaid.push_str(&format!("    subgraph {}\n", self.project_name));

        let modules = self.extract_modules();
        
        for (i, module) in modules.iter().enumerate().take(10) {
            let node_id = format!("M{}", i);
            mermaid.push_str(&format!("        {}[{}]\n", node_id, module));
        }

        mermaid.push_str("    end\n");

        // Add dependencies
        let deps = self.detect_module_dependencies(&modules);
        for dep in deps.iter().take(15) {
            mermaid.push_str(&format!("    {}\n", dep));
        }

        Ok(DiagramResult {
            diagram_type: "Module Diagram".to_string(),
            mermaid_code: mermaid,
            description: format!("Module dependencies in {}", self.project_name),
            file_path: None,
            title: "Module Diagram".to_string(),
        })
    }

    /// Generate sequence diagram for main flows
    pub fn generate_sequence_diagram(&self) -> Result<DiagramResult> {
        let mut mermaid = String::new();
        mermaid.push_str("sequenceDiagram\n");
        mermaid.push_str("    participant User\n");
        mermaid.push_str("    participant API\n");
        mermaid.push_str("    participant Service\n");
        mermaid.push_str("    participant Database\n\n");

        // Detect main flow patterns
        let flow_patterns = self.detect_flow_patterns();
        
        if !flow_patterns.is_empty() {
            for pattern in flow_patterns.iter().take(3) {
                mermaid.push_str(&format!("    User->>API: {}\n", pattern));
                mermaid.push_str("    API->>Service: Process Request\n");
                mermaid.push_str("    Service->>Database: Query Data\n");
                mermaid.push_str("    Database-->>Service: Return Data\n");
                mermaid.push_str("    Service-->>API: Process Result\n");
                mermaid.push_str("    API-->>User: Response\n\n");
            }
        } else {
            mermaid.push_str("    User->>API: Request\n");
            mermaid.push_str("    API->>Service: Process\n");
            mermaid.push_str("    Service->>Database: CRUD\n");
            mermaid.push_str("    Database-->>Service: Result\n");
            mermaid.push_str("    Service-->>API: Response\n");
            mermaid.push_str("    API-->>User: Output\n");
        }

        Ok(DiagramResult {
            diagram_type: "Sequence Diagram".to_string(),
            mermaid_code: mermaid,
            description: format!("Main flow sequence in {}", self.project_name),
            file_path: None,
            title: "Sequence Diagram".to_string(),
        })
    }

    /// Detect entry points (main.rs, index.js, app.py, etc.)
    fn detect_entry_points(&self) -> Vec<String> {
        let mut entries = Vec::new();
        let entry_patterns = ["main", "index", "app", "server", "handler", "route", "api"];
        
        for entry in WalkDir::new(&self.project_path)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            for pattern in &entry_patterns {
                if file_name.contains(pattern) {
                    let name = entry.path()
                        .strip_prefix(&self.project_path)
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .to_string();
                    if !entries.contains(&name) {
                        entries.push(name);
                    }
                    break;
                }
            }
        }
        
        entries
    }

    /// Extract classes and structs from code
    fn extract_classes_and_structs(&self) -> HashMap<String, Vec<String>> {
        let mut classes: HashMap<String, Vec<String>> = HashMap::new();
        
        for entry in WalkDir::new(&self.project_path)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            
            if matches!(ext, "rs" | "py" | "js" | "ts" | "java" | "cs") {
                if let Ok(content) = fs::read_to_string(path) {
                    self.parse_classes_from_content(&content, ext, &mut classes);
                }
            }
        }
        
        classes
    }

    fn parse_classes_from_content(&self, content: &str, ext: &str, classes: &mut HashMap<String, Vec<String>>) {
        for line in content.lines() {
            let trimmed = line.trim();
            
            match ext {
                "rs" => {
                    // Rust struct/impl
                    if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                        if let Some(name) = trimmed.split_whitespace().nth(if trimmed.starts_with("pub") { 2 } else { 1 }) {
                            let clean_name = name.trim_end_matches('{').trim_end_matches('<');
                            classes.entry(clean_name.to_string()).or_insert_with(Vec::new);
                        }
                    }
                    if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") || trimmed.starts_with("async fn ") {
                        if let Some(name) = trimmed.split('(').next() {
                            let fn_name = name.split_whitespace().last().unwrap_or("");
                            if !fn_name.is_empty() {
                                for (_, members) in classes.iter_mut() {
                                    if members.len() < 5 {
                                        members.push(format!("+{}()", fn_name));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                "py" => {
                    // Python class
                    if trimmed.starts_with("class ") {
                        if let Some(name) = trimmed.strip_prefix("class ").and_then(|s| s.split(&['(', ':'][..]).next()) {
                            classes.entry(name.trim().to_string()).or_insert_with(Vec::new);
                        }
                    }
                    if trimmed.starts_with("def ") {
                        if let Some(name) = trimmed.strip_prefix("def ").and_then(|s| s.split('(').next()) {
                            for (_, members) in classes.iter_mut() {
                                if members.len() < 5 {
                                    members.push(format!("+{}()", name.trim()));
                                    break;
                                }
                            }
                        }
                    }
                }
                "js" | "ts" => {
                    // JavaScript/TypeScript class
                    if trimmed.starts_with("class ") || trimmed.starts_with("export class ") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if let Some(&name) = parts.iter().find(|&&p| p != "class" && p != "export" && !p.contains('{')) {
                            classes.entry(name.to_string()).or_insert_with(Vec::new);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn detect_class_relationships(&self, classes: &HashMap<String, Vec<String>>) -> Vec<String> {
        let mut relationships = Vec::new();
        let class_names: Vec<&String> = classes.keys().collect();
        
        // Create some relationships based on naming patterns
        for (i, name) in class_names.iter().enumerate() {
            if name.contains("Handler") || name.contains("Controller") {
                if let Some(service) = class_names.iter().find(|n| n.contains("Service")) {
                    relationships.push(format!("{} --> {}", name, service));
                }
            }
            if name.contains("Service") {
                if let Some(repo) = class_names.iter().find(|n| n.contains("Repository") || n.contains("Repo")) {
                    relationships.push(format!("{} --> {}", name, repo));
                }
            }
            // Add inheritance for naming patterns
            if i > 0 && name.ends_with("Impl") {
                if let Some(base) = class_names.iter().find(|n| name.starts_with(**n)) {
                    relationships.push(format!("{} --|> {}", name, base));
                }
            }
        }
        
        relationships
    }

    fn extract_modules(&self) -> Vec<String> {
        let mut modules = Vec::new();
        
        for entry in WalkDir::new(&self.project_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with('.') && 
               !matches!(name.as_str(), "target" | "node_modules" | "__pycache__" | "dist" | "build") {
                modules.push(name);
            }
        }
        
        modules
    }

    fn detect_module_dependencies(&self, modules: &[String]) -> Vec<String> {
        let mut deps = Vec::new();
        
        // Simple dependency detection based on common patterns
        for (i, module) in modules.iter().enumerate() {
            if i > 0 && modules.len() > 1 {
                let prev_idx = (i + modules.len() - 1) % modules.len();
                if modules[prev_idx] != *module {
                    deps.push(format!("M{} --> M{}", prev_idx, i));
                }
            }
        }
        
        deps
    }

    fn detect_flow_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();
        let common_flows = ["GET /api", "POST /api", "PUT /api", "DELETE /api", "Login", "Register", "CRUD"];
        
        // Check for route patterns in files
        for entry in WalkDir::new(&self.project_path)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                for flow in &common_flows {
                    if content.to_lowercase().contains(&flow.to_lowercase()) && !patterns.contains(&flow.to_string()) {
                        patterns.push(flow.to_string());
                    }
                }
            }
        }
        
        if patterns.is_empty() {
            patterns.push("Main Request".to_string());
        }
        
        patterns
    }

    /// Save diagram to file
    pub fn save_diagram(&self, diagram: &DiagramResult, output_dir: &Path) -> Result<String> {
        let file_name = format!("{}.md", diagram.diagram_type.to_lowercase().replace(" ", "_"));
        let file_path = output_dir.join(&file_name);
        
        let content = format!(
            "# {}\n\n{}\n\n```mermaid\n{}\n```\n",
            diagram.diagram_type,
            diagram.description,
            diagram.mermaid_code
        );
        
        fs::write(&file_path, content)?;
        
        Ok(file_path.to_string_lossy().to_string())
    }
}

/// Print diagram to console
pub fn print_diagram(diagram: &DiagramResult) {
    println!("\n[{}]", diagram.diagram_type);
    println!("Description: {}", diagram.description);
    println!("\nMermaid Code:");
    println!("```mermaid");
    println!("{}", diagram.mermaid_code);
    println!("```\n");
}
