use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FixCoLlmConfig {
    pub enable_semantic_analysis: bool,
    pub enable_security_analysis: bool,
    pub enable_performance_analysis: bool,
    pub enable_automotive_analysis: bool,
    pub max_length: usize,
    pub temperature: f32,
}

impl Default for FixCoLlmConfig {
    fn default() -> Self {
        Self {
            enable_semantic_analysis: true,
            enable_security_analysis: true,
            enable_performance_analysis: true,
            enable_automotive_analysis: true,
            max_length: 1024,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeAnalysisResult {
    pub analysis_id: String,
    pub original_code: String,
    pub language: String,
    pub issues: Vec<CodeIssue>,
    pub security_vulnerabilities: Vec<SecurityVulnerability>,
    pub performance_issues: Vec<PerformanceIssue>,
    pub architecture_recommendations: Vec<ArchitectureRecommendation>,
    pub metrics: CodeMetrics,
    pub summary: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CodeIssue {
    pub id: String,
    pub issue_type: String,
    pub severity: String,
    pub line: usize,
    pub message: String,
    pub suggested_fix: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct SecurityVulnerability {
    pub id: String,
    pub cwe_id: Option<String>,
    pub severity: String,
    pub line: usize,
    pub description: String,
    pub remediation: String,
}

#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub id: String,
    pub issue_type: String,
    pub severity: String,
    pub line: usize,
    pub description: String,
    pub suggested_optimization: String,
}

#[derive(Debug, Clone)]
pub struct ArchitectureRecommendation {
    pub id: String,
    pub recommendation_type: String,
    pub component: String,
    pub description: String,
    pub rationale: String,
    pub effort: String,
    pub impact: String,
}

#[derive(Debug, Clone)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub cyclomatic_complexity: f32,
    pub cognitive_complexity: f32,
    pub maintainability_index: f32,
    pub technical_debt_ratio: f32,
    pub duplication_percentage: f32,
    pub comment_density: f32,
}

pub struct FixCoLlmSystem {
    config: FixCoLlmConfig,
}

impl FixCoLlmSystem {
    pub fn new(config: FixCoLlmConfig) -> Self {
        Self { config }
    }

    pub fn analyze_code(&self, code: &str, language: &str) -> Result<CodeAnalysisResult, String> {
        let analysis_id = format!("analysis_{}", chrono::Utc::now().timestamp());
        let lines: Vec<&str> = code.lines().collect();
        let lines_of_code = lines.len();

        let mut issues = Vec::new();
        let mut security_vulnerabilities = Vec::new();
        let mut performance_issues = Vec::new();
        let mut architecture_recommendations = Vec::new();
        match language.to_lowercase().as_str() {
            "rust" => self.analyze_rust_code(&lines, &mut issues, &mut security_vulnerabilities, &mut performance_issues),
            "python" => self.analyze_python_code(&lines, &mut issues, &mut security_vulnerabilities, &mut performance_issues),
            "javascript" | "typescript" => self.analyze_js_code(&lines, &mut issues, &mut security_vulnerabilities, &mut performance_issues),
            _ => self.analyze_generic_code(&lines, &mut issues, &mut security_vulnerabilities, &mut performance_issues),
        }
        self.generate_architecture_recommendations(&lines, language, &mut architecture_recommendations);
        let metrics = self.calculate_metrics(&lines, language);
        let summary = format!(
            "Analyzed {} lines of {} code. Found {} issues, {} security, {} performance issues, and {} architecture recommendations.",
            lines_of_code, language, issues.len(), security_vulnerabilities.len(), performance_issues.len(), architecture_recommendations.len()
        );
        let mut metadata = HashMap::new();
        metadata.insert("analyzer".to_string(), "FixCo LLM".to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("language".to_string(), language.to_string());

        Ok(CodeAnalysisResult {
            analysis_id,
            original_code: code.to_string(),
            language: language.to_string(),
            issues,
            security_vulnerabilities,
            performance_issues,
            architecture_recommendations,
            metrics,
            summary,
            metadata,
        })
    }

    pub fn generate_code(&self, prompt: &str, language: &str) -> Result<String, String> {
        match language.to_lowercase().as_str() {
            "rust" => self.generate_rust_code(prompt),
            "python" => self.generate_python_code(prompt),
            "javascript" => self.generate_javascript_code(prompt),
            _ => Ok(format!("// Generated {} code for: {}\n// TODO: Implement functionality", language, prompt)),
        }
    }

    fn analyze_rust_code(&self, lines: &[&str], issues: &mut Vec<CodeIssue>, security: &mut Vec<SecurityVulnerability>, performance: &mut Vec<PerformanceIssue>) {
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            if line.contains(".unwrap()") {
                issues.push(CodeIssue {
                    id: format!("rust_unwrap_{}", line_num),
                    issue_type: "potential_panic".to_string(),
                    severity: "warning".to_string(),
                    line: line_num,
                    message: "Use of .unwrap() can cause panic. Using error handling.".to_string(),
                    suggested_fix: Some("Use .expect() with a error handling".to_string()),
                    confidence: 0.9,
                });
            }

            if line.contains("unsafe") {
                security.push(SecurityVulnerability {
                    id: format!("rust_unsafe_{}", line_num),
                    cwe_id: Some("CWE-119".to_string()),
                    severity: "high".to_string(),
                    line: line_num,
                    description: "Unsafe block detected. Review for memory".to_string(),
                    remediation: "Ensure all unsafe operations are valid".to_string(),
                });
            }

            if line.contains("+ &") && line.contains("String") {
                performance.push(PerformanceIssue {
                    id: format!("rust_string_concat_{}", line_num),
                    issue_type: "inefficient_concatenation".to_string(),
                    severity: "medium".to_string(),
                    line: line_num,
                    description: "Need Improve string concatenation detected.".to_string(),
                    suggested_optimization: "Use format!() macro or String::push_str() for fixing".to_string(),
                });
            }
        }
    }

    fn analyze_python_code(&self, lines: &[&str], issues: &mut Vec<CodeIssue>, security: &mut Vec<SecurityVulnerability>, performance: &mut Vec<PerformanceIssue>) {
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            if line.trim() == "except:" {
                issues.push(CodeIssue {
                    id: format!("python_except_{}", line_num),
                    issue_type: "bad_practice".to_string(),
                    severity: "warning".to_string(),
                    line: line_num,
                    message: "Except using '{}' ".to_string(),
                    suggested_fix: Some("Use 'except ValueError:'".to_string()),//Add Lebih banyak
                    confidence: 0.95,
                });
            }

            if line.contains("eval(") {
                security.push(SecurityVulnerability {
                    id: format!("python_eval_{}", line_num),
                    cwe_id: Some("CWE-94".to_string()),
                    severity: "critical".to_string(),
                    line: line_num,
                    description: "Use of eval() can lead to code injection vulnerabilities.".to_string(),
                    remediation: "Avoid eval(). Use ast.literal_eval() for safe evaluation or alternative parsing methods.".to_string(),
                });
            }

            if line.contains("+=") && line.contains("[") {
                performance.push(PerformanceIssue {
                    id: format!("python_list_concat_{}", line_num),
                    issue_type: "inefficient_operation".to_string(),
                    severity: "medium".to_string(),
                    line: line_num,
                    description: "Inefficient list concatenation with += operator.".to_string(),
                    suggested_optimization: "Use list.extend() or list comprehension for better performance.".to_string(),
                });
            }
        }
    }

    fn analyze_js_code(&self, lines: &[&str], issues: &mut Vec<CodeIssue>, security: &mut Vec<SecurityVulnerability>, performance: &mut Vec<PerformanceIssue>) {
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            // Check for == vs ===
            if line.contains("==") && !line.contains("===") {
                issues.push(CodeIssue {
                    id: format!("js_equality_{}", line_num),
                    issue_type: "type_coercion".to_string(),
                    severity: "warning".to_string(),
                    line: line_num,
                    message: "Use === for strict equality comparison instead of ==.".to_string(),
                    suggested_fix: Some("Replace == with === to avoid type coercion".to_string()),
                    confidence: 0.8,
                });
            }

            if line.contains("eval(") {
                security.push(SecurityVulnerability {
                    id: format!("js_eval_{}", line_num),
                    cwe_id: Some("CWE-94".to_string()),
                    severity: "critical".to_string(),
                    line: line_num,
                    description: "Use of eval() can lead to code injection vulnerabilities.".to_string(),
                    remediation: "Avoid eval(). Use JSON.parse() for data or alternative parsing methods.".to_string(),
                });
            }

            if line.contains(".innerHTML") {
                security.push(SecurityVulnerability {
                    id: format!("js_innerHTML_{}", line_num),
                    cwe_id: Some("CWE-79".to_string()),
                    severity: "medium".to_string(),
                    line: line_num,
                    description: "Use of innerHTML can lead to XSS vulnerabilities.".to_string(),
                    remediation: "Use textContent or properly sanitize HTML content.".to_string(),
                });
            }
        }
    }

    fn analyze_generic_code(&self, lines: &[&str], issues: &mut Vec<CodeIssue>, _security: &mut Vec<SecurityVulnerability>, _performance: &mut Vec<PerformanceIssue>) {
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            if line.to_lowercase().contains("todo") {
                issues.push(CodeIssue {
                    id: format!("generic_todo_{}", line_num),
                    issue_type: "incomplete_code".to_string(),
                    severity: "info".to_string(),
                    line: line_num,
                    message: "TODO comment found. Consider implementing or removing.".to_string(),
                    suggested_fix: Some("Implement the functionality or create a proper issue tracker item".to_string()),
                    confidence: 1.0,
                });
            }
        }
    }

    fn generate_architecture_recommendations(&self, lines: &[&str], language: &str, recommendations: &mut Vec<ArchitectureRecommendation>) {
        let lines_count = lines.len();
        if lines_count > 100 {
            recommendations.push(ArchitectureRecommendation {
                id: "arch_large_file".to_string(),
                recommendation_type: "modularity".to_string(),
                component: "file_structure".to_string(),
                description: "Large file detected. Consider breaking into smaller modules.".to_string(),
                rationale: "Smaller files are easier to maintain and test.".to_string(),
                effort: "medium".to_string(),
                impact: "high".to_string(),
            });
        }

        match language.to_lowercase().as_str() {
            "rust" => {
                if lines.iter().any(|line| line.contains("pub fn")) {
                    recommendations.push(ArchitectureRecommendation {
                        id: "rust_documentation".to_string(),
                        recommendation_type: "documentation".to_string(),
                        component: "public_api".to_string(),
                        description: "Add documentation comments to public functions.".to_string(),
                        rationale: "Public APIs should be well documented for users.".to_string(),
                        effort: "low".to_string(),
                        impact: "medium".to_string(),
                    });
                }
            },
            "python" => {
                if lines.iter().any(|line| line.contains("def ")) {
                    recommendations.push(ArchitectureRecommendation {
                        id: "python_type_hints".to_string(),
                        recommendation_type: "type_safety".to_string(),
                        component: "functions".to_string(),
                        description: "Add type hints to function signatures.".to_string(),
                        rationale: "Type hints improve code readability and enable better tooling.".to_string(),
                        effort: "low".to_string(),
                        impact: "medium".to_string(),
                    });
                }
            },
            _ => {}
        }
    }

    fn calculate_metrics(&self, lines: &[&str], _language: &str) -> CodeMetrics {
        let lines_of_code = lines.len();
        let function_count = lines.iter().filter(|line| {
            line.contains("fn ") || line.contains("def ") || line.contains("function ")
        }).count();
        let class_count = lines.iter().filter(|line| {
            line.contains("class ") || line.contains("struct ") || line.contains("impl ")
        }).count();
        let comment_lines = lines.iter().filter(|line| {
            line.trim_start().starts_with("//") || line.trim_start().starts_with("#") || line.trim_start().starts_with("/*")
        }).count();
        let comment_density = if lines_of_code > 0 {
            (comment_lines as f32 / lines_of_code as f32) * 100.0
        } else {
            0.0
        };
        let complexity_keywords = ["if", "else", "for", "while", "match", "case", "try", "catch"];
        let cyclomatic_complexity = lines.iter().map(|line| {
            complexity_keywords.iter().filter(|&&keyword| line.contains(keyword)).count() as f32
        }).sum::<f32>() + 1.0;
        CodeMetrics {
            lines_of_code,
            function_count,
            class_count,
            cyclomatic_complexity,
            cognitive_complexity: cyclomatic_complexity * 1.2,
            maintainability_index: 100.0 - (cyclomatic_complexity * 2.0).min(100.0),
            technical_debt_ratio: (cyclomatic_complexity / lines_of_code as f32) * 100.0,
            duplication_percentage: 0.0,
            comment_density,
        }
    }

    fn generate_rust_code(&self, prompt: &str) -> Result<String, String> {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("factorial") {
            Ok(r#"fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
    }
}"#.to_string())
        } else if prompt_lower.contains("add") && prompt_lower.contains("number") {
            Ok(r#"fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_numbers() {
        assert_eq!(add_numbers(2, 3), 5);
        assert_eq!(add_numbers(-1, 1), 0);
    }
}"#.to_string())
        } else {
            Ok(format!(r#"// Generated Rust code for: {}
fn generated_function() -> Result<(), Box<dyn std::error::Error>> {{
    // TODO: Implement functionality based on prompt
    println!("Generated function for: {}", "{}");
    Ok(())
}}"#, prompt, prompt, prompt))
        }
    }

    fn generate_python_code(&self, prompt: &str) -> Result<String, String> {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("factorial") {
            Ok(r#"def factorial(n: int) -> int:
    """Calculate the factorial of a number."""
    if n < 0:
        raise ValueError("Factorial is not defined for negative numbers")
    if n == 0 or n == 1:
        return 1
    return n * factorial(n - 1)

# Test
if __name__ == "__main__":
    print(f"factorial(5) = {factorial(5)}")  # Should print 120"#.to_string())
        } else if prompt_lower.contains("add") && prompt_lower.contains("number") {
            Ok(r#"def add_numbers(a: int, b: int) -> int:
    """Add two numbers and return the result."""
    return a + b

# Test
if __name__ == "__main__":
    result = add_numbers(2, 3)
    print(f"2 + 3 = {result}")  # Should print 5"#.to_string())
        } else {
            Ok(format!(r#"# Generated Python code for: {}
def generated_function():
    """Generated function based on prompt."""
    # TODO: Implement functionality based on prompt
    print("Generated function for: {}")
    pass

if __name__ == "__main__":
    generated_function()"#, prompt, prompt))
        }
    }

    fn generate_javascript_code(&self, prompt: &str) -> Result<String, String> {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("factorial") {
            Ok(r#"function factorial(n) {
    if (n < 0) {
        throw new Error("Factorial is not defined for negative numbers");
    }
    if (n === 0 || n === 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

// Test
console.log(`factorial(5) = ${factorial(5)}`); // Should print 120"#.to_string())
        } else if prompt_lower.contains("add") && prompt_lower.contains("number") {
            Ok(r#"function addNumbers(a, b) {
    return a + b;
}

// Test
const result = addNumbers(2, 3);
console.log(`2 + 3 = ${result}`); // Should print 5"#.to_string())
        } else {
            Ok(format!(r#"// Generated JavaScript code for: {}
function generatedFunction() {{
    // TODO: Implement functionality based on prompt
    console.log("Generated function for: {}");
}}

// Call the function
generatedFunction();"#, prompt, prompt))
        }
    }
}

pub fn fixco_module(lang_detect: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
    let config = FixCoLlmConfig::default();
    let system = FixCoLlmSystem::new(config);

    let mut think_code = r#""#;
    let mut lang_think_code = "";

    println!("fixco_module prepare lang detect");

    for lang_detail in &lang_detect {
        lang_think_code = &lang_detail;
        println!("think code = {}", lang_think_code);
    }

    println!("fixco_module prepare lang result");

    if "rust".eq(lang_think_code) {
        let think_code = r#"
fn main() {
    let data = vec![1, 2, 3];
    let result = data.get(10).unwrap(); // Potential panic
    println!("Result: {}", result);
}
"#;
    }

    println!("fixco_module prepare lang start think analizer");

    match system.analyze_code(think_code, lang_think_code) {//using "rust" first
        Ok(analysis) => {
            println!("Analysis ID: {}", analysis.analysis_id);
            println!("Language: {}", analysis.language);
            println!("Lines of code: {}", analysis.metrics.lines_of_code);
            println!("Issues found: {}", analysis.issues.len());
            println!("Security: {}", analysis.security_vulnerabilities.len());
            println!("Performance issues: {}", analysis.performance_issues.len());
            println!("Summary: {}", analysis.summary);
            for issue in &analysis.issues {
                println!("  - Issue: {} (Line {}): {}", issue.issue_type, issue.line, issue.message);
            }
        }
        Err(e) => println!("Error analyzing code: {}", e),
    }

    let mut result_generated = String::new();

    println!("**** Generating Code ****");
    match system.generate_code("A function Done ", lang_think_code) {
        Ok(generated) => {
            println!("Generated {:?} code:",lang_think_code);
            println!("{}", generated);

            result_generated = generated;
        }
        Err(e) => println!("Error generating code: {}", e),
    }

    println!("FixCo AiS's LLM DONE");

    Ok(result_generated)
}

mod chrono {
    use std::time::{SystemTime, UNIX_EPOCH};
    pub struct Utc;
    impl Utc {
        pub fn now() -> DateTime {
            DateTime {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            }
        }
    }
    pub struct DateTime {
        timestamp: i64,
    }
    impl DateTime {
        pub fn timestamp(&self) -> i64 {
            self.timestamp
        }
        pub fn to_rfc3339(&self) -> String {
            format!("2025-10-12T{}:00:00Z", self.timestamp % 24)
        }
    }
}
