// Claude Code Agent System
// 智能代码审查和分析系统

pub mod core;
pub mod project_detector;
pub mod workflow_engine;
pub mod mcp_tools;
pub mod config;
pub mod agents;
pub mod github;

pub use core::*;
pub use project_detector::ProjectDetector;
pub use workflow_engine::WorkflowEngine;
pub use mcp_tools::{MCPToolRegistry, MCPTool};
pub use config::{ConfigManager, GlobalConfig, AgentConfig, WorkflowConfig};
pub use agents::{CodeAnalysisAgent, CodeAnalysisInput, CodeAnalysisResult, AnalysisType, AnalysisOptions};
pub use github::{GitHubIntegrationAgent, GitHubIntegrationInput, GitHubAction, GitHubOptions, GitHubIntegrationResult};

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Claude Code Agent 系统主结构
pub struct ClaudeAgent {
    registry: AgentRegistry,
    workflow_engine: Arc<RwLock<WorkflowEngine>>,
    config_manager: Arc<RwLock<ConfigManager>>,
    mcp_tool_registry: Arc<MCPToolRegistry>,
    lifecycle_manager: AgentLifecycleManager,
}

impl ClaudeAgent {
    /// 创建新的 Claude Agent 实例
    pub async fn new<P: AsRef<std::path::Path>>(config_path: P) -> Result<Self> {
        tracing_subscriber::fmt::init();
        
        info!("Initializing Claude Agent system");
        
        let config_manager = Arc::new(RwLock::new(ConfigManager::new(config_path)));
        let registry = AgentRegistry::new();
        let workflow_engine = Arc::new(RwLock::new(WorkflowEngine::new(
            registry.clone(),
            config_manager.clone(),
        )));
        let mcp_tool_registry = Arc::new(MCPToolRegistry::new(
            workflow_engine.clone(),
            ConfigManager::new(config_path), // Create a separate instance for MCP
        ));
        
        let mut agent = Self {
            registry,
            workflow_engine,
            config_manager,
            mcp_tool_registry,
            lifecycle_manager: AgentLifecycleManager::new(),
        };
        
        // 注册内置 Agents
        agent.register_builtin_agents().await?;
        
        info!("Claude Agent system initialized successfully");
        Ok(agent)
    }
    
    /// 注册内置 Agents
    async fn register_builtin_agents(&mut self) -> Result<()> {
        info!("Registering builtin agents");
        
        // 注册代码分析 Agent
        let code_analysis_agent = CodeAnalysisAgent::new();
        self.registry.register_agent(code_analysis_agent);
        
        // 注册 GitHub 集成 Agent
        let github_agent = GitHubIntegrationAgent::new();
        self.registry.register_agent(github_agent);
        
        info!("Registered {} builtin agents", self.registry.list_agents().len());
        Ok(())
    }
    
    /// 执行工作流
    pub async fn execute_workflow(
        &self,
        workflow: WorkflowDefinition,
        context: WorkflowContext,
    ) -> Result<WorkflowResult> {
        info!("Executing workflow: {}", workflow.name);
        
        let engine = self.workflow_engine.read().await;
        let result = engine.execute_workflow(workflow, context).await?;
        
        info!("Workflow execution completed: {:?}", result.success);
        Ok(result)
    }
    
    /// 执行 PR 审查工作流
    pub async fn execute_pr_review_workflow<P: AsRef<std::path::Path>>(
        &self,
        project_path: P,
        pr_number: u32,
    ) -> Result<WorkflowResult> {
        let workflow = create_pr_review_workflow(pr_number)?;
        let context = WorkflowContext {
            project_path: project_path.as_ref().to_path_buf(),
            environment: Environment::Development,
            variables: {
                let mut vars = std::collections::HashMap::new();
                vars.insert("pr_number".to_string(), serde_json::Value::Number(serde_json::Number::from(pr_number)));
                vars
            },
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("workflow_type".to_string(), "pr_review".to_string());
                meta
            },
        };
        
        self.execute_workflow(workflow, context).await
    }
    
    /// 执行完整分析工作流
    pub async fn execute_full_analysis_workflow<P: AsRef<std::path::Path>>(
        &self,
        project_path: P,
    ) -> Result<WorkflowResult> {
        let workflow = create_full_analysis_workflow()?;
        let context = WorkflowContext {
            project_path: project_path.as_ref().to_path_buf(),
            environment: Environment::Development,
            variables: std::collections::HashMap::new(),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("workflow_type".to_string(), "full_analysis".to_string());
                meta
            },
        };
        
        self.execute_workflow(workflow, context).await
    }
    
    /// 执行安全扫描工作流
    pub async fn execute_security_scan_workflow<P: AsRef<std::path::Path>>(
        &self,
        project_path: P,
    ) -> Result<WorkflowResult> {
        let workflow = create_security_scan_workflow()?;
        let context = WorkflowContext {
            project_path: project_path.as_ref().to_path_buf(),
            environment: Environment::Development,
            variables: std::collections::HashMap::new(),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("workflow_type".to_string(), "security_scan".to_string());
                meta
            },
        };
        
        self.execute_workflow(workflow, context).await
    }
    
    /// 执行性能分析工作流
    pub async fn execute_performance_analysis_workflow<P: AsRef<std::path::Path>>(
        &self,
        project_path: P,
    ) -> Result<WorkflowResult> {
        let workflow = create_performance_analysis_workflow()?;
        let context = WorkflowContext {
            project_path: project_path.as_ref().to_path_buf(),
            environment: Environment::Development,
            variables: std::collections::HashMap::new(),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("workflow_type".to_string(), "performance_analysis".to_string());
                meta
            },
        };
        
        self.execute_workflow(workflow, context).await
    }
    
    /// 执行 MCP 工具
    pub async fn execute_mcp_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        info!("Executing MCP tool: {}", tool_name);
        
        let result = self.mcp_tool_registry.execute_tool(tool_name, input).await?;
        
        info!("MCP tool execution completed: {}", tool_name);
        Ok(result)
    }
    
    /// 获取可用的 MCP 工具列表
    pub fn get_available_tools(&self) -> Vec<&str> {
        self.mcp_tool_registry.list_tools()
    }
    
    /// 获取工具模式
    pub fn get_tool_schemas(&self) -> std::collections::HashMap<String, (serde_json::Value, serde_json::Value)> {
        self.mcp_tool_registry.get_tool_schemas()
    }
    
    /// 检测项目类型
    pub async fn detect_project<P: AsRef<std::path::Path>>(
        &self,
        project_path: P,
    ) -> Result<ProjectInfo> {
        ProjectDetector::detect_project(project_path).await
    }
    
    /// 获取执行历史
    pub async fn get_execution_history(&self) -> Vec<workflow_engine::WorkflowExecution> {
        let engine = self.workflow_engine.read().await;
        engine.get_execution_history().await
    }
    
    /// 获取特定的执行记录
    pub async fn get_execution(
        &self,
        execution_id: uuid::Uuid,
    ) -> Option<workflow_engine::WorkflowExecution> {
        let engine = self.workflow_engine.read().await;
        engine.get_execution(execution_id).await
    }
    
    /// 生成项目配置
    pub async fn generate_project_config<P: AsRef<std::path::Path>>(
        &self,
        project_path: P,
        config_type: &str,
    ) -> Result<serde_json::Value> {
        let project_info = self.detect_project(project_path).await?;
        generate_project_config_for_type(&project_info, config_type).await
    }
}

/// 创建 PR 审查工作流
fn create_pr_review_workflow(pr_number: u32) -> Result<WorkflowDefinition> {
    Ok(WorkflowDefinition {
        name: format!("PR Review Workflow - PR #{}", pr_number),
        version: "1.0".to_string(),
        description: "Comprehensive PR review workflow with Claude Code + Copilot integration".to_string(),
        triggers: vec![WorkflowTrigger::PullRequest],
        steps: vec![
            WorkflowStep {
                name: "detect_project".to_string(),
                agent: "project_detector".to_string(),
                input: WorkflowInput::ProjectPath(std::path::PathBuf::from(".")),
                output: WorkflowOutput::ProjectInfo,
                dependencies: vec![],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(30),
            },
            WorkflowStep {
                name: "analyze_code_quality".to_string(),
                agent: "analysis_engine".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(300),
            },
            WorkflowStep {
                name: "security_scan".to_string(),
                agent: "security_scanner".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(180),
            },
            WorkflowStep {
                name: "generate_copilot_prompt".to_string(),
                agent: "prompt_generator".to_string(),
                input: WorkflowInput::Custom(serde_json::json!({
                    "type": "pr_review",
                    "pr_number": pr_number
                })),
                output: WorkflowOutput::CopilotPrompt,
                dependencies: vec!["analyze_code_quality".to_string(), "security_scan".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(60),
            },
            WorkflowStep {
                name: "copilot_review".to_string(),
                agent: "copilot_integration".to_string(),
                input: WorkflowInput::CopilotPrompt,
                output: WorkflowOutput::CopilotResponse,
                dependencies: vec!["generate_copilot_prompt".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(600),
            },
            WorkflowStep {
                name: "generate_report".to_string(),
                agent: "report_generator".to_string(),
                input: WorkflowInput::Custom(serde_json::json!({
                    "type": "pr_review",
                    "include_analysis": true,
                    "include_copilot": true
                })),
                output: WorkflowOutput::Report,
                dependencies: vec!["copilot_review".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(120),
            },
        ],
        variables: std::collections::HashMap::new(),
        outputs: vec![WorkflowOutput::Report],
    })
}

/// 创建完整分析工作流
fn create_full_analysis_workflow() -> Result<WorkflowDefinition> {
    Ok(WorkflowDefinition {
        name: "Full Analysis Workflow".to_string(),
        version: "1.0".to_string(),
        description: "Comprehensive code analysis including quality, security, and performance".to_string(),
        triggers: vec![WorkflowTrigger::Manual],
        steps: vec![
            WorkflowStep {
                name: "detect_project".to_string(),
                agent: "project_detector".to_string(),
                input: WorkflowInput::ProjectPath(std::path::PathBuf::from(".")),
                output: WorkflowOutput::ProjectInfo,
                dependencies: vec![],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(30),
            },
            WorkflowStep {
                name: "analyze_code_quality".to_string(),
                agent: "analysis_engine".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(300),
            },
            WorkflowStep {
                name: "security_scan".to_string(),
                agent: "security_scanner".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(180),
            },
            WorkflowStep {
                name: "performance_analysis".to_string(),
                agent: "performance_analyzer".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(240),
            },
            WorkflowStep {
                name: "test_coverage_analysis".to_string(),
                agent: "test_coverage_analyzer".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(180),
            },
            WorkflowStep {
                name: "generate_report".to_string(),
                agent: "report_generator".to_string(),
                input: WorkflowInput::Custom(serde_json::json!({
                    "type": "full_analysis",
                    "include_quality": true,
                    "include_security": true,
                    "include_performance": true,
                    "include_coverage": true
                })),
                output: WorkflowOutput::Report,
                dependencies: vec!["analyze_code_quality".to_string(), "security_scan".to_string(), "performance_analysis".to_string(), "test_coverage_analysis".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(120),
            },
        ],
        variables: std::collections::HashMap::new(),
        outputs: vec![WorkflowOutput::Report],
    })
}

/// 创建安全扫描工作流
fn create_security_scan_workflow() -> Result<WorkflowDefinition> {
    Ok(WorkflowDefinition {
        name: "Security Scan Workflow".to_string(),
        version: "1.0".to_string(),
        description: "Focused security vulnerability scanning and analysis".to_string(),
        triggers: vec![WorkflowTrigger::Manual],
        steps: vec![
            WorkflowStep {
                name: "detect_project".to_string(),
                agent: "project_detector".to_string(),
                input: WorkflowInput::ProjectPath(std::path::PathBuf::from(".")),
                output: WorkflowOutput::ProjectInfo,
                dependencies: vec![],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(30),
            },
            WorkflowStep {
                name: "dependency_vulnerability_scan".to_string(),
                agent: "dependency_scanner".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(120),
            },
            WorkflowStep {
                name: "code_security_scan".to_string(),
                agent: "code_scanner".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(180),
            },
            WorkflowStep {
                name: "generate_report".to_string(),
                agent: "report_generator".to_string(),
                input: WorkflowInput::Custom(serde_json::json!({
                    "type": "security_scan",
                    "include_dependencies": true,
                    "include_code_analysis": true
                })),
                output: WorkflowOutput::Report,
                dependencies: vec!["dependency_vulnerability_scan".to_string(), "code_security_scan".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(120),
            },
        ],
        variables: std::collections::HashMap::new(),
        outputs: vec![WorkflowOutput::Report],
    })
}

/// 创建性能分析工作流
fn create_performance_analysis_workflow() -> Result<WorkflowDefinition> {
    Ok(WorkflowDefinition {
        name: "Performance Analysis Workflow".to_string(),
        version: "1.0".to_string(),
        description: "Comprehensive performance analysis and optimization recommendations".to_string(),
        triggers: vec![WorkflowTrigger::Manual],
        steps: vec![
            WorkflowStep {
                name: "detect_project".to_string(),
                agent: "project_detector".to_string(),
                input: WorkflowInput::ProjectPath(std::path::PathBuf::from(".")),
                output: WorkflowOutput::ProjectInfo,
                dependencies: vec![],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(30),
            },
            WorkflowStep {
                name: "build_performance_analysis".to_string(),
                agent: "build_analyzer".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(180),
            },
            WorkflowStep {
                name: "runtime_performance_analysis".to_string(),
                agent: "runtime_analyzer".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(240),
            },
            WorkflowStep {
                name: "memory_usage_analysis".to_string(),
                agent: "memory_analyzer".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(120),
            },
            WorkflowStep {
                name: "generate_report".to_string(),
                agent: "report_generator".to_string(),
                input: WorkflowInput::Custom(serde_json::json!({
                    "type": "performance_analysis",
                    "include_build": true,
                    "include_runtime": true,
                    "include_memory": true
                })),
                output: WorkflowOutput::Report,
                dependencies: vec!["build_performance_analysis".to_string(), "runtime_performance_analysis".to_string(), "memory_usage_analysis".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: std::time::Duration::from_secs(120),
            },
        ],
        variables: std::collections::HashMap::new(),
        outputs: vec![WorkflowOutput::Report],
    })
}

/// 生成项目配置
async fn generate_project_config_for_type(
    project_info: &ProjectInfo,
    config_type: &str,
) -> Result<serde_json::Value> {
    match config_type {
        "minimal" => {
            Ok(serde_json::json!({
                "version": "1.0",
                "project": {
                    "name": project_info.root_path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown"),
                    "type": format!("{:?}", project_info.project_type),
                    "language": project_info.language
                },
                "analysis": {
                    "basics": {
                        "code_quality": true,
                        "security_scan": true
                    }
                }
            }))
        }
        "standard" => {
            Ok(serde_json::json!({
                "version": "1.0",
                "project": {
                    "name": project_info.root_path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown"),
                    "type": format!("{:?}", project_info.project_type),
                    "language": project_info.language,
                    "framework": project_info.framework,
                    "build_system": project_info.build_system
                },
                "analysis": {
                    "basics": {
                        "code_quality": true,
                        "security_scan": true,
                        "dependency_check": true,
                        "test_coverage": true
                    },
                    "advanced": {
                        "performance_analysis": true,
                        "architecture_review": true
                    }
                }
            }))
        }
        "comprehensive" => {
            Ok(serde_json::json!({
                "version": "1.0",
                "project": {
                    "name": project_info.root_path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown"),
                    "type": format!("{:?}", project_info.project_type),
                    "language": project_info.language,
                    "framework": project_info.framework,
                    "build_system": project_info.build_system,
                    "package_manager": project_info.package_manager
                },
                "analysis": {
                    "basics": {
                        "code_quality": true,
                        "security_scan": true,
                        "dependency_check": true,
                        "test_coverage": true,
                        "documentation_check": true
                    },
                    "advanced": {
                        "performance_analysis": true,
                        "architecture_review": true,
                        "best_practices": true,
                        "accessibility_check": false
                    }
                }
            }))
        }
        _ => {
            Err(AgentError::Configuration(format!("Unknown config type: {}", config_type)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_claude_agent_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // 创建一个简单的配置文件
        std::fs::write(&config_path, r#"
version = "1.0"
[project]
name = "test"
type = "generic"
language = "unknown"
"#).unwrap();
        
        let agent = ClaudeAgent::new(&config_path).await;
        assert!(agent.is_ok());
    }
    
    #[tokio::test]
    async fn test_workflow_creation() {
        let workflow = create_pr_review_workflow(123);
        assert!(workflow.is_ok());
        
        let workflow = workflow.unwrap();
        assert_eq!(workflow.name, "PR Review Workflow - PR #123");
        assert_eq!(workflow.steps.len(), 6);
    }
    
    #[tokio::test]
    async fn test_full_analysis_workflow() {
        let workflow = create_full_analysis_workflow();
        assert!(workflow.is_ok());
        
        let workflow = workflow.unwrap();
        assert_eq!(workflow.name, "Full Analysis Workflow");
        assert_eq!(workflow.steps.len(), 6);
    }
    
    #[tokio::test]
    async fn test_security_scan_workflow() {
        let workflow = create_security_scan_workflow();
        assert!(workflow.is_ok());
        
        let workflow = workflow.unwrap();
        assert_eq!(workflow.name, "Security Scan Workflow");
        assert_eq!(workflow.steps.len(), 4);
    }
    
    #[tokio::test]
    async fn test_performance_analysis_workflow() {
        let workflow = create_performance_analysis_workflow();
        assert!(workflow.is_ok());
        
        let workflow = workflow.unwrap();
        assert_eq!(workflow.name, "Performance Analysis Workflow");
        assert_eq!(workflow.steps.len(), 5);
    }
    
    #[tokio::test]
    async fn test_project_config_generation() {
        let project_info = ProjectInfo {
            project_type: ProjectType::Rust,
            language: "rust".to_string(),
            framework: None,
            build_system: Some("cargo".to_string()),
            package_manager: Some("cargo".to_string()),
            root_path: std::path::PathBuf::from("/test/project"),
            config_files: vec![std::path::PathBuf::from("Cargo.toml")],
            dependencies: vec![],
        };
        
        let config = generate_project_config_for_type(&project_info, "standard").await;
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config["project"]["type"], "Rust");
        assert_eq!(config["project"]["language"], "rust");
        assert_eq!(config["project"]["build_system"], "cargo");
    }
}