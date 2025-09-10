use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;

// 核心类型定义

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    Nodejs,
    Python,
    Java,
    Go,
    React,
    Vue,
    Angular,
    Terraform,
    Docker,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub project_type: ProjectType,
    pub language: String,
    pub framework: Option<String>,
    pub build_system: Option<String>,
    pub package_manager: Option<String>,
    pub root_path: PathBuf,
    pub config_files: Vec<PathBuf>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    Cargo,
    Npm,
    Pip,
    Maven,
    Gradle,
    GoMod,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentCapability {
    ProjectDetection,
    CodeAnalysis,
    SecurityScan,
    PerformanceAnalysis,
    TestCoverage,
    DocumentationGeneration,
    CopilotIntegration,
    ReportGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowTrigger {
    PullRequest,
    Push,
    Manual,
    Schedule,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub agent: String,
    pub input: WorkflowInput,
    pub output: WorkflowOutput,
    pub dependencies: Vec<String>,
    pub retry_policy: RetryPolicy,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowInput {
    ProjectPath(PathBuf),
    ProjectInfo,
    AnalysisResults,
    CopilotPrompt,
    Custom(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowOutput {
    ProjectPath(PathBuf),
    ProjectInfo,
    AnalysisResults,
    CopilotPrompt,
    CopilotResponse,
    Report,
    Custom(serde_json::Value),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_strategy: BackoffStrategy::Exponential,
            max_delay: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed,
    Linear,
    Exponential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub version: String,
    pub description: String,
    pub triggers: Vec<WorkflowTrigger>,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, WorkflowVariable>,
    pub outputs: Vec<WorkflowOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVariable {
    pub name: String,
    pub value: serde_json::Value,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub project_path: PathBuf,
    pub environment: Environment,
    pub variables: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub outputs: HashMap<String, serde_json::Value>,
    pub execution_time: Duration,
    pub error: Option<String>,
    pub logs: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

// 核心 Agent trait
#[async_trait]
pub trait Agent: Send + Sync {
    type Input: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    type Output: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    type Config: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<()>;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
    async fn cleanup(&self) -> Result<()>;
    
    fn supported_project_types(&self) -> &[ProjectType];
    fn capabilities(&self) -> &[AgentCapability];
}

// 错误类型
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Unsupported project type: {0:?}")]
    UnsupportedProjectType(ProjectType),
    
    #[error("Missing required capability: {0}")]
    MissingCapability(String),
    
    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("GitHub integration error: {0}")]
    GitHubIntegration(String),
    
    #[error("Other error: {0}")]
    Other(String),
    
    #[error("Code analysis error: {0}")]
    CodeAnalysis(String),
}

pub type Result<T> = std::result::Result<T, AgentError>;

// Agent 注册器
pub struct AgentRegistry {
    agents: HashMap<String, Box<dyn AgentDispatcher>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }
    
    pub fn register_agent<A>(&mut self, agent: A)
    where
        A: Agent + 'static,
        A::Input: Send + Sync + 'static,
        A::Output: Send + Sync + 'static,
        A::Config: Send + Sync + 'static,
    {
        let dispatcher = Box::new(AgentDispatcherImpl::<A>::new(agent));
        self.agents.insert(dispatcher.name().to_string(), dispatcher);
    }
    
    pub fn get_agent(&self, name: &str) -> Option<&dyn AgentDispatcher> {
        self.agents.get(name).map(|agent| agent.as_ref())
    }
    
    pub fn list_agents(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }
    
    pub fn find_agents_by_capability(&self, capability: &AgentCapability) -> Vec<&dyn AgentDispatcher> {
        self.agents
            .values()
            .filter(|agent| agent.capabilities().contains(capability))
            .map(|agent| agent.as_ref())
            .collect()
    }
    
    pub fn find_agents_by_project_type(&self, project_type: &ProjectType) -> Vec<&dyn AgentDispatcher> {
        self.agents
            .values()
            .filter(|agent| agent.supported_project_types().contains(project_type))
            .map(|agent| agent.as_ref())
            .collect()
    }
}

// Agent 调度器 trait
#[async_trait]
pub trait AgentDispatcher: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn supported_project_types(&self) -> &[ProjectType];
    fn capabilities(&self) -> &[AgentCapability];
    
    async fn initialize(&mut self, config: serde_json::Value) -> Result<()>;
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value>;
    async fn cleanup(&self) -> Result<()>;
}

// Agent 调度器实现
struct AgentDispatcherImpl<A>
where
    A: Agent + 'static,
    A::Input: Send + Sync + 'static,
    A::Output: Send + Sync + 'static,
    A::Config: Send + Sync + 'static,
{
    agent: Option<A>,
    initialized: bool,
}

impl<A> AgentDispatcherImpl<A>
where
    A: Agent + 'static,
    A::Input: Send + Sync + 'static,
    A::Output: Send + Sync + 'static,
    A::Config: Send + Sync + 'static,
{
    fn new(agent: A) -> Self {
        Self {
            agent: Some(agent),
            initialized: false,
        }
    }
}

#[async_trait]
impl<A> AgentDispatcher for AgentDispatcherImpl<A>
where
    A: Agent + 'static,
    A::Input: Send + Sync + 'static,
    A::Output: Send + Sync + 'static,
    A::Config: Send + Sync + 'static,
{
    fn name(&self) -> &str {
        self.agent.as_ref().unwrap().name()
    }
    
    fn version(&self) -> &str {
        self.agent.as_ref().unwrap().version()
    }
    
    fn description(&self) -> &str {
        self.agent.as_ref().unwrap().description()
    }
    
    fn supported_project_types(&self) -> &[ProjectType] {
        self.agent.as_ref().unwrap().supported_project_types()
    }
    
    fn capabilities(&self) -> &[AgentCapability] {
        self.agent.as_ref().unwrap().capabilities()
    }
    
    async fn initialize(&mut self, config: serde_json::Value) -> Result<()> {
        if !self.initialized {
            let config: A::Config = serde_json::from_value(config)
                .map_err(|e| AgentError::Configuration(e.to_string()))?;
            
            self.agent.as_mut().unwrap().initialize(config).await?;
            self.initialized = true;
        }
        Ok(())
    }
    
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        if !self.initialized {
            return Err(AgentError::Configuration("Agent not initialized".to_string()));
        }
        
        let input: A::Input = serde_json::from_value(input)
            .map_err(|e| AgentError::Configuration(e.to_string()))?;
        
        let output = self.agent.as_ref().unwrap().execute(input).await?;
        
        serde_json::to_value(output)
            .map_err(|e| AgentError::Serialization(e))
    }
    
    async fn cleanup(&self) -> Result<()> {
        if self.initialized {
            self.agent.as_ref().unwrap().cleanup().await?;
        }
        Ok(())
    }
}

// 生命周期管理
pub struct AgentLifecycleManager {
    agents: Vec<Box<dyn AgentDispatcher>>,
}

impl AgentLifecycleManager {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
        }
    }
    
    pub fn add_agent(&mut self, agent: Box<dyn AgentDispatcher>) {
        self.agents.push(agent);
    }
    
    pub async fn initialize_all(&mut self, configs: HashMap<String, serde_json::Value>) -> Result<()> {
        for agent in &mut self.agents {
            let name = agent.name();
            if let Some(config) = configs.get(name) {
                agent.initialize(config.clone()).await?;
            }
        }
        Ok(())
    }
    
    pub async fn cleanup_all(&mut self) -> Result<()> {
        for agent in &self.agents {
            agent.cleanup().await?;
        }
        Ok(())
    }
}

impl Drop for AgentLifecycleManager {
    fn drop(&mut self) {
        // 在 drop 时清理资源
        if !self.agents.is_empty() {
            let agents = std::mem::take(&mut self.agents);
            tokio::spawn(async move {
                let mut manager = AgentLifecycleManager { agents };
                if let Err(e) = manager.cleanup_all().await {
                    eprintln!("Error during cleanup: {}", e);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_registry() {
        let mut registry = AgentRegistry::new();
        assert_eq!(registry.list_agents().len(), 0);
        
        // 这里可以添加测试 Agent
        // registry.register_agent(test_agent);
        
        assert_eq!(registry.list_agents().len(), 0);
    }
    
    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert!(matches!(policy.backoff_strategy, BackoffStrategy::Exponential));
        assert_eq!(policy.max_delay, Duration::from_secs(60));
    }
    
    #[test]
    fn test_project_info_creation() {
        let project_info = ProjectInfo {
            project_type: ProjectType::Rust,
            language: "rust".to_string(),
            framework: None,
            build_system: Some("cargo".to_string()),
            package_manager: Some("cargo".to_string()),
            root_path: PathBuf::from("/test"),
            config_files: vec![PathBuf::from("Cargo.toml")],
            dependencies: vec![],
        };
        
        assert!(matches!(project_info.project_type, ProjectType::Rust));
        assert_eq!(project_info.language, "rust");
    }
}