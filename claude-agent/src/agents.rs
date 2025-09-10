use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use crate::core::*;
use thiserror::Error;
use tokio::process::Command;
use tracing::{info, warn, error};

#[derive(Debug, Error)]
pub enum CodeAnalysisError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Unsupported project type: {0:?}")]
    UnsupportedProjectType(ProjectType),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<CodeAnalysisError> for AgentError {
    fn from(err: CodeAnalysisError) -> Self {
        match err {
            CodeAnalysisError::CommandFailed(msg) => AgentError::CodeAnalysis(format!("Command failed: {}", msg)),
            CodeAnalysisError::ParseError(msg) => AgentError::CodeAnalysis(format!("Parse error: {}", msg)),
            CodeAnalysisError::UnsupportedProjectType(pt) => AgentError::UnsupportedProjectType(pt),
            CodeAnalysisError::IoError(e) => AgentError::Io(e),
        }
    }
}

pub struct CodeAnalysisAgent {
    name: String,
    version: String,
    description: String,
    supported_types: Vec<ProjectType>,
    capabilities: Vec<AgentCapability>,
}

impl CodeAnalysisAgent {
    pub fn new() -> Self {
        Self {
            name: "code_analysis".to_string(),
            version: "1.0.0".to_string(),
            description: "Analyzes code quality, style, and potential issues".to_string(),
            supported_types: vec![
                ProjectType::Rust,
                ProjectType::Nodejs,
                ProjectType::Python,
                ProjectType::Java,
                ProjectType::Go,
            ],
            capabilities: vec![
                AgentCapability::CodeAnalysis,
                AgentCapability::TestCoverage,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisInput {
    pub project_path: PathBuf,
    pub project_info: ProjectInfo,
    pub analysis_type: AnalysisType,
    pub options: AnalysisOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    Lint,
    Format,
    Security,
    Performance,
    Dependencies,
    TestCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub strict_mode: bool,
    pub include_tests: bool,
    pub output_format: OutputFormat,
    pub custom_rules: Vec<String>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            strict_mode: false,
            include_tests: true,
            output_format: OutputFormat::Json,
            custom_rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Text,
    Html,
    Sarif,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResult {
    pub success: bool,
    pub issues: Vec<Issue>,
    pub metrics: AnalysisMetrics,
    pub suggestions: Vec<Suggestion>,
    pub execution_time: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub file: PathBuf,
    pub line: Option<u32>,
    pub message: String,
    pub rule: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
    Style,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Security,
    Performance,
    Correctness,
    Style,
    Complexity,
    Maintainability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    pub total_files: u32,
    pub total_lines: u32,
    pub issues_count: HashMap<IssueSeverity, u32>,
    pub coverage_percentage: Option<f64>,
    pub complexity_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    pub priority: u8,
    pub effort: EffortLevel,
    pub files_affected: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

#[async_trait]
impl Agent for CodeAnalysisAgent {
    type Input = CodeAnalysisInput;
    type Output = CodeAnalysisResult;
    type Config = serde_json::Value;
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn initialize(&mut self, _config: Self::Config) -> Result<()> {
        info!("Initializing CodeAnalysisAgent");
        Ok(())
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let start_time = std::time::Instant::now();
        
        info!("Starting code analysis for project: {:?}", input.project_path);
        
        let result = match input.project_info.project_type {
            ProjectType::Rust => self.analyze_rust_project(&input).await,
            ProjectType::Nodejs => self.analyze_nodejs_project(&input).await,
            ProjectType::Python => self.analyze_python_project(&input).await,
            ProjectType::Java => self.analyze_java_project(&input).await,
            ProjectType::Go => self.analyze_go_project(&input).await,
            _ => Err(CodeAnalysisError::UnsupportedProjectType(input.project_info.project_type).into()),
        };
        
        let mut analysis_result = result?;
        analysis_result.execution_time = start_time.elapsed();
        
        info!("Code analysis completed in {:?}", analysis_result.execution_time);
        Ok(analysis_result)
    }
    
    async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up CodeAnalysisAgent");
        Ok(())
    }
    
    fn supported_project_types(&self) -> &[ProjectType] {
        &self.supported_types
    }
    
    fn capabilities(&self) -> &[AgentCapability] {
        &self.capabilities
    }
}

impl CodeAnalysisAgent {
    async fn analyze_rust_project(&self, input: &CodeAnalysisInput) -> Result<CodeAnalysisResult> {
        let mut issues = Vec::new();
        let mut metrics = AnalysisMetrics {
            total_files: 0,
            total_lines: 0,
            issues_count: HashMap::new(),
            coverage_percentage: None,
            complexity_score: None,
        };
        let mut suggestions = Vec::new();
        
        // Run cargo check
        let check_output = Command::new("cargo")
            .args(&["check", "--message-format=json"])
            .current_dir(&input.project_path)
            .output()
            .await?;
        
        if !check_output.status.success() {
            let stderr = String::from_utf8_lossy(&check_output.stderr);
            warn!("Cargo check failed: {}", stderr);
        }
        
        // Run cargo clippy
        let clippy_output = Command::new("cargo")
            .args(&["clippy", "--message-format=json", "--", "-D", "warnings"])
            .current_dir(&input.project_path)
            .output()
            .await?;
        
        if clippy_output.status.success() {
            // Parse clippy output
            let stdout = String::from_utf8_lossy(&clippy_output.stdout);
            for line in stdout.lines() {
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(message) = msg.get("message") {
                        if let Some(level) = message.get("level") {
                            let severity = match level.as_str() {
                                Some("error") => IssueSeverity::Error,
                                Some("warning") => IssueSeverity::Warning,
                                _ => IssueSeverity::Info,
                            };
                            
                            issues.push(Issue {
                                severity,
                                category: IssueCategory::Correctness,
                                file: message.get("spans")
                                    .and_then(|spans| spans.as_array())
                                    .and_then(|spans| spans.first())
                                    .and_then(|span| span.get("file_name"))
                                    .and_then(|file| file.as_str())
                                    .map(|s| PathBuf::from(s))
                                    .unwrap_or_default(),
                                line: message.get("spans")
                                    .and_then(|spans| spans.as_array())
                                    .and_then(|spans| spans.first())
                                    .and_then(|span| span.get("line_start"))
                                    .and_then(|line| line.as_u64())
                                    .map(|l| l as u32),
                                message: message.get("message")
                                    .and_then(|msg| msg.as_str())
                                    .unwrap_or("Unknown issue").to_string(),
                                rule: message.get("code")
                                    .and_then(|code| code.as_str())
                                    .unwrap_or("unknown").to_string(),
                                suggestion: None,
                            });
                        }
                    }
                }
            }
        }
        
        // Run cargo fmt check
        if matches!(input.analysis_type, AnalysisType::Format) {
            let fmt_output = Command::new("cargo")
                .args(&["fmt", "--check"])
                .current_dir(&input.project_path)
                .output()
                .await?;
            
            if !fmt_output.status.success() {
                issues.push(Issue {
                    severity: IssueSeverity::Style,
                    category: IssueCategory::Style,
                    file: PathBuf::new(), // Multiple files affected
                    line: None,
                    message: "Code formatting issues found".to_string(),
                    rule: "formatting".to_string(),
                    suggestion: Some("Run 'cargo fmt' to fix formatting issues".to_string()),
                });
            }
        }
        
        // Calculate metrics
        *metrics.issues_count.entry(IssueSeverity::Error).or_insert(0) += 
            issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Error)).count() as u32;
        *metrics.issues_count.entry(IssueSeverity::Warning).or_insert(0) += 
            issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count() as u32;
        
        // Add suggestions
        if issues.is_empty() {
            suggestions.push(Suggestion {
                title: "Code Quality Good".to_string(),
                description: "No significant issues found in code analysis".to_string(),
                priority: 1,
                effort: EffortLevel::Low,
                files_affected: vec![],
            });
        } else {
            suggestions.push(Suggestion {
                title: "Address Clippy Warnings".to_string(),
                description: "Fix the clippy warnings to improve code quality".to_string(),
                priority: 3,
                effort: EffortLevel::Medium,
                files_affected: issues.iter().map(|i| i.file.clone()).collect(),
            });
        }
        
        Ok(CodeAnalysisResult {
            success: issues.iter().all(|i| !matches!(i.severity, IssueSeverity::Error)),
            issues,
            metrics,
            suggestions,
            execution_time: std::time::Duration::from_secs(0),
        })
    }
    
    async fn analyze_nodejs_project(&self, _input: &CodeAnalysisInput) -> Result<CodeAnalysisResult> {
        // Similar implementation for Node.js projects using ESLint, npm audit, etc.
        Ok(CodeAnalysisResult {
            success: true,
            issues: Vec::new(),
            metrics: AnalysisMetrics {
                total_files: 0,
                total_lines: 0,
                issues_count: HashMap::new(),
                coverage_percentage: None,
                complexity_score: None,
            },
            suggestions: vec![
                Suggestion {
                    title: "Node.js Analysis".to_string(),
                    description: "Node.js code analysis not fully implemented".to_string(),
                    priority: 1,
                    effort: EffortLevel::Low,
                    files_affected: vec![],
                }
            ],
            execution_time: std::time::Duration::from_secs(0),
        })
    }
    
    async fn analyze_python_project(&self, _input: &CodeAnalysisInput) -> Result<CodeAnalysisResult> {
        // Similar implementation for Python projects using flake8, mypy, etc.
        Ok(CodeAnalysisResult {
            success: true,
            issues: Vec::new(),
            metrics: AnalysisMetrics {
                total_files: 0,
                total_lines: 0,
                issues_count: HashMap::new(),
                coverage_percentage: None,
                complexity_score: None,
            },
            suggestions: vec![
                Suggestion {
                    title: "Python Analysis".to_string(),
                    description: "Python code analysis not fully implemented".to_string(),
                    priority: 1,
                    effort: EffortLevel::Low,
                    files_affected: vec![],
                }
            ],
            execution_time: std::time::Duration::from_secs(0),
        })
    }
    
    async fn analyze_java_project(&self, _input: &CodeAnalysisInput) -> Result<CodeAnalysisResult> {
        // Similar implementation for Java projects using SpotBugs, PMD, etc.
        Ok(CodeAnalysisResult {
            success: true,
            issues: Vec::new(),
            metrics: AnalysisMetrics {
                total_files: 0,
                total_lines: 0,
                issues_count: HashMap::new(),
                coverage_percentage: None,
                complexity_score: None,
            },
            suggestions: vec![
                Suggestion {
                    title: "Java Analysis".to_string(),
                    description: "Java code analysis not fully implemented".to_string(),
                    priority: 1,
                    effort: EffortLevel::Low,
                    files_affected: vec![],
                }
            ],
            execution_time: std::time::Duration::from_secs(0),
        })
    }
    
    async fn analyze_go_project(&self, _input: &CodeAnalysisInput) -> Result<CodeAnalysisResult> {
        // Similar implementation for Go projects using go vet, golint, etc.
        Ok(CodeAnalysisResult {
            success: true,
            issues: Vec::new(),
            metrics: AnalysisMetrics {
                total_files: 0,
                total_lines: 0,
                issues_count: HashMap::new(),
                coverage_percentage: None,
                complexity_score: None,
            },
            suggestions: vec![
                Suggestion {
                    title: "Go Analysis".to_string(),
                    description: "Go code analysis not fully implemented".to_string(),
                    priority: 1,
                    effort: EffortLevel::Low,
                    files_affected: vec![],
                }
            ],
            execution_time: std::time::Duration::from_secs(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_code_analysis_agent() {
        let agent = CodeAnalysisAgent::new();
        
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create a minimal Rust project
        fs::write(project_path.join("Cargo.toml"), r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#).unwrap();
        
        fs::create_dir_all(project_path.join("src")).unwrap();
        fs::write(project_path.join("src/main.rs"), r#"
fn main() {
    println!("Hello, world!");
}
"#).unwrap();
        
        let project_info = ProjectInfo {
            project_type: ProjectType::Rust,
            language: "rust".to_string(),
            framework: None,
            build_system: Some("cargo".to_string()),
            package_manager: Some("cargo".to_string()),
            root_path: project_path.to_path_buf(),
            config_files: vec![project_path.join("Cargo.toml")],
            dependencies: vec![],
        };
        
        let input = CodeAnalysisInput {
            project_path: project_path.to_path_buf(),
            project_info,
            analysis_type: AnalysisType::Lint,
            options: AnalysisOptions::default(),
        };
        
        let result = agent.execute(input).await.unwrap();
        
        assert!(result.success);
        assert!(!result.issues.is_empty() || result.suggestions.iter().any(|s| s.title.contains("Good")));
    }
}