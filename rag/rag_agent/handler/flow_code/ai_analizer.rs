use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use reqwest::Client;
use uuid::Uuid;
use colored::*;
use crate::domain::flow_code::*;
//crates. rust

pub struct AICodeAnalyzer {
    ollama_url: String,
    model_name: String,
    client: Client,
}

impl AICodeAnalyzer {
    pub fn new(ollama_url: Option<String>, model_name: Option<String>) -> Self {
        Self {
            ollama_url: ollama_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model_name: model_name.unwrap_or_else(|| "asist:latest".to_string()),
            client: Client::new(),
        }
    }

    pub async fn analyze_with_ai(&self, project_analysis: &ProjectAnalysis, project_path: &Path) -> Result<AIAnalysisResult> {
        println!("{}", "Starting analysis...".bright_cyan());

        let analysis_id = Uuid::new_v4().to_string();

        println!("{}", " Analyzing code stack...".bright_blue());
        let complexity_analysis = self.analyze_complexity(project_analysis, project_path).await?;

        println!("{}", "  Generating architecture recommendations...".bright_blue());
        let architecture_recommendations = self.generate_architecture_recommendations(project_analysis).await?;

        println!("{}", "  Performing security...".bright_blue());
        let security_assessment = self.assess_security(project_analysis, project_path).await?;

        println!("{}", "  Analyzing patterns...".bright_blue());
        let performance_insights = self.analyze_performance(project_analysis, project_path).await?;

        println!("{}", "  Generating insights...".bright_blue());
        let ai_insights = self.generate_ai_insights(project_analysis, project_path).await?;

        let code_quality_score = self.calculate_quality_score(&complexity_analysis, &security_assessment, &ai_insights);

        println!("{}", "  Generating natural language summary...".bright_blue());
        let natural_language_summary = self.generate_summary(
            project_analysis,
            &ai_insights,
            &architecture_recommendations,
            code_quality_score
        ).await?;

        Ok(AIAnalysisResult {
            analysis_id,
            project_name: project_analysis.name.clone(),
            ai_insights,
            architecture_recommendations,
            code_quality_score,
            complexity_analysis,
            security_assessment,
            performance_insights,
            natural_language_summary,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    async fn analyze_complexity(&self, project_analysis: &ProjectAnalysis, project_path: &Path) -> Result<ComplexityAnalysis> {
        let prompt = self.create_complexity_analysis_prompt(project_analysis);
        let response = self.query_ollama(&prompt).await?;

        // Parse AI response, extract metrics
        // combined with AI insights
        let cyclomatic_complexity = self.estimate_cyclomatic_complexity(project_analysis);
        let cognitive_complexity = self.estimate_cognitive_complexity(project_analysis);
        let maintainability_index = self.calculate_maintainability_index(project_analysis);
        let technical_debt_ratio = self.estimate_technical_debt(project_analysis);

        let hotspots = self.identify_complexity_hotspots(project_analysis, &response).await?;

        Ok(ComplexityAnalysis {
            cyclomatic_complexity,
            cognitive_complexity,
            maintainability_index,
            technical_debt_ratio,
            hotspots,
        })
    }

    async fn generate_architecture_recommendations(&self, project_analysis: &ProjectAnalysis) -> Result<Vec<ArchitectureRecommendation>> {
        let prompt = format!(
            r#"Analyze this software project and arch recommendations:

Project: {}
Language: {}
Framework: {:?}
Files: {}
Dependencies: {:?}

Recommendations on:
1. Code org and struct
2. Design patterns
3. Improvements
4. Maintain
5. Optimizations

Format recommendations part."#,
            project_analysis.name,
            project_analysis.language,
            project_analysis.framework,
            project_analysis.total_files,
            project_analysis.dependencies.iter().take(10).collect::<Vec<_>>()
        );

        let response = self.query_ollama(&prompt).await?;
        self.parse_architecture_recommendations(&response)
    }

    async fn assess_security(&self, project_analysis: &ProjectAnalysis, project_path: &Path) -> Result<SecurityAssessment> {
        let prompt = self.create_security_analysis_prompt(project_analysis);
        let response = self.query_ollama(&prompt).await?;

        // rule-based security && AI insights
        let vulnerabilities = self.identify_security_vulnerabilities(project_analysis, &response).await?;
        let security_patterns = self.identify_security_patterns(&response);
        let compliance_status = self.check_compliance_status(project_analysis);
        let overall_score = self.calculate_security_score(&vulnerabilities, &security_patterns);

        Ok(SecurityAssessment {
            overall_score,
            vulnerabilities,
            security_patterns,
            compliance_status,
        })
    }

    async fn analyze_performance(&self, project_analysis: &ProjectAnalysis, project_path: &Path) -> Result<Vec<PerformanceInsight>> {
        let prompt = self.create_performance_analysis_prompt(project_analysis);
        let response = self.query_ollama(&prompt).await?;

        self.parse_performance_insights(&response, project_analysis)
    }

    async fn generate_ai_insights(&self, project_analysis: &ProjectAnalysis, project_path: &Path) -> Result<Vec<AIInsight>> {
        let mut insights = Vec::new();

        for issue in &project_analysis.issues_found {
            let prompt = format!(
                r#"Analyze code issue and insights:

File: {}
Line: {}
Issue: {}
Severity: {}

Provide:
1. Explanation of issue
2. Effect
3. Suggestion Fixing it
4. Alternative approaches
5. Dev Next Impl

context suggestions and pattern detection."#,
                issue.file, issue.line, issue.message, issue.severity
            );

            let response = self.query_ollama(&prompt).await?;
            let ai_insight = self.parse_ai_insight(&response, issue)?;
            insights.push(ai_insight);
        }

        let project_insights = self.generate_project_level_insights(project_analysis).await?;
        insights.extend(project_insights);

        Ok(insights)
    }

    async fn generate_summary(
        &self,
        project_analysis: &ProjectAnalysis,
        ai_insights: &[AIInsight],
        architecture_recommendations: &[ArchitectureRecommendation],
        code_quality_score: f64,
    ) -> Result<String> {
        let prompt = format!(
            r#"Generate a summary of this code analysis:

Project: {} ({})
Quality Score: {:.1}/10
Total Issues: {}
Archi Recommendations: {}
PIN AI Insights: {}

Create detailed summary that explains:
1. Project health and quality
2. Improvement
3. Issues to address
4. Recommended next
5. Next Dev Recommendations

Write in analysis insights"#,
            project_analysis.name,
            project_analysis.language,
            code_quality_score,
            project_analysis.issues_found.len(),
            architecture_recommendations.len(),
            ai_insights.len()
        );

        let response = self.query_ollama(&prompt).await?;
        Ok(response)
    }

    async fn query_ollama(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature: 0.7,
                top_p: 0.9,
                max_tokens: Some(2048),
            },
        };

        let response = self.client
            .post(&format!("{}/api/generate", self.ollama_url))
            .json(&request)
            .send()
            .await
            .context("Error request to Ollama")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Ollama API error: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response.json().await
            .context("Error to parse Ollama response")?;

        Ok(ollama_response.response)
    }

    // Helper methods for analysis
    fn create_complexity_analysis_prompt(&self, project_analysis: &ProjectAnalysis) -> String {
        format!(
            r#"Analyze complexity of io project:

Project: {}
Language: {}
Files: {}
Lines of Code: {}
Issues Found: {}

Identify structure stack complexibility project."#,
            project_analysis.name,
            project_analysis.language,
            project_analysis.total_files,
            project_analysis.total_lines,
            project_analysis.issues_found.len()
        )
    }

    fn create_security_analysis_prompt(&self, project_analysis: &ProjectAnalysis) -> String {
        format!(
            r#"Run security of io project:

Project: {}
Language: {}
Framework: {:?}
Dependencies: {:?}

Identify security for {} applications."#,
            project_analysis.name,
            project_analysis.language,
            project_analysis.framework,
            project_analysis.dependencies.iter().take(5).collect::<Vec<_>>(),
            project_analysis.language
        )
    }

    fn create_performance_analysis_prompt(&self, project_analysis: &ProjectAnalysis) -> String {
        format!(
            r#"Analyze software io project:

Project: {}
Language: {}
Framework: {:?}
Files: {}
Lines: {}

Identify, optimization, and scale."#,
            project_analysis.name,
            project_analysis.language,
            project_analysis.framework,
            project_analysis.total_files,
            project_analysis.total_lines
        )
    }

    fn estimate_cyclomatic_complexity(&self, project_analysis: &ProjectAnalysis) -> f64 {
        (project_analysis.total_files as f64 * 1.5) + (project_analysis.issues_found.len() as f64 * 0.5)
    }

    fn estimate_cognitive_complexity(&self, project_analysis: &ProjectAnalysis) -> f64 {
        project_analysis.total_lines as f64 / 100.0
    }

    fn calculate_maintainability_index(&self, project_analysis: &ProjectAnalysis) -> f64 {
        let base_score = 100.0;
        let issue_penalty = project_analysis.issues_found.len() as f64 * 2.0;
        let complexity_penalty = (project_analysis.total_lines as f64 / 1000.0) * 5.0;

        (base_score - issue_penalty - complexity_penalty).max(0.0)
    }

    fn estimate_technical_debt(&self, project_analysis: &ProjectAnalysis) -> f64 {
        project_analysis.issues_found.len() as f64 / project_analysis.total_files as f64
    }

    fn calculate_quality_score(&self, complexity: &ComplexityAnalysis, security: &SecurityAssessment, insights: &[AIInsight]) -> f64 {
        let complexity_score = (100.0 - complexity.technical_debt_ratio * 10.0).max(0.0);
        let security_score = security.overall_score * 10.0;
        let insight_penalty = insights.iter()
            .filter(|i| i.severity == "High")
            .count() as f64 * 5.0;

        ((complexity_score + security_score) / 2.0 - insight_penalty).max(0.0).min(10.0)
    }

    async fn identify_complexity_hotspots(&self, _project_analysis: &ProjectAnalysis, _ai_response: &str) -> Result<Vec<ComplexityHotspot>> {
        Ok(vec![])
    }

    fn parse_architecture_recommendations(&self, _response: &str) -> Result<Vec<ArchitectureRecommendation>> {
        Ok(vec![
            ArchitectureRecommendation {
                category: "Code Organization".to_string(),
                title: "Implement Modular Architecture".to_string(),
                description: "Organizing code into modules for maintainability".to_string(),
                rationale: "Modular improves code organization and testability".to_string(),
                implementation_steps: vec![
                    "Identify functionality".to_string(),
                    "Create modules".to_string(),
                    "Define interfaces".to_string(),
                ],
                estimated_effort: "Medium".to_string(),
                priority: "High".to_string(),
            }
        ])
    }

    async fn identify_security_vulnerabilities(&self, _project_analysis: &ProjectAnalysis, _ai_response: &str) -> Result<Vec<SecurityVulnerability>> {
        Ok(vec![])
    }

    fn identify_security_patterns(&self, _response: &str) -> Vec<String> {
        vec!["Input validation".to_string(), "Error handling".to_string()]
    }

    fn check_compliance_status(&self, _project_analysis: &ProjectAnalysis) -> HashMap<String, bool> {
        let mut compliance = HashMap::new();
        compliance.insert("OWASP".to_string(), true);
        compliance.insert("GDPR".to_string(), false);
        compliance
    }

    fn calculate_security_score(&self, vulnerabilities: &[SecurityVulnerability], _patterns: &[String]) -> f64 {
        let high_vuln_count = vulnerabilities.iter().filter(|v| v.severity == "High").count();
        (10.0 - high_vuln_count as f64).max(0.0)
    }

    fn parse_performance_insights(&self, _response: &str, _project_analysis: &ProjectAnalysis) -> Result<Vec<PerformanceInsight>> {
        Ok(vec![])
    }

    fn parse_ai_insight(&self, ai_response: &str, original_issue: &CodeIssue) -> Result<AIInsight> {
        Ok(AIInsight {
            insight_type: "PIN's AI Analysis".to_string(),
            file_path: original_issue.file.clone(),
            line_range: Some((original_issue.line, original_issue.line)),
            severity: original_issue.severity.clone(),
            title: format!("PIN's AI Insight: {}", original_issue.message),
            description: original_issue.message.clone(),
            explanation: ai_response.chars().take(500).collect::<String>() + "...",
            suggested_fix: original_issue.suggestion.clone().unwrap_or_else(|| "No specific suggestion available".to_string()),
            confidence_score: 0.85,
            tags: vec!["ai-generated".to_string(), "context-aware".to_string()],
        })
    }

    async fn generate_project_level_insights(&self, _project_analysis: &ProjectAnalysis) -> Result<Vec<AIInsight>> {
        Ok(vec![])
    }
}
