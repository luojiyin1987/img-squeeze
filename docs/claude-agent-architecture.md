# Claude Code Agent 系统架构

## 🎯 系统概述

Claude Code Agent 是一个智能代码审查和分析系统，专为 Claude Code 设计。它通过自动化工作流集成多种代码分析工具，并与 GitHub Copilot 协作提供高质量的代码审查建议。

## 🏗️ 核心架构

### 1. 系统组件

```
Claude Code Agent System
├── Agent Core                    # 核心 Agent 引擎
│   ├── AgentRegistry            # Agent 注册中心
│   ├── AgentOrchestrator        # Agent 编排器
│   └── AgentLifecycle          # Agent 生命周期管理
├── Analysis Engine              # 分析引擎
│   ├── ProjectDetector         # 项目类型检测器
│   ├── ToolExecutor            # 工具执行器
│   └── ResultAggregator        # 结果聚合器
├── Workflow Engine             # 工作流引擎
│   ├── WorkflowDefinition      # 工作流定义
│   ├── WorkflowExecutor        # 工作流执行器
│   └── WorkflowScheduler       # 工作流调度器
├── Integration Layer           # 集成层
│   ├── ClaudeCodeAPI          # Claude Code API
│   ├── CopilotIntegration     # Copilot 集成
│   └── MCPTools               # MCP 工具接口
└── Configuration System        # 配置系统
    ├── ConfigManager          # 配置管理器
    ├── TemplateEngine         # 模板引擎
    └── SchemaValidator        # 模式验证器
```

### 2. Agent 核心设计

```rust
// 核心 Agent trait
pub trait Agent: Send + Sync {
    type Input: Serialize + Deserialize;
    type Output: Serialize + Deserialize;
    type Config: Serialize + Deserialize;
    
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<()>;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
    async fn cleanup(&self) -> Result<()>;
    
    fn supported_project_types(&self) -> &[ProjectType];
    fn capabilities(&self) -> &[AgentCapability];
}

// Agent 编排器
pub struct AgentOrchestrator {
    agents: HashMap<String, Box<dyn Agent>>,
    workflow_engine: WorkflowEngine,
    config_manager: ConfigManager,
}

impl AgentOrchestrator {
    pub async fn execute_workflow(
        &self,
        workflow: WorkflowDefinition,
        context: WorkflowContext,
    ) -> Result<WorkflowResult> {
        // 1. 解析工作流
        let parsed_workflow = self.parse_workflow(workflow).await?;
        
        // 2. 创建执行计划
        let execution_plan = self.create_execution_plan(parsed_workflow).await?;
        
        // 3. 执行工作流
        let results = self.execute_plan(execution_plan, context).await?;
        
        // 4. 聚合结果
        self.aggregate_results(results).await
    }
}
```

### 3. 项目检测器

```rust
// 项目类型检测
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

impl ProjectDetector {
    pub async fn detect_project(project_path: &Path) -> Result<ProjectInfo> {
        let mut detector = Self::new(project_path);
        
        // 检测项目类型
        let project_type = detector.detect_project_type().await?;
        
        // 检测语言
        let language = detector.detect_language().await?;
        
        // 检测框架
        let framework = detector.detect_framework().await?;
        
        // 检测构建系统
        let build_system = detector.detect_build_system().await?;
        
        // 检测包管理器
        let package_manager = detector.detect_package_manager().await?;
        
        Ok(ProjectInfo {
            project_type,
            language,
            framework,
            build_system,
            package_manager,
            root_path: project_path.to_path_buf(),
            config_files: detector.config_files,
            dependencies: detector.dependencies,
        })
    }
    
    async fn detect_project_type(&self) -> Result<ProjectType> {
        if self.path.join("Cargo.toml").exists() {
            Ok(ProjectType::Rust)
        } else if self.path.join("package.json").exists() {
            self.detect_frontend_framework().await
        } else if self.path.join("requirements.txt").exists() 
            || self.path.join("pyproject.toml").exists() {
            Ok(ProjectType::Python)
        } else if self.path.join("pom.xml").exists() 
            || self.path.join("build.gradle").exists() {
            Ok(ProjectType::Java)
        } else if self.path.join("go.mod").exists() {
            Ok(ProjectType::Go)
        } else {
            Ok(ProjectType::Generic)
        }
    }
}
```

### 4. 工作流引擎

```rust
// 工作流定义
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub version: String,
    pub description: String,
    pub triggers: Vec<WorkflowTrigger>,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, WorkflowVariable>,
    pub outputs: Vec<WorkflowOutput>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub agent: String,
    pub input: WorkflowInput,
    pub output: WorkflowOutput,
    pub dependencies: Vec<String>,
    pub retry_policy: RetryPolicy,
    pub timeout: Duration,
}

impl WorkflowEngine {
    pub async fn execute_workflow(
        &self,
        workflow: WorkflowDefinition,
        context: WorkflowContext,
    ) -> Result<WorkflowResult> {
        let mut executor = WorkflowExecutor::new(workflow, context);
        
        // 1. 验证工作流
        executor.validate().await?;
        
        // 2. 创建执行图
        let execution_graph = executor.create_execution_graph().await?;
        
        // 3. 执行工作流
        let results = executor.execute_graph(execution_graph).await?;
        
        // 4. 生成报告
        executor.generate_report(results).await
    }
}
```

### 5. MCP 工具集成

```rust
// MCP 工具接口
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub handler: ToolHandler,
}

// 工具处理器类型
type ToolHandler = Arc<dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value>> + Send + Sync>;

pub struct MCPToolRegistry {
    tools: HashMap<String, MCPTool>,
    tool_metadata: HashMap<String, ToolMetadata>,
}

impl MCPToolRegistry {
    pub fn register_claude_copilot_tools(&mut self) {
        // 注册工作流执行工具
        self.register_tool(MCPTool {
            name: "execute_claude_copilot_workflow".to_string(),
            description: "Execute Claude Code + Copilot workflow for PR review and analysis".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    },
                    "workflow_type": {
                        "type": "string",
                        "enum": ["pr_review", "full_analysis", "security_scan", "performance_analysis"],
                        "description": "Type of workflow to execute"
                    },
                    "pr_number": {
                        "type": "integer",
                        "description": "PR number for pr_review workflow"
                    }
                },
                "required": ["project_path", "workflow_type"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "success": {"type": "boolean"},
                    "report": {"type": "string"},
                    "suggestions": {"type": "array", "items": {"type": "string"}},
                    "execution_time": {"type": "number"}
                }
            }),
            handler: Arc::new(execute_workflow_handler),
        });
        
        // 注册项目检测工具
        self.register_tool(MCPTool {
            name: "detect_project_type".to_string(),
            description: "Detect project type and configuration".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    }
                },
                "required": ["project_path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_type": {"type": "string"},
                    "language": {"type": "string"},
                    "framework": {"type": "string"},
                    "build_system": {"type": "string"},
                    "package_manager": {"type": "string"}
                }
            }),
            handler: Arc::new(detect_project_handler),
        });
    }
}
```

### 6. 配置管理

```rust
// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: WorkflowConfig,
    schema: serde_json::Value,
}

impl ConfigManager {
    pub async fn load_config(&mut self, path: &Path) -> Result<WorkflowConfig> {
        let config_content = tokio::fs::read_to_string(path).await?;
        let config: WorkflowConfig = toml::from_str(&config_content)?;
        
        // 验证配置
        self.validate_config(&config).await?;
        
        // 合并默认配置
        let merged_config = self.merge_with_defaults(config);
        
        self.config = merged_config;
        Ok(self.config.clone())
    }
    
    pub async fn generate_project_config(
        &self,
        project_info: &ProjectInfo,
    ) -> Result<WorkflowConfig> {
        let template = self.load_template(project_info.project_type).await?;
        
        // 根据项目信息定制配置
        let mut config = self.customize_template(template, project_info).await?;
        
        // 验证配置
        self.validate_config(&config).await?;
        
        Ok(config)
    }
}
```

## 🔄 工作流程

### 1. 初始化流程
```
用户请求 → Agent Registry → Project Detector → Config Manager → Workflow Engine
```

### 2. 执行流程
```
Workflow Engine → Agent Orchestrator → Tool Executor → Result Aggregator → Report Generator
```

### 3. 集成流程
```
Claude Code API → MCP Tools → Agent System → GitHub Copilot → Results
```

## 🎨 使用示例

### 1. 基本使用
```rust
use claude_agent::{AgentOrchestrator, WorkflowDefinition};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建 Agent 编排器
    let orchestrator = AgentOrchestrator::new().await?;
    
    // 定义工作流
    let workflow = WorkflowDefinition {
        name: "PR Review".to_string(),
        version: "1.0".to_string(),
        description: "Comprehensive PR review workflow".to_string(),
        triggers: vec![WorkflowTrigger::PullRequest],
        steps: vec![
            // 项目检测
            WorkflowStep {
                name: "detect_project".to_string(),
                agent: "project_detector".to_string(),
                input: WorkflowInput::ProjectPath(PathBuf::from(".")),
                output: WorkflowOutput::ProjectInfo,
                dependencies: vec![],
                retry_policy: RetryPolicy::default(),
                timeout: Duration::from_secs(30),
            },
            // 代码分析
            WorkflowStep {
                name: "analyze_code".to_string(),
                agent: "analysis_engine".to_string(),
                input: WorkflowInput::ProjectInfo,
                output: WorkflowOutput::AnalysisResults,
                dependencies: vec!["detect_project".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: Duration::from_secs(300),
            },
            // Copilot 集成
            WorkflowStep {
                name: "copilot_review".to_string(),
                agent: "copilot_integration".to_string(),
                input: WorkflowInput::AnalysisResults,
                output: WorkflowOutput::CopilotResponse,
                dependencies: vec!["analyze_code".to_string()],
                retry_policy: RetryPolicy::default(),
                timeout: Duration::from_secs(600),
            },
        ],
        variables: HashMap::new(),
        outputs: vec![WorkflowOutput::CopilotResponse],
    };
    
    // 执行工作流
    let result = orchestrator.execute_workflow(workflow, context).await?;
    
    println!("Workflow completed: {:?}", result);
    Ok(())
}
```

### 2. MCP 工具使用
```rust
// 通过 MCP 工具调用
let tool_result = mcp_tool_registry.execute_tool(
    "execute_claude_copilot_workflow",
    serde_json::json!({
        "project_path": "/path/to/project",
        "workflow_type": "pr_review",
        "pr_number": 123
    }),
).await?;
```

## 🚀 优势

1. **模块化设计**：每个 Agent 都是独立的，可以单独开发和测试
2. **可扩展性**：支持新的项目类型和分析工具
3. **异步执行**：基于 Tokio 的高性能异步处理
4. **配置驱动**：通过配置文件灵活控制工作流
5. **错误恢复**：完善的错误处理和重试机制
6. **监控和日志**：完整的执行过程监控和日志记录

这个架构设计提供了一个强大、灵活且可扩展的 Claude Code Agent 系统，可以适应各种项目的代码审查和分析需求。