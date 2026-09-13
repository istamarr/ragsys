use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};


/** ai-analizer **/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResult {
    pub analysis_id: String,
    pub project_name: String,
    pub ai_insights: Vec<AIInsight>,
    pub architecture_recommendations: Vec<ArchitectureRecommendation>,
    pub code_quality_score: f64,
    pub complexity_analysis: ComplexityAnalysis,
    pub security_assessment: SecurityAssessment,
    pub performance_insights: Vec<PerformanceInsight>,
    pub natural_language_summary: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInsight {
    pub insight_type: String,
    pub file_path: String,
    pub line_range: Option<(usize, usize)>,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub explanation: String,
    pub suggested_fix: String,
    pub confidence_score: f64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureRecommendation {
    pub category: String,
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub implementation_steps: Vec<String>,
    pub estimated_effort: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    pub cyclomatic_complexity: f64,
    pub cognitive_complexity: f64,
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
    pub hotspots: Vec<ComplexityHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHotspot {
    pub file_path: String,
    pub function_name: String,
    pub complexity_score: f64,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssessment {
    pub overall_score: f64,
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub security_patterns: Vec<String>,
    pub compliance_status: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub cve_id: Option<String>,
    pub severity: String,
    pub category: String,
    pub description: String,
    pub affected_files: Vec<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsight {
    pub category: String,
    pub file_path: String,
    pub issue_description: String,
    pub impact_level: String,
    pub optimization_suggestion: String,
    pub estimated_improvement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: OllamaOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaOptions {
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
    pub context: Option<Vec<i32>>,
}

/** analizer **/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub name: String,
    pub root_path: String,
    pub language: String,
    pub framework: Option<String>,
    pub total_files: usize,
    pub total_lines: usize,
    pub file_types: HashMap<String, usize>,
    pub issues_found: Vec<CodeIssue>,
    pub dependencies: Vec<String>,
    pub structure_tree: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    pub file: String,
    pub line: usize,
    pub severity: String,
    pub message: String,
    pub suggestion: Option<String>,
}