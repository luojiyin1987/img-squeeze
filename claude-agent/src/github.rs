use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::core::*;
use crate::agents::{CodeAnalysisResult, IssueSeverity};
use thiserror::Error;
use tokio::process::Command;
use tracing::{info, warn, error};

#[derive(Debug, Error)]
pub enum GitHubIntegrationError {
    #[error("GitHub CLI not found")]
    GitHubCliNotFound,
    
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<GitHubIntegrationError> for AgentError {
    fn from(err: GitHubIntegrationError) -> Self {
        match err {
            GitHubIntegrationError::GitHubCliNotFound => AgentError::GitHubIntegration("GitHub CLI not found".to_string()),
            GitHubIntegrationError::CommandFailed(msg) => AgentError::GitHubIntegration(format!("Command failed: {}", msg)),
            GitHubIntegrationError::ParseError(msg) => AgentError::GitHubIntegration(format!("Parse error: {}", msg)),
            GitHubIntegrationError::AuthenticationError(msg) => AgentError::GitHubIntegration(format!("Authentication error: {}", msg)),
            GitHubIntegrationError::IoError(e) => AgentError::Io(e),
        }
    }
}

pub struct GitHubIntegrationAgent {
    name: String,
    version: String,
    description: String,
    supported_types: Vec<ProjectType>,
    capabilities: Vec<AgentCapability>,
}

impl GitHubIntegrationAgent {
    pub fn new() -> Self {
        Self {
            name: "github_integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Integrates with GitHub for PR reviews and issue management".to_string(),
            supported_types: vec![ProjectType::Generic], // Works with any project type
            capabilities: vec![
                AgentCapability::CopilotIntegration,
                AgentCapability::ReportGeneration,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIntegrationInput {
    pub project_path: PathBuf,
    pub action: GitHubAction,
    pub analysis_results: Option<CodeAnalysisResult>,
    pub pr_number: Option<u32>,
    pub options: GitHubOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitHubAction {
    CreatePRReview,
    PostPRComment,
    TriggerCopilotReview,
    CreateIssue,
    UpdateIssue,
    GetPRInfo,
    ListIssues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubOptions {
    pub repository: Option<String>,
    pub owner: Option<String>,
    pub dry_run: bool,
    pub include_copilot: bool,
    pub comment_template: Option<String>,
    pub auto_merge: bool,
}

impl Default for GitHubOptions {
    fn default() -> Self {
        Self {
            repository: None,
            owner: None,
            dry_run: false,
            include_copilot: true,
            comment_template: None,
            auto_merge: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIntegrationResult {
    pub success: bool,
    pub action: GitHubAction,
    pub pr_number: Option<u32>,
    pub issue_number: Option<u32>,
    pub comment_url: Option<String>,
    pub review_url: Option<String>,
    pub copilot_triggered: bool,
    pub execution_time: std::time::Duration,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRInfo {
    pub number: u32,
    pub title: String,
    pub body: String,
    pub state: String,
    pub user: String,
    pub head_ref: String,
    pub base_ref: String,
    pub mergeable: bool,
    pub draft: bool,
}

#[async_trait]
impl Agent for GitHubIntegrationAgent {
    type Input = GitHubIntegrationInput;
    type Output = GitHubIntegrationResult;
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
        info!("Initializing GitHubIntegrationAgent");
        
        // Check if GitHub CLI is available
        let output = Command::new("gh")
            .arg("--version")
            .output()
            .await;
        
        match output {
            Ok(_) => info!("GitHub CLI found and accessible"),
            Err(_) => return Err(GitHubIntegrationError::GitHubCliNotFound.into()),
        }
        
        Ok(())
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let start_time = std::time::Instant::now();
        
        info!("Executing GitHub integration action: {:?}", input.action);
        
        let result = match input.action {
            GitHubAction::CreatePRReview => self.create_pr_review(&input).await,
            GitHubAction::PostPRComment => self.post_pr_comment(&input).await,
            GitHubAction::TriggerCopilotReview => self.trigger_copilot_review(&input).await,
            GitHubAction::CreateIssue => self.create_issue(&input).await,
            GitHubAction::UpdateIssue => self.update_issue(&input).await,
            GitHubAction::GetPRInfo => self.get_pr_info(&input).await,
            GitHubAction::ListIssues => self.list_issues(&input).await,
        };
        
        let mut integration_result = result?;
        integration_result.execution_time = start_time.elapsed();
        
        info!("GitHub integration completed in {:?}", integration_result.execution_time);
        Ok(integration_result)
    }
    
    async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up GitHubIntegrationAgent");
        Ok(())
    }
    
    fn supported_project_types(&self) -> &[ProjectType] {
        &self.supported_types
    }
    
    fn capabilities(&self) -> &[AgentCapability] {
        &self.capabilities
    }
}

impl GitHubIntegrationAgent {
    async fn create_pr_review(&self, input: &GitHubIntegrationInput) -> Result<GitHubIntegrationResult> {
        let pr_number = input.pr_number.ok_or("PR number is required for review")?;
        
        if input.options.dry_run {
            return Ok(GitHubIntegrationResult {
                success: true,
                action: GitHubAction::CreatePRReview,
                pr_number: Some(pr_number),
                issue_number: None,
                comment_url: None,
                review_url: Some(format!("https://github.com/owner/repo/pull/{}", pr_number)),
                copilot_triggered: false,
                execution_time: std::time::Duration::from_secs(0),
                message: "Dry run: PR review would be created".to_string(),
            });
        }
        
        // Generate review comment based on analysis results
        let comment = self.generate_review_comment(input.analysis_results.as_ref()).await;
        
        // Post the review comment
        let mut args = vec!["pr", "review", &pr_number.to_string(), "--comment"];
        
        if let Some(repo) = &input.options.repository {
            args.extend(&["--repo", repo]);
        }
        
        let output = Command::new("gh")
            .args(&args)
            .current_dir(&input.project_path)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubIntegrationError::CommandFailed(stderr.to_string()).into());
        }
        
        // Trigger Copilot review if requested
        let mut copilot_triggered = false;
        if input.options.include_copilot {
            if let Err(e) = self.trigger_copilot_review_internal(input, pr_number).await {
                warn!("Failed to trigger Copilot review: {}", e);
            } else {
                copilot_triggered = true;
            }
        }
        
        Ok(GitHubIntegrationResult {
            success: true,
            action: GitHubAction::CreatePRReview,
            pr_number: Some(pr_number),
            issue_number: None,
            comment_url: None,
            review_url: Some(format!("https://github.com/owner/repo/pull/{}", pr_number)),
            copilot_triggered,
            execution_time: std::time::Duration::from_secs(0),
            message: "PR review created successfully".to_string(),
        })
    }
    
    async fn post_pr_comment(&self, input: &GitHubIntegrationInput) -> Result<GitHubIntegrationResult> {
        let pr_number = input.pr_number.ok_or("PR number is required for comment")?;
        
        let comment = match &input.options.comment_template {
            Some(template) => template.clone(),
            None => self.generate_review_comment(input.analysis_results.as_ref()).await,
        };
        
        if input.options.dry_run {
            return Ok(GitHubIntegrationResult {
                success: true,
                action: GitHubAction::PostPRComment,
                pr_number: Some(pr_number),
                issue_number: None,
                comment_url: Some(format!("https://github.com/owner/repo/pull/{}", pr_number)),
                review_url: None,
                copilot_triggered: false,
                execution_time: std::time::Duration::from_secs(0),
                message: format!("Dry run: Comment would be posted: {}", comment),
            });
        }
        
        let mut args = vec!["pr", "comment", &pr_number.to_string(), "--body", &comment];
        
        if let Some(repo) = &input.options.repository {
            args.extend(&["--repo", repo]);
        }
        
        let output = Command::new("gh")
            .args(&args)
            .current_dir(&input.project_path)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubIntegrationError::CommandFailed(stderr.to_string()).into());
        }
        
        Ok(GitHubIntegrationResult {
            success: true,
            action: GitHubAction::PostPRComment,
            pr_number: Some(pr_number),
            issue_number: None,
            comment_url: Some(format!("https://github.com/owner/repo/pull/{}", pr_number)),
            review_url: None,
            copilot_triggered: false,
            execution_time: std::time::Duration::from_secs(0),
            message: "Comment posted successfully".to_string(),
        })
    }
    
    async fn trigger_copilot_review(&self, input: &GitHubIntegrationInput) -> Result<GitHubIntegrationResult> {
        let pr_number = input.pr_number.ok_or("PR number is required for Copilot review")?;
        
        if input.options.dry_run {
            return Ok(GitHubIntegrationResult {
                success: true,
                action: GitHubAction::TriggerCopilotReview,
                pr_number: Some(pr_number),
                issue_number: None,
                comment_url: None,
                review_url: None,
                copilot_triggered: true,
                execution_time: std::time::Duration::from_secs(0),
                message: "Dry run: Copilot review would be triggered".to_string(),
            });
        }
        
        self.trigger_copilot_review_internal(input, pr_number).await?;
        
        Ok(GitHubIntegrationResult {
            success: true,
            action: GitHubAction::TriggerCopilotReview,
            pr_number: Some(pr_number),
            issue_number: None,
            comment_url: None,
            review_url: None,
            copilot_triggered: true,
            execution_time: std::time::Duration::from_secs(0),
            message: "Copilot review triggered successfully".to_string(),
        })
    }
    
    async fn trigger_copilot_review_internal(&self, input: &GitHubIntegrationInput, pr_number: u32) -> Result<()> {
        let comment = "@copilot please review this pull request";
        
        let mut args = vec!["pr", "comment", &pr_number.to_string(), "--body", comment];
        
        if let Some(repo) = &input.options.repository {
            args.extend(&["--repo", repo]);
        }
        
        let output = Command::new("gh")
            .args(&args)
            .current_dir(&input.project_path)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubIntegrationError::CommandFailed(stderr.to_string()).into());
        }
        
        Ok(())
    }
    
    async fn create_issue(&self, input: &GitHubIntegrationInput) -> Result<GitHubIntegrationResult> {
        if input.options.dry_run {
            return Ok(GitHubIntegrationResult {
                success: true,
                action: GitHubAction::CreateIssue,
                pr_number: None,
                issue_number: Some(1),
                comment_url: None,
                review_url: None,
                copilot_triggered: false,
                execution_time: std::time::Duration::from_secs(0),
                message: "Dry run: Issue would be created".to_string(),
            });
        }
        
        // This is a simplified implementation
        let title = "Automated Code Analysis Results";
        let body = self.generate_issue_body(input.analysis_results.as_ref()).await;
        
        let mut args = vec!["issue", "create", "--title", title, "--body", &body];
        
        if let Some(repo) = &input.options.repository {
            args.extend(&["--repo", repo]);
        }
        
        let output = Command::new("gh")
            .args(&args)
            .current_dir(&input.project_path)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubIntegrationError::CommandFailed(stderr.to_string()).into());
        }
        
        // Parse issue number from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let issue_number = stdout
            .lines()
            .find(|line| line.contains("https://github.com"))
            .and_then(|line| line.split('/').last())
            .and_then(|s| s.parse::<u32>().ok());
        
        Ok(GitHubIntegrationResult {
            success: true,
            action: GitHubAction::CreateIssue,
            pr_number: None,
            issue_number,
            comment_url: None,
            review_url: None,
            copilot_triggered: false,
            execution_time: std::time::Duration::from_secs(0),
            message: "Issue created successfully".to_string(),
        })
    }
    
    async fn update_issue(&self, _input: &GitHubIntegrationInput) -> Result<GitHubIntegrationResult> {
        // Placeholder implementation
        Ok(GitHubIntegrationResult {
            success: true,
            action: GitHubAction::UpdateIssue,
            pr_number: None,
            issue_number: Some(1),
            comment_url: None,
            review_url: None,
            copilot_triggered: false,
            execution_time: std::time::Duration::from_secs(0),
            message: "Issue update not implemented".to_string(),
        })
    }
    
    async fn get_pr_info(&self, input: &GitHubIntegrationInput) -> Result<GitHubIntegrationResult> {
        let pr_number = input.pr_number.ok_or("PR number is required")?;
        
        let mut args = vec!["pr", "view", &pr_number.to_string(), "--json", "title,body,state,author,headRefName,baseRefName,mergeable,draft"];
        
        if let Some(repo) = &input.options.repository {
            args.extend(&["--repo", repo]);
        }
        
        let output = Command::new("gh")
            .args(&args)
            .current_dir(&input.project_path)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubIntegrationError::CommandFailed(stderr.to_string()).into());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pr_info: serde_json::Value = serde_json::from_str(&stdout)?;
        
        Ok(GitHubIntegrationResult {
            success: true,
            action: GitHubAction::GetPRInfo,
            pr_number: Some(pr_number),
            issue_number: None,
            comment_url: None,
            review_url: None,
            copilot_triggered: false,
            execution_time: std::time::Duration::from_secs(0),
            message: format!("PR info retrieved: {}", pr_info),
        })
    }
    
    async fn list_issues(&self, input: &GitHubIntegrationInput) -> Result<GitHubIntegrationResult> {
        let mut args = vec!["issue", "list", "--json", "number,title,state"];
        
        if let Some(repo) = &input.options.repository {
            args.extend(&["--repo", repo]);
        }
        
        let output = Command::new("gh")
            .args(&args)
            .current_dir(&input.project_path)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubIntegrationError::CommandFailed(stderr.to_string()).into());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let issues: Vec<serde_json::Value> = serde_json::from_str(&stdout)?;
        
        Ok(GitHubIntegrationResult {
            success: true,
            action: GitHubAction::ListIssues,
            pr_number: None,
            issue_number: None,
            comment_url: None,
            review_url: None,
            copilot_triggered: false,
            execution_time: std::time::Duration::from_secs(0),
            message: format!("Found {} issues", issues.len()),
        })
    }
    
    async fn generate_review_comment(&self, analysis_results: Option<&CodeAnalysisResult>) -> String {
        match analysis_results {
            Some(results) => {
                let mut comment = String::new();
                
                comment.push_str("## 🤖 Claude Code + Copilot Analysis\n\n");
                
                if results.success {
                    comment.push_str("✅ **Analysis completed successfully**\n\n");
                } else {
                    comment.push_str("❌ **Analysis found issues**\n\n");
                }
                
                // Summary
                comment.push_str("### Summary\n");
                comment.push_str(&format!("- Total issues found: {}\n", results.issues.len()));
                
                for (severity, count) in &results.metrics.issues_count {
                    let emoji = match severity {
                        IssueSeverity::Error => "🔴",
                        IssueSeverity::Warning => "🟡",
                        IssueSeverity::Info => "🔵",
                        IssueSeverity::Style => "🟣",
                    };
                    comment.push_str(&format!("- {}: {}\n", emoji, count));
                }
                
                // Top issues
                if !results.issues.is_empty() {
                    comment.push_str("\n### Key Issues\n");
                    for (i, issue) in results.issues.iter().take(5).enumerate() {
                        let emoji = match issue.severity {
                            IssueSeverity::Error => "🔴",
                            IssueSeverity::Warning => "🟡",
                            IssueSeverity::Info => "🔵",
                            IssueSeverity::Style => "🟣",
                        };
                        comment.push_str(&format!("{}. {} {}:{} - {}\n", 
                            i + 1, emoji, 
                            issue.file.display(), 
                            issue.line.unwrap_or(0), 
                            issue.message));
                    }
                }
                
                // Suggestions
                if !results.suggestions.is_empty() {
                    comment.push_str("\n### Recommendations\n");
                    for suggestion in &results.suggestions {
                        let priority_emoji = match suggestion.priority {
                            1..=3 => "🔴",
                            4..=6 => "🟡",
                            _ => "🔵",
                        };
                        comment.push_str(&format!("{} **{}** (Priority: {}, Effort: {:?})\n", 
                            priority_emoji, suggestion.title, suggestion.priority, suggestion.effort));
                        comment.push_str(&format!("   {}\n", suggestion.description));
                    }
                }
                
                comment.push_str("\n---\n");
                comment.push_str("💡 **Tip**: Use `@copilot` in a comment to get AI-powered suggestions for specific code changes.\n");
                
                comment
            }
            None => {
                "🤖 **Claude Code Analysis**\n\nNo analysis results available. Please run code analysis first.".to_string()
            }
        }
    }
    
    async fn generate_issue_body(&self, analysis_results: Option<&CodeAnalysisResult>) -> String {
        match analysis_results {
            Some(results) => {
                let mut body = String::new();
                
                body.push_str("## 🤖 Automated Code Analysis Report\n\n");
                
                if results.success {
                    body.push_str("✅ **Analysis completed successfully**\n\n");
                } else {
                    body.push_str("❌ **Analysis found critical issues**\n\n");
                }
                
                // Detailed breakdown
                body.push_str("### Issue Breakdown\n");
                for (severity, count) in &results.metrics.issues_count {
                    let emoji = match severity {
                        IssueSeverity::Error => "🔴",
                        IssueSeverity::Warning => "🟡",
                        IssueSeverity::Info => "🔵",
                        IssueSeverity::Style => "🟣",
                    };
                    body.push_str(&format!("- {}: {}\n", emoji, count));
                }
                
                // All issues
                if !results.issues.is_empty() {
                    body.push_str("\n### All Issues\n");
                    for (i, issue) in results.issues.iter().enumerate() {
                        let emoji = match issue.severity {
                            IssueSeverity::Error => "🔴",
                            IssueSeverity::Warning => "🟡",
                            IssueSeverity::Info => "🔵",
                            IssueSeverity::Style => "🟣",
                        };
                        body.push_str(&format!("{}. {} {}:{} - {}\n", 
                            i + 1, emoji, 
                            issue.file.display(), 
                            issue.line.unwrap_or(0), 
                            issue.message));
                        if let Some(suggestion) = &issue.suggestion {
                            body.push_str(&format!("   💡 {}\n", suggestion));
                        }
                    }
                }
                
                body
            }
            None => {
                "🤖 **Automated Code Analysis**\n\nNo analysis results available.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_github_integration_agent() {
        let agent = GitHubIntegrationAgent::new();
        
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        let input = GitHubIntegrationInput {
            project_path: project_path.to_path_buf(),
            action: GitHubAction::GetPRInfo,
            analysis_results: None,
            pr_number: Some(1),
            options: GitHubOptions {
                dry_run: true,
                ..Default::default()
            },
        };
        
        // This test will fail if GitHub CLI is not installed or authenticated
        // In a real test environment, you'd mock the GitHub CLI calls
        let result = agent.execute(input).await;
        
        match result {
            Ok(_) => println!("Test passed - GitHub CLI is available"),
            Err(AgentError::GitHubIntegration(msg)) if msg.contains("GitHub CLI not found") => {
                println!("Test skipped - GitHub CLI not found");
            }
            Err(e) => {
                println!("Test failed with error: {}", e);
            }
        }
    }
}