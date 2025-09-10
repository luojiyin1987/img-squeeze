use crate::core::*;
use crate::project_detector::ProjectDetector;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use std::path::{Path, PathBuf};
use tokio::sync::{RwLock, mpsc};
use tokio::task::JoinHandle;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

// 工作流引擎
pub struct WorkflowEngine {
    registry: AgentRegistry,
    config_manager: ConfigManager,
    execution_history: RwLock<Vec<WorkflowExecution>>,
}

impl WorkflowEngine {
    pub fn new(registry: AgentRegistry, config_manager: ConfigManager) -> Self {
        Self {
            registry,
            config_manager,
            execution_history: RwLock::new(Vec::new()),
        }
    }
    
    pub async fn execute_workflow(
        &self,
        workflow: WorkflowDefinition,
        context: WorkflowContext,
    ) -> Result<WorkflowResult> {
        let execution_id = Uuid::new_v4();
        let start_time = Instant::now();
        
        info!("Starting workflow execution: {} (ID: {})", workflow.name, execution_id);
        
        // 创建执行记录
        let execution = WorkflowExecution {
            id: execution_id,
            workflow_name: workflow.name.clone(),
            start_time: chrono::Utc::now(),
            end_time: None,
            execution_time: None,
            status: ExecutionStatus::Running,
            context: context.clone(),
            step_results: HashMap::new(),
            logs: Vec::new(),
        };
        
        {
            let mut history = self.execution_history.write().await;
            history.push(execution);
        }
        
        // 执行工作流
        let result = self.execute_workflow_internal(workflow, context, execution_id).await;
        
        let execution_time = start_time.elapsed();
        
        // 更新执行记录
        {
            let mut history = self.execution_history.write().await;
            if let Some(execution) = history.iter_mut().find(|e| e.id == execution_id) {
                execution.end_time = Some(chrono::Utc::now());
                execution.execution_time = Some(execution_time);
                execution.status = match &result {
                    Ok(_) => ExecutionStatus::Completed,
                    Err(_) => ExecutionStatus::Failed,
                };
            }
        }
        
        result
    }
    
    async fn execute_workflow_internal(
        &self,
        workflow: WorkflowDefinition,
        context: WorkflowContext,
        execution_id: Uuid,
    ) -> Result<WorkflowResult> {
        // 1. 验证工作流
        self.validate_workflow(&workflow).await?;
        
        // 2. 创建执行图
        let execution_graph = self.create_execution_graph(&workflow).await?;
        
        // 3. 执行工作流
        let step_results = self.execute_execution_graph(execution_graph, &workflow, &context, execution_id).await?;
        
        // 4. 生成输出
        let outputs = self.generate_outputs(&workflow, &step_results).await?;
        
        let execution_time = {
            let history = self.execution_history.read().await;
            history
                .iter()
                .find(|e| e.id == execution_id)
                .and_then(|e| e.execution_time)
                .unwrap_or(Duration::from_secs(0))
        };
        
        let logs = {
            let history = self.execution_history.read().await;
            history
                .iter()
                .find(|e| e.id == execution_id)
                .map(|e| e.logs.clone())
                .unwrap_or_default()
        };
        
        Ok(WorkflowResult {
            success: true,
            outputs,
            execution_time,
            error: None,
            logs,
        })
    }
    
    async fn validate_workflow(&self, workflow: &WorkflowDefinition) -> Result<()> {
        // 检查工作流名称
        if workflow.name.is_empty() {
            return Err(AgentError::Configuration("Workflow name cannot be empty".to_string()));
        }
        
        // 检查步骤名称唯一性
        let step_names: HashSet<&str> = workflow.steps.iter().map(|s| s.name.as_str()).collect();
        if step_names.len() != workflow.steps.len() {
            return Err(AgentError::Configuration("Duplicate step names found".to_string()));
        }
        
        // 检查依赖的有效性
        for step in &workflow.steps {
            for dep in &step.dependencies {
                if !step_names.contains(dep.as_str()) {
                    return Err(AgentError::Configuration(format!(
                        "Step '{}' depends on non-existent step '{}'",
                        step.name, dep
                    )));
                }
            }
        }
        
        // 检查 Agent 是否存在
        for step in &workflow.steps {
            if self.registry.get_agent(&step.agent).is_none() {
                return Err(AgentError::Configuration(format!(
                    "Agent '{}' not found for step '{}'",
                    step.agent, step.name
                )));
            }
        }
        
        // 检查循环依赖
        if self.has_circular_dependencies(workflow)? {
            return Err(AgentError::Configuration("Circular dependencies detected".to_string()));
        }
        
        Ok(())
    }
    
    fn has_circular_dependencies(&self, workflow: &WorkflowDefinition) -> Result<bool> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        
        for step in &workflow.steps {
            if self.has_circular_dependencies_helper(step, workflow, &mut visited, &mut recursion_stack)? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn has_circular_dependencies_helper(
        &self,
        step: &WorkflowStep,
        workflow: &WorkflowDefinition,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> Result<bool> {
        let step_name = &step.name;
        
        if visited.contains(step_name) {
            return Ok(false);
        }
        
        visited.insert(step_name.clone());
        recursion_stack.insert(step_name.clone());
        
        for dep in &step.dependencies {
            if let Some(dep_step) = workflow.steps.iter().find(|s| s.name == *dep) {
                if recursion_stack.contains(dep) {
                    return Ok(true);
                }
                
                if self.has_circular_dependencies_helper(dep_step, workflow, visited, recursion_stack)? {
                    return Ok(true);
                }
            }
        }
        
        recursion_stack.remove(step_name);
        Ok(false)
    }
    
    async fn create_execution_graph(&self, workflow: &WorkflowDefinition) -> Result<ExecutionGraph> {
        let mut graph = HashMap::new();
        
        for step in &workflow.steps {
            let node = ExecutionNode {
                step: step.clone(),
                status: NodeStatus::Pending,
                dependencies: step.dependencies.clone(),
                dependents: Vec::new(),
                result: None,
                start_time: None,
                end_time: None,
            };
            graph.insert(step.name.clone(), node);
        }
        
        // 构建依赖关系（反向）
        for (step_name, node) in &graph {
            for dep in &node.dependencies {
                if let Some(dep_node) = graph.get_mut(dep) {
                    dep_node.dependents.push(step_name.clone());
                }
            }
        }
        
        Ok(ExecutionGraph { nodes: graph })
    }
    
    async fn execute_execution_graph(
        &self,
        mut graph: ExecutionGraph,
        workflow: &WorkflowDefinition,
        context: &WorkflowContext,
        execution_id: Uuid,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut ready_steps = VecDeque::new();
        let mut step_results = HashMap::new();
        let mut running_steps = HashMap::new();
        let mut completed_steps = HashSet::new();
        
        // 初始化：找到没有依赖的步骤
        for (step_name, node) in &graph.nodes {
            if node.dependencies.is_empty() {
                ready_steps.push_back(step_name.clone());
            }
        }
        
        // 执行步骤
        while !ready_steps.is_empty() || !running_steps.is_empty() {
            // 启动就绪的步骤
            while let Some(step_name) = ready_steps.pop_front() {
                if let Some(node) = graph.nodes.get_mut(&step_name) {
                    let step = node.step.clone();
                    let context = context.clone();
                    let execution_id = execution_id;
                    
                    let handle = tokio::spawn(async move {
                        self.execute_step(step, context, execution_id).await
                    });
                    
                    running_steps.insert(step_name.clone(), handle);
                    node.status = NodeStatus::Running;
                    node.start_time = Some(chrono::Utc::now());
                    
                    self.log_execution_event(execution_id, LogLevel::Info, format!("Started step: {}", step_name)).await;
                }
            }
            
            // 等待至少一个步骤完成
            if let Some((step_name, result)) = self.wait_for_any_step(&mut running_steps).await {
                match result {
                    Ok(step_result) => {
                        step_results.insert(step_name.clone(), step_result.clone());
                        completed_steps.insert(step_name.clone());
                        
                        if let Some(node) = graph.nodes.get_mut(&step_name) {
                            node.status = NodeStatus::Completed;
                            node.result = Some(step_result);
                            node.end_time = Some(chrono::Utc::now());
                        }
                        
                        self.log_execution_event(execution_id, LogLevel::Info, format!("Completed step: {}", step_name)).await;
                        
                        // 检查依赖该步骤的其他步骤是否可以执行
                        for dependent in &graph.nodes[&step_name].dependents {
                            if self.can_execute_step(dependent, &graph, &completed_steps) {
                                ready_steps.push_back(dependent.clone());
                            }
                        }
                    }
                    Err(e) => {
                        if let Some(node) = graph.nodes.get_mut(&step_name) {
                            node.status = NodeStatus::Failed;
                            node.end_time = Some(chrono::Utc::now());
                        }
                        
                        self.log_execution_event(execution_id, LogLevel::Error, format!("Step {} failed: {}", step_name, e)).await;
                        
                        // 根据重试策略决定是否继续
                        if !self.should_retry_step(&graph.nodes[&step_name].step) {
                            return Err(e);
                        }
                    }
                }
            }
        }
        
        Ok(step_results)
    }
    
    async fn execute_step(
        &self,
        step: WorkflowStep,
        context: WorkflowContext,
        execution_id: Uuid,
    ) -> Result<serde_json::Value> {
        let agent = self.registry.get_agent(&step.agent)
            .ok_or_else(|| AgentError::Configuration(format!("Agent '{}' not found", step.agent)))?;
        
        // 准备输入
        let input = self.prepare_step_input(&step, &context).await?;
        
        // 执行步骤（带重试）
        let mut attempt = 0;
        let max_attempts = step.retry_policy.max_attempts;
        
        loop {
            attempt += 1;
            
            debug!("Executing step {} (attempt {}/{})", step.name, attempt, max_attempts);
            
            match tokio::time::timeout(step.timeout, agent.execute(input.clone())).await {
                Ok(Ok(result)) => {
                    debug!("Step {} completed successfully", step.name);
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    warn!("Step {} failed: {}", step.name, e);
                    
                    if attempt >= max_attempts {
                        error!("Step {} failed after {} attempts", step.name, max_attempts);
                        return Err(e);
                    }
                    
                    // 应用退避策略
                    let delay = self.calculate_backoff(attempt, &step.retry_policy);
                    tokio::time::sleep(delay).await;
                }
                Err(_) => {
                    warn!("Step {} timed out", step.name);
                    
                    if attempt >= max_attempts {
                        error!("Step {} timed out after {} attempts", step.name, max_attempts);
                        return Err(AgentError::Timeout(format!("Step {} timed out", step.name)));
                    }
                    
                    // 应用退避策略
                    let delay = self.calculate_backoff(attempt, &step.retry_policy);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    async fn prepare_step_input(
        &self,
        step: &WorkflowStep,
        context: &WorkflowContext,
    ) -> Result<serde_json::Value> {
        match &step.input {
            WorkflowInput::ProjectPath(path) => Ok(serde_json::json!({
                "project_path": path
            })),
            WorkflowInput::ProjectInfo => {
                // 检测项目信息
                let project_info = ProjectDetector::detect_project(&context.project_path).await?;
                Ok(serde_json::to_value(project_info)?)
            }
            WorkflowInput::AnalysisResults => {
                // 从上下文中获取分析结果
                context.variables.get("analysis_results")
                    .cloned()
                    .ok_or_else(|| AgentError::Execution("Analysis results not found in context".to_string()))
            }
            WorkflowInput::CopilotPrompt => {
                // 生成 Copilot 提示
                let prompt = self.generate_copilot_prompt(context).await?;
                Ok(serde_json::json!({
                    "prompt": prompt
                }))
            }
            WorkflowInput::Custom(value) => Ok(value.clone()),
        }
    }
    
    async fn generate_copilot_prompt(&self, context: &WorkflowContext) -> Result<String> {
        let project_info = ProjectDetector::detect_project(&context.project_path).await?;
        
        let mut prompt = format!(
            "Project Analysis Request\n\n\
            Project Type: {:?}\n\
            Language: {}\n\
            Framework: {:?}\n\
            Build System: {:?}\n\n",
            project_info.project_type,
            project_info.language,
            project_info.framework,
            project_info.build_system
        );
        
        prompt.push_str("Please provide a comprehensive code review focusing on:\n");
        prompt.push_str("- Code quality and best practices\n");
        prompt.push_str("- Security considerations\n");
        prompt.push_str("- Performance optimization opportunities\n");
        prompt.push_str("- Maintainability and readability\n");
        
        if let Some(env) = context.variables.get("environment") {
            prompt.push_str(&format!("\nEnvironment: {}\n", env));
        }
        
        if let Some(pr_number) = context.variables.get("pr_number") {
            prompt.push_str(&format!("\nPR Number: {}\n", pr_number));
        }
        
        Ok(prompt)
    }
    
    fn calculate_backoff(&self, attempt: u32, retry_policy: &RetryPolicy) -> Duration {
        match retry_policy.backoff_strategy {
            BackoffStrategy::Fixed => Duration::from_secs(1),
            BackoffStrategy::Linear => Duration::from_secs(attempt as u64),
            BackoffStrategy::Exponential => {
                let delay = 2u64.pow(attempt.saturating_sub(1));
                Duration::from_secs(delay.min(retry_policy.max_delay.as_secs()))
            }
        }
    }
    
    fn should_retry_step(&self, step: &WorkflowStep) -> bool {
        // 根据步骤类型和错误决定是否重试
        matches!(step.output, WorkflowOutput::AnalysisResults | WorkflowOutput::CopilotResponse)
    }
    
    async fn wait_for_any_step(
        &self,
        running_steps: &mut HashMap<String, JoinHandle<Result<serde_json::Value>>>,
    ) -> Option<(String, Result<serde_json::Value>)> {
        if running_steps.is_empty() {
            return None;
        }
        
        let mut completed = None;
        
        // 使用 select! 等待任何步骤完成
        tokio::select! {
            Some((step_name, result)) = async {
                for (step_name, handle) in running_steps.iter_mut() {
                    if handle.is_finished() {
                        let result = handle.await;
                        return Some((step_name.clone(), result));
                    }
                }
                None
            } => {
                completed = Some(result);
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // 继续等待
            }
        }
        
        if let Some((step_name, result)) = completed {
            running_steps.remove(&step_name);
            Some((step_name, result))
        } else {
            None
        }
    }
    
    fn can_execute_step(
        &self,
        step_name: &str,
        graph: &ExecutionGraph,
        completed_steps: &HashSet<String>,
    ) -> bool {
        if let Some(node) = graph.nodes.get(step_name) {
            // 检查所有依赖是否都已完成
            node.dependencies.iter().all(|dep| completed_steps.contains(dep))
        } else {
            false
        }
    }
    
    async fn generate_outputs(
        &self,
        workflow: &WorkflowDefinition,
        step_results: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut outputs = HashMap::new();
        
        for output_spec in &workflow.outputs {
            let output_value = match output_spec {
                WorkflowOutput::ProjectPath(_) => {
                    step_results.get("project_path")
                        .cloned()
                        .ok_or_else(|| AgentError::Execution("Project path not available".to_string()))?
                }
                WorkflowOutput::ProjectInfo => {
                    step_results.get("project_info")
                        .cloned()
                        .ok_or_else(|| AgentError::Execution("Project info not available".to_string()))?
                }
                WorkflowOutput::AnalysisResults => {
                    step_results.get("analysis_results")
                        .cloned()
                        .ok_or_else(|| AgentError::Execution("Analysis results not available".to_string()))?
                }
                WorkflowOutput::CopilotPrompt => {
                    step_results.get("copilot_prompt")
                        .cloned()
                        .ok_or_else(|| AgentError::Execution("Copilot prompt not available".to_string()))?
                }
                WorkflowOutput::CopilotResponse => {
                    step_results.get("copilot_response")
                        .cloned()
                        .ok_or_else(|| AgentError::Execution("Copilot response not available".to_string()))?
                }
                WorkflowOutput::Report => {
                    self.generate_report(step_results).await?
                }
                WorkflowOutput::Custom(value) => value.clone(),
            };
            
            outputs.insert(self.output_name_to_string(output_spec), output_value);
        }
        
        Ok(outputs)
    }
    
    fn output_name_to_string(&self, output: &WorkflowOutput) -> String {
        match output {
            WorkflowOutput::ProjectPath(_) => "project_path".to_string(),
            WorkflowOutput::ProjectInfo => "project_info".to_string(),
            WorkflowOutput::AnalysisResults => "analysis_results".to_string(),
            WorkflowOutput::CopilotPrompt => "copilot_prompt".to_string(),
            WorkflowOutput::CopilotResponse => "copilot_response".to_string(),
            WorkflowOutput::Report => "report".to_string(),
            WorkflowOutput::Custom(value) => {
                if let Some(name) = value.get("name").and_then(|n| n.as_str()) {
                    name.to_string()
                } else {
                    "custom_output".to_string()
                }
            }
        }
    }
    
    async fn generate_report(&self, step_results: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value> {
        let mut report = HashMap::new();
        
        // 添加项目信息
        if let Some(project_info) = step_results.get("project_info") {
            report.insert("project_info".to_string(), project_info.clone());
        }
        
        // 添加分析结果
        if let Some(analysis_results) = step_results.get("analysis_results") {
            report.insert("analysis_results".to_string(), analysis_results.clone());
        }
        
        // 添加 Copilot 响应
        if let Some(copilot_response) = step_results.get("copilot_response") {
            report.insert("copilot_response".to_string(), copilot_response.clone());
        }
        
        // 生成报告摘要
        let summary = self.generate_report_summary(&report).await?;
        report.insert("summary".to_string(), serde_json::Value::String(summary));
        
        // 添加时间戳
        report.insert("generated_at".to_string(), serde_json::Value::String(
            chrono::Utc::now().to_rfc3339()
        ));
        
        Ok(serde_json::Value::Object(report.into_iter().collect()))
    }
    
    async fn generate_report_summary(&self, report: &HashMap<String, serde_json::Value>) -> Result<String> {
        let mut summary = String::new();
        summary.push_str("Claude Code + Copilot Analysis Report\n");
        summary.push_str("=====================================\n\n");
        
        if let Some(project_info) = report.get("project_info") {
            summary.push_str("Project Information:\n");
            if let Some(obj) = project_info.as_object() {
                for (key, value) in obj {
                    summary.push_str(&format!("  {}: {}\n", key, value));
                }
            }
            summary.push('\n');
        }
        
        if let Some(analysis_results) = report.get("analysis_results") {
            summary.push_str("Analysis Results:\n");
            if let Some(obj) = analysis_results.as_object() {
                for (key, value) in obj {
                    summary.push_str(&format!("  {}: {}\n", key, value));
                }
            }
            summary.push('\n');
        }
        
        if let Some(copilot_response) = report.get("copilot_response") {
            summary.push_str("Copilot Review:\n");
            if let Some(obj) = copilot_response.as_object() {
                if let Some(suggestions) = obj.get("suggestions").and_then(|s| s.as_array()) {
                    for (i, suggestion) in suggestions.iter().enumerate() {
                        summary.push_str(&format!("  {}. {}\n", i + 1, suggestion));
                    }
                }
            }
            summary.push('\n');
        }
        
        Ok(summary)
    }
    
    async fn log_execution_event(&self, execution_id: Uuid, level: LogLevel, message: String) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            message,
            metadata: HashMap::new(),
        };
        
        let mut history = self.execution_history.write().await;
        if let Some(execution) = history.iter_mut().find(|e| e.id == execution_id) {
            execution.logs.push(log_entry);
        }
    }
    
    pub async fn get_execution_history(&self) -> Vec<WorkflowExecution> {
        let history = self.execution_history.read().await;
        history.clone()
    }
    
    pub async fn get_execution(&self, execution_id: Uuid) -> Option<WorkflowExecution> {
        let history = self.execution_history.read().await;
        history.iter().find(|e| e.id == execution_id).cloned()
    }
}

// 辅助类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionGraph {
    nodes: HashMap<String, ExecutionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionNode {
    step: WorkflowStep,
    status: NodeStatus,
    dependencies: Vec<String>,
    dependents: Vec<String>,
    result: Option<serde_json::Value>,
    start_time: Option<chrono::DateTime<chrono::Utc>>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub execution_time: Option<Duration>,
    pub status: ExecutionStatus,
    pub context: WorkflowContext,
    pub step_results: HashMap<String, serde_json::Value>,
    pub logs: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: Option<WorkflowConfig>,
}

impl ConfigManager {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            config: None,
        }
    }
    
    pub async fn load_config(&mut self) -> Result<WorkflowConfig> {
        let config_content = tokio::fs::read_to_string(&self.config_path).await?;
        let config: WorkflowConfig = toml::from_str(&config_content)
            .map_err(|e| AgentError::Configuration(e.to_string()))?;
        
        // 验证配置
        self.validate_config(&config).await?;
        
        // 合并默认配置
        let merged_config = self.merge_with_defaults(config);
        
        self.config = Some(merged_config.clone());
        Ok(merged_config)
    }
    
    async fn validate_config(&self, config: &WorkflowConfig) -> Result<()> {
        // 检查必需字段
        if config.version.is_empty() {
            return Err(AgentError::Configuration("Config version cannot be empty".to_string()));
        }
        
        // 验证项目配置
        if let Some(project) = &config.project {
            if project.name.is_empty() {
                return Err(AgentError::Configuration("Project name cannot be empty".to_string()));
            }
        }
        
        // 验证质量门禁
        if let Some(quality_gates) = &config.quality_gates {
            if let Some(code_quality) = &quality_gates.code_quality {
                if code_quality.max_errors > 0 && code_quality.fail_on_error {
                    return Err(AgentError::Configuration(
                        "Cannot have max_errors > 0 with fail_on_error".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    fn merge_with_defaults(&self, config: WorkflowConfig) -> WorkflowConfig {
        // 这里可以实现默认配置的合并逻辑
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub version: String,
    pub project: Option<ProjectConfig>,
    pub analysis: Option<AnalysisConfig>,
    pub copilot: Option<CopilotConfig>,
    pub quality_gates: Option<QualityGatesConfig>,
    pub reporting: Option<ReportingConfig>,
    pub integrations: Option<IntegrationsConfig>,
    pub environments: Option<EnvironmentsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub project_type: String,
    pub language: String,
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub basics: Option<AnalysisBasicsConfig>,
    pub advanced: Option<AnalysisAdvancedConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisBasicsConfig {
    pub code_quality: bool,
    pub security_scan: bool,
    pub dependency_check: bool,
    pub test_coverage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisAdvancedConfig {
    pub performance_analysis: bool,
    pub architecture_review: bool,
    pub best_practices: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotConfig {
    pub focus_areas: Vec<String>,
    pub template: String,
    pub depth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGatesConfig {
    pub code_quality: Option<CodeQualityConfig>,
    pub security: Option<SecurityConfig>,
    pub test_coverage: Option<TestCoverageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityConfig {
    pub max_warnings: u32,
    pub max_errors: u32,
    pub fail_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub max_vulnerabilities: u32,
    pub severity_levels: Vec<String>,
    pub fail_on_vulnerability: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverageConfig {
    pub min_coverage: u32,
    pub fail_below_min: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    pub formats: Vec<String>,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    pub github: Option<GithubConfig>,
    pub cargo: Option<CargoConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    pub enabled: bool,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoConfig {
    pub enabled: bool,
    pub clippy: bool,
    pub fmt: bool,
    pub audit: bool,
    pub doc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentsConfig {
    pub development: Option<EnvironmentConfig>,
    pub testing: Option<EnvironmentConfig>,
    pub production: Option<EnvironmentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub skip_security: bool,
    pub skip_performance: bool,
    pub skip_tests: bool,
    pub require_approval: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_validation() {
        let registry = AgentRegistry::new();
        let config_manager = ConfigManager::new("test_config.toml");
        let engine = WorkflowEngine::new(registry, config_manager);
        
        // 创建一个简单的工作流
        let workflow = WorkflowDefinition {
            name: "test_workflow".to_string(),
            version: "1.0".to_string(),
            description: "Test workflow".to_string(),
            triggers: vec![WorkflowTrigger::Manual],
            steps: vec![],
            variables: HashMap::new(),
            outputs: vec![],
        };
        
        let context = WorkflowContext {
            project_path: PathBuf::from("/test"),
            environment: Environment::Development,
            variables: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // 验证工作流
        let result = engine.execute_workflow(workflow, context).await;
        assert!(result.is_ok());
    }
}