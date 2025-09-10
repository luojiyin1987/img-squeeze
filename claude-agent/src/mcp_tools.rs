use crate::core::*;
use crate::project_detector::ProjectDetector;
use crate::workflow_engine::WorkflowEngine;
use crate::config::ConfigManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use futures::future::BoxFuture;

// MCP 工具注册器
pub struct MCPToolRegistry {
    tools: HashMap<String, MCPTool>,
    workflow_engine: Arc<RwLock<WorkflowEngine>>,
    config_manager: ConfigManager,
}

impl MCPToolRegistry {
    pub fn new(
        workflow_engine: Arc<RwLock<WorkflowEngine>>,
        config_manager: ConfigManager,
    ) -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            workflow_engine,
            config_manager,
        };
        
        registry.register_default_tools();
        registry
    }
    
    fn register_default_tools(&mut self) {
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
                    "package_manager": {"type": "string"},
                    "dependencies": {"type": "array", "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "version": {"type": "string"},
                            "source": {"type": "string"}
                        }
                    }}
                }
            }),
            handler: Arc::new(detect_project_handler),
        });
        
        // 注册工作流配置生成工具
        self.register_tool(MCPTool {
            name: "generate_workflow_config".to_string(),
            description: "Generate workflow configuration for a project".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    },
                    "config_type": {
                        "type": "string",
                        "enum": ["minimal", "standard", "comprehensive"],
                        "default": "standard",
                        "description": "Type of configuration to generate"
                    }
                },
                "required": ["project_path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "config": {"type": "object"},
                    "config_path": {"type": "string"},
                    "description": {"type": "string"}
                }
            }),
            handler: Arc::new(generate_config_handler),
        });
        
        // 注册代码分析工具
        self.register_tool(MCPTool {
            name: "analyze_code_quality".to_string(),
            description: "Analyze code quality using project-specific tools".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    },
                    "include_tests": {
                        "type": "boolean",
                        "default": false,
                        "description": "Include test files in analysis"
                    },
                    "strict_mode": {
                        "type": "boolean",
                        "default": false,
                        "description": "Use strict analysis mode"
                    }
                },
                "required": ["project_path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "quality_score": {"type": "number"},
                    "issues": {"type": "array", "items": {
                        "type": "object",
                        "properties": {
                            "severity": {"type": "string"},
                            "message": {"type": "string"},
                            "file": {"type": "string"},
                            "line": {"type": "number"}
                        }
                    }},
                    "suggestions": {"type": "array", "items": {"type": "string"}},
                    "execution_time": {"type": "number"}
                }
            }),
            handler: Arc::new(analyze_code_quality_handler),
        });
        
        // 注册安全扫描工具
        self.register_tool(MCPTool {
            name: "security_scan".to_string(),
            description: "Perform security scan on the project".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    },
                    "scan_dependencies": {
                        "type": "boolean",
                        "default": true,
                        "description": "Scan dependencies for vulnerabilities"
                    },
                    "scan_code": {
                        "type": "boolean",
                        "default": true,
                        "description": "Scan code for security issues"
                    }
                },
                "required": ["project_path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "vulnerabilities": {"type": "array", "items": {
                        "type": "object",
                        "properties": {
                            "severity": {"type": "string"},
                            "package": {"type": "string"},
                            "version": {"type": "string"},
                            "description": {"type": "string"}
                        }
                    }},
                    "security_score": {"type": "number"},
                    "recommendations": {"type": "array", "items": {"type": "string"}},
                    "execution_time": {"type": "number"}
                }
            }),
            handler: Arc::new(security_scan_handler),
        });
        
        // 注册 Copilot 集成工具
        self.register_tool(MCPTool {
            name: "copilot_review".to_string(),
            description: "Generate Copilot review for code changes".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    },
                    "pr_number": {
                        "type": "integer",
                        "description": "Pull request number"
                    },
                    "focus_areas": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["code_quality", "security", "performance", "maintainability", "testing", "documentation"]
                        },
                        "description": "Areas to focus the review on"
                    },
                    "depth": {
                        "type": "string",
                        "enum": ["basic", "detailed", "comprehensive"],
                        "default": "detailed",
                        "description": "Review depth"
                    }
                },
                "required": ["project_path", "pr_number"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "review_comments": {"type": "array", "items": {"type": "string"}},
                    "suggestions": {"type": "array", "items": {"type": "string"}},
                    "overall_score": {"type": "number"},
                    "focus_areas": {"type": "array", "items": {"type": "string"}},
                    "execution_time": {"type": "number"}
                }
            }),
            handler: Arc::new(copilot_review_handler),
        });
        
        // 注册性能分析工具
        self.register_tool(MCPTool {
            name: "performance_analysis".to_string(),
            description: "Analyze performance characteristics of the project".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    },
                    "analysis_type": {
                        "type": "string",
                        "enum": ["build_time", "runtime", "memory", "cpu"],
                        "default": "runtime",
                        "description": "Type of performance analysis"
                    }
                },
                "required": ["project_path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "performance_metrics": {"type": "object"},
                    "bottlenecks": {"type": "array", "items": {"type": "string"}},
                    "optimization_suggestions": {"type": "array", "items": {"type": "string"}},
                    "execution_time": {"type": "number"}
                }
            }),
            handler: Arc::new(performance_analysis_handler),
        });
        
        // 注册测试覆盖率分析工具
        self.register_tool(MCPTool {
            name: "test_coverage_analysis".to_string(),
            description: "Analyze test coverage of the project".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    },
                    "coverage_type": {
                        "type": "string",
                        "enum": ["line", "branch", "function", "statement"],
                        "default": "line",
                        "description": "Type of coverage analysis"
                    }
                },
                "required": ["project_path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "coverage_percentage": {"type": "number"},
                    "uncovered_files": {"type": "array", "items": {"type": "string"}},
                    "coverage_report": {"type": "string"},
                    "suggestions": {"type": "array", "items": {"type": "string"}},
                    "execution_time": {"type": "number"}
                }
            }),
            handler: Arc::new(test_coverage_analysis_handler),
        });
        
        // 注册文档生成工具
        self.register_tool(MCPTool {
            name: "generate_documentation".to_string(),
            description: "Generate documentation for the project".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    },
                    "doc_type": {
                        "type": "string",
                        "enum": ["api", "user_guide", "technical", "readme"],
                        "default": "readme",
                        "description": "Type of documentation to generate"
                    },
                    "output_format": {
                        "type": "string",
                        "enum": ["markdown", "html", "pdf"],
                        "default": "markdown",
                        "description": "Output format for documentation"
                    }
                },
                "required": ["project_path"]
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "documentation": {"type": "string"},
                    "generated_files": {"type": "array", "items": {"type": "string"}},
                    "execution_time": {"type": "number"}
                }
            }),
            handler: Arc::new(generate_documentation_handler),
        });
    }
    
    pub fn register_tool(&mut self, tool: MCPTool) {
        self.tools.insert(tool.name.clone(), tool);
    }
    
    pub fn get_tool(&self, name: &str) -> Option<&MCPTool> {
        self.tools.get(name)
    }
    
    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
    
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let tool = self.tools.get(tool_name)
            .ok_or_else(|| AgentError::Configuration(format!("Tool '{}' not found", tool_name)))?;
        
        // 验证输入模式
        self.validate_input_schema(&tool.input_schema, &input)?;
        
        // 执行工具
        let start_time = std::time::Instant::now();
        let result = (tool.handler)(input, self.workflow_engine.clone(), self.config_manager.clone()).await;
        let execution_time = start_time.elapsed();
        
        match result {
            Ok(mut output) => {
                // 添加执行时间
                if let Some(obj) = output.as_object_mut() {
                    obj.insert("execution_time".to_string(), serde_json::Value::Number(
                        serde_json::Number::from(execution_time.as_secs_f64())
                    ));
                }
                Ok(output)
            }
            Err(e) => Err(e),
        }
    }
    
    fn validate_input_schema(&self, schema: &serde_json::Value, input: &serde_json::Value) -> Result<()> {
        // 简化的模式验证（实际项目中应该使用完整的 JSON Schema 验证）
        if let Some(schema_obj) = schema.as_object() {
            if let Some(required) = schema_obj.get("required").and_then(|r| r.as_array()) {
                for required_field in required {
                    if let Some(field_name) = required_field.as_str() {
                        if !input.as_object()
                            .and_then(|obj| obj.get(field_name))
                            .is_some() {
                            return Err(AgentError::Configuration(format!(
                                "Missing required field: {}", field_name
                            )));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_tool_schemas(&self) -> HashMap<String, (serde_json::Value, serde_json::Value)> {
        self.tools.iter()
            .map(|(name, tool)| {
                (name.clone(), (tool.input_schema.clone(), tool.output_schema.clone()))
            })
            .collect()
    }
}

// MCP 工具定义
#[derive(Debug, Clone)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub handler: ToolHandler,
}

// 工具处理器类型
pub type ToolHandler = Arc<dyn Fn(serde_json::Value, Arc<RwLock<WorkflowEngine>>, Arc<RwLock<ConfigManager>>) -> 
    BoxFuture<'static, Result<serde_json::Value>> + Send + Sync>;

// 工具处理器实现
async fn execute_workflow_handler(
    input: serde_json::Value,
    workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let workflow_type = input.get("workflow_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing workflow_type".to_string()))?;
    
    let pr_number = input.get("pr_number").and_then(|v| v.as_u64());
    
    // 创建工作流定义
    let workflow = create_workflow_definition(workflow_type, pr_number)?;
    
    // 创建上下文
    let context = WorkflowContext {
        project_path: PathBuf::from(project_path),
        environment: Environment::Development,
        variables: HashMap::new(),
        metadata: HashMap::new(),
    };
    
    // 执行工作流
    let engine = workflow_engine.read().await;
    let result = engine.execute_workflow(workflow, context).await?;
    
    Ok(serde_json::json!({
        "success": result.success,
        "outputs": result.outputs,
        "execution_time": result.execution_time.as_secs_f64(),
        "error": result.error,
        "logs": result.logs
    }))
}

async fn detect_project_handler(
    input: serde_json::Value,
    _workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let project_info = ProjectDetector::detect_project(project_path).await?;
    
    Ok(serde_json::json!({
        "project_type": format!("{:?}", project_info.project_type),
        "language": project_info.language,
        "framework": project_info.framework,
        "build_system": project_info.build_system,
        "package_manager": project_info.package_manager,
        "dependencies": project_info.dependencies,
        "config_files": project_info.config_files,
        "root_path": project_info.root_path
    }))
}

async fn generate_config_handler(
    input: serde_json::Value,
    _workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let config_type = input.get("config_type")
        .and_then(|v| v.as_str())
        .unwrap_or("standard");
    
    let project_info = ProjectDetector::detect_project(project_path).await?;
    
    let config = generate_project_config(&project_info, config_type)?;
    
    Ok(serde_json::json!({
        "config": config,
        "config_path": format!("{}/.claude-workflow.yml", project_path),
        "description": format!("Generated {} configuration for {:?} project", config_type, project_info.project_type)
    }))
}

async fn analyze_code_quality_handler(
    input: serde_json::Value,
    _workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let include_tests = input.get("include_tests").and_then(|v| v.as_bool()).unwrap_or(false);
    let strict_mode = input.get("strict_mode").and_then(|v| v.as_bool()).unwrap_or(false);
    
    // 执行代码质量分析
    let analysis_result = analyze_code_quality(project_path, include_tests, strict_mode).await?;
    
    Ok(serde_json::json!({
        "quality_score": analysis_result.quality_score,
        "issues": analysis_result.issues,
        "suggestions": analysis_result.suggestions,
        "summary": analysis_result.summary
    }))
}

async fn security_scan_handler(
    input: serde_json::Value,
    _workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let scan_dependencies = input.get("scan_dependencies").and_then(|v| v.as_bool()).unwrap_or(true);
    let scan_code = input.get("scan_code").and_then(|v| v.as_bool()).unwrap_or(true);
    
    // 执行安全扫描
    let scan_result = perform_security_scan(project_path, scan_dependencies, scan_code).await?;
    
    Ok(serde_json::json!({
        "vulnerabilities": scan_result.vulnerabilities,
        "security_score": scan_result.security_score,
        "recommendations": scan_result.recommendations,
        "summary": scan_result.summary
    }))
}

async fn copilot_review_handler(
    input: serde_json::Value,
    _workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let pr_number = input.get("pr_number")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AgentError::Configuration("Missing pr_number".to_string()))?;
    
    let focus_areas = input.get("focus_areas")
        .and_then(|v| v.as_array())
        .unwrap_or(&serde_json::Value::Array(vec![]))
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    
    let depth = input.get("depth")
        .and_then(|v| v.as_str())
        .unwrap_or("detailed");
    
    // 生成 Copilot 评论
    let review_result = generate_copilot_review(project_path, pr_number, focus_areas, depth).await?;
    
    Ok(serde_json::json!({
        "review_comments": review_result.comments,
        "suggestions": review_result.suggestions,
        "overall_score": review_result.overall_score,
        "focus_areas": review_result.focus_areas,
        "summary": review_result.summary
    }))
}

async fn performance_analysis_handler(
    input: serde_json::Value,
    _workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let analysis_type = input.get("analysis_type")
        .and_then(|v| v.as_str())
        .unwrap_or("runtime");
    
    // 执行性能分析
    let analysis_result = perform_performance_analysis(project_path, analysis_type).await?;
    
    Ok(serde_json::json!({
        "performance_metrics": analysis_result.metrics,
        "bottlenecks": analysis_result.bottlenecks,
        "optimization_suggestions": analysis_result.suggestions,
        "summary": analysis_result.summary
    }))
}

async fn test_coverage_analysis_handler(
    input: serde_json::Value,
    _workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let coverage_type = input.get("coverage_type")
        .and_then(|v| v.as_str())
        .unwrap_or("line");
    
    // 执行测试覆盖率分析
    let coverage_result = analyze_test_coverage(project_path, coverage_type).await?;
    
    Ok(serde_json::json!({
        "coverage_percentage": coverage_result.coverage_percentage,
        "uncovered_files": coverage_result.uncovered_files,
        "coverage_report": coverage_result.report,
        "suggestions": coverage_result.suggestions,
        "summary": coverage_result.summary
    }))
}

async fn generate_documentation_handler(
    input: serde_json::Value,
    _workflow_engine: Arc<RwLock<WorkflowEngine>>,
    _config_manager: Arc<RwLock<ConfigManager>>,
) -> Result<serde_json::Value> {
    let project_path = input.get("project_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError::Configuration("Missing project_path".to_string()))?;
    
    let doc_type = input.get("doc_type")
        .and_then(|v| v.as_str())
        .unwrap_or("readme");
    
    let output_format = input.get("output_format")
        .and_then(|v| v.as_str())
        .unwrap_or("markdown");
    
    // 生成文档
    let doc_result = generate_project_documentation(project_path, doc_type, output_format).await?;
    
    Ok(serde_json::json!({
        "documentation": doc_result.content,
        "generated_files": doc_result.generated_files,
        "summary": doc_result.summary
    }))
}

// 辅助函数
fn create_workflow_definition(workflow_type: &str, pr_number: Option<u64>) -> Result<WorkflowDefinition> {
    match workflow_type {
        "pr_review" => {
            let mut steps = vec![
                WorkflowStep {
                    name: "detect_project".to_string(),
                    agent: "project_detector".to_string(),
                    input: WorkflowInput::ProjectPath(PathBuf::from(".")),
                    output: WorkflowOutput::ProjectInfo,
                    dependencies: vec![],
                    retry_policy: RetryPolicy::default(),
                    timeout: Duration::from_secs(30),
                },
                WorkflowStep {
                    name: "analyze_code".to_string(),
                    agent: "analysis_engine".to_string(),
                    input: WorkflowInput::ProjectInfo,
                    output: WorkflowOutput::AnalysisResults,
                    dependencies: vec!["detect_project".to_string()],
                    retry_policy: RetryPolicy::default(),
                    timeout: Duration::from_secs(300),
                },
            ];
            
            if let Some(pr_num) = pr_number {
                steps.push(WorkflowStep {
                    name: "copilot_review".to_string(),
                    agent: "copilot_integration".to_string(),
                    input: WorkflowInput::CopilotPrompt,
                    output: WorkflowOutput::CopilotResponse,
                    dependencies: vec!["analyze_code".to_string()],
                    retry_policy: RetryPolicy::default(),
                    timeout: Duration::from_secs(600),
                });
            }
            
            Ok(WorkflowDefinition {
                name: format!("PR Review Workflow - PR #{}", pr_number.unwrap_or(0)),
                version: "1.0".to_string(),
                description: "Comprehensive PR review workflow".to_string(),
                triggers: vec![WorkflowTrigger::PullRequest],
                steps,
                variables: HashMap::new(),
                outputs: vec![WorkflowOutput::Report],
            })
        }
        "full_analysis" => {
            Ok(WorkflowDefinition {
                name: "Full Analysis Workflow".to_string(),
                version: "1.0".to_string(),
                description: "Comprehensive code analysis workflow".to_string(),
                triggers: vec![WorkflowTrigger::Manual],
                steps: vec![
                    WorkflowStep {
                        name: "detect_project".to_string(),
                        agent: "project_detector".to_string(),
                        input: WorkflowInput::ProjectPath(PathBuf::from(".")),
                        output: WorkflowOutput::ProjectInfo,
                        dependencies: vec![],
                        retry_policy: RetryPolicy::default(),
                        timeout: Duration::from_secs(30),
                    },
                    WorkflowStep {
                        name: "analyze_code".to_string(),
                        agent: "analysis_engine".to_string(),
                        input: WorkflowInput::ProjectInfo,
                        output: WorkflowOutput::AnalysisResults,
                        dependencies: vec!["detect_project".to_string()],
                        retry_policy: RetryPolicy::default(),
                        timeout: Duration::from_secs(300),
                    },
                    WorkflowStep {
                        name: "security_scan".to_string(),
                        agent: "security_scanner".to_string(),
                        input: WorkflowInput::ProjectInfo,
                        output: WorkflowOutput::AnalysisResults,
                        dependencies: vec!["detect_project".to_string()],
                        retry_policy: RetryPolicy::default(),
                        timeout: Duration::from_secs(180),
                    },
                ],
                variables: HashMap::new(),
                outputs: vec![WorkflowOutput::Report],
            })
        }
        "security_scan" => {
            Ok(WorkflowDefinition {
                name: "Security Scan Workflow".to_string(),
                version: "1.0".to_string(),
                description: "Security-focused analysis workflow".to_string(),
                triggers: vec![WorkflowTrigger::Manual],
                steps: vec![
                    WorkflowStep {
                        name: "detect_project".to_string(),
                        agent: "project_detector".to_string(),
                        input: WorkflowInput::ProjectPath(PathBuf::from(".")),
                        output: WorkflowOutput::ProjectInfo,
                        dependencies: vec![],
                        retry_policy: RetryPolicy::default(),
                        timeout: Duration::from_secs(30),
                    },
                    WorkflowStep {
                        name: "security_scan".to_string(),
                        agent: "security_scanner".to_string(),
                        input: WorkflowInput::ProjectInfo,
                        output: WorkflowOutput::AnalysisResults,
                        dependencies: vec!["detect_project".to_string()],
                        retry_policy: RetryPolicy::default(),
                        timeout: Duration::from_secs(180),
                    },
                ],
                variables: HashMap::new(),
                outputs: vec![WorkflowOutput::Report],
            })
        }
        "performance_analysis" => {
            Ok(WorkflowDefinition {
                name: "Performance Analysis Workflow".to_string(),
                version: "1.0".to_string(),
                description: "Performance-focused analysis workflow".to_string(),
                triggers: vec![WorkflowTrigger::Manual],
                steps: vec![
                    WorkflowStep {
                        name: "detect_project".to_string(),
                        agent: "project_detector".to_string(),
                        input: WorkflowInput::ProjectPath(PathBuf::from(".")),
                        output: WorkflowOutput::ProjectInfo,
                        dependencies: vec![],
                        retry_policy: RetryPolicy::default(),
                        timeout: Duration::from_secs(30),
                    },
                    WorkflowStep {
                        name: "performance_analysis".to_string(),
                        agent: "performance_analyzer".to_string(),
                        input: WorkflowInput::ProjectInfo,
                        output: WorkflowOutput::AnalysisResults,
                        dependencies: vec!["detect_project".to_string()],
                        retry_policy: RetryPolicy::default(),
                        timeout: Duration::from_secs(240),
                    },
                ],
                variables: HashMap::new(),
                outputs: vec![WorkflowOutput::Report],
            })
        }
        _ => Err(AgentError::Configuration(format!("Unknown workflow type: {}", workflow_type))),
    }
}

// 模拟的分析函数（实际项目中会调用真实的分析工具）
async fn analyze_code_quality(project_path: &str, include_tests: bool, strict_mode: bool) -> Result<CodeQualityResult> {
    // 这里应该调用真实的代码质量分析工具
    // 例如：cargo clippy, npm lint, pylint 等
    
    Ok(CodeQualityResult {
        quality_score: 85.0,
        issues: vec![
            CodeIssue {
                severity: "warning".to_string(),
                message: "Unused variable detected".to_string(),
                file: "src/main.rs".to_string(),
                line: 42,
            },
        ],
        suggestions: vec![
            "Remove unused variables to improve code clarity".to_string(),
            "Add more comprehensive error handling".to_string(),
        ],
        summary: "Good overall code quality with minor issues".to_string(),
    })
}

async fn perform_security_scan(project_path: &str, scan_dependencies: bool, scan_code: bool) -> Result<SecurityScanResult> {
    // 这里应该调用真实的安全扫描工具
    // 例如：cargo audit, npm audit, bandit 等
    
    Ok(SecurityScanResult {
        vulnerabilities: vec![],
        security_score: 95.0,
        recommendations: vec![
            "Enable dependency vulnerability scanning in CI/CD pipeline".to_string(),
            "Implement input validation for user-facing endpoints".to_string(),
        ],
        summary: "No critical security vulnerabilities found".to_string(),
    })
}

async fn generate_copilot_review(project_path: &str, pr_number: u64, focus_areas: Vec<String>, depth: &str) -> Result<CopilotReviewResult> {
    // 这里应该生成真实的 Copilot 评论
    // 例如：通过 GitHub API 调用 Copilot
    
    Ok(CopilotReviewResult {
        comments: vec![
            "Good overall code structure and organization".to_string(),
            "Consider adding more comprehensive error handling".to_string(),
        ],
        suggestions: vec![
            "Add unit tests for the new functionality".to_string(),
            "Consider using async/await for better performance".to_string(),
        ],
        overall_score: 88,
        focus_areas,
        summary: format!("Comprehensive review for PR #{}", pr_number),
    })
}

async fn perform_performance_analysis(project_path: &str, analysis_type: &str) -> Result<PerformanceAnalysisResult> {
    // 这里应该调用真实的性能分析工具
    // 例如：benchmarking, profiling 等
    
    Ok(PerformanceAnalysisResult {
        metrics: serde_json::json!({
            "build_time": 2.5,
            "memory_usage": "45MB",
            "cpu_usage": "15%"
        }),
        bottlenecks: vec![
            "Database query optimization needed".to_string(),
        ],
        suggestions: vec![
            "Implement database connection pooling".to_string(),
            "Consider caching frequently accessed data".to_string(),
        ],
        summary: format!("Performance analysis completed for {}", analysis_type),
    })
}

async fn analyze_test_coverage(project_path: &str, coverage_type: &str) -> Result<TestCoverageResult> {
    // 这里应该调用真实的测试覆盖率工具
    // 例如：cargo tarpaulin, jest coverage, pytest-cov 等
    
    Ok(TestCoverageResult {
        coverage_percentage: 78.5,
        uncovered_files: vec![
            "src/utils.rs".to_string(),
        ],
        report: "Test coverage report generated".to_string(),
        suggestions: vec![
            "Add tests for utility functions".to_string(),
            "Consider increasing coverage to 85%".to_string(),
        ],
        summary: format!("Test coverage analysis completed for {}", coverage_type),
    })
}

async fn generate_project_documentation(project_path: &str, doc_type: &str, output_format: &str) -> Result<DocumentationResult> {
    // 这里应该生成真实的文档
    // 例如：使用文档生成工具
    
    Ok(DocumentationResult {
        content: "# Project Documentation\n\nThis is auto-generated documentation.".to_string(),
        generated_files: vec![format!("README.{}", output_format)],
        summary: format!("{} documentation generated in {} format", doc_type, output_format),
    })
}

async fn generate_project_config(project_info: &ProjectInfo, config_type: &str) -> Result<serde_json::Value> {
    // 根据项目信息生成配置
    let config = match config_type {
        "minimal" => {
            serde_json::json!({
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
            })
        }
        "standard" => {
            serde_json::json!({
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
            })
        }
        "comprehensive" => {
            serde_json::json!({
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
            })
        }
        _ => {
            return Err(AgentError::Configuration(format!("Unknown config type: {}", config_type)));
        }
    };
    
    Ok(config)
}

// 结果类型定义
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeQualityResult {
    pub quality_score: f64,
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeIssue {
    pub severity: String,
    pub message: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityScanResult {
    pub vulnerabilities: Vec<Vulnerability>,
    pub security_score: f64,
    pub recommendations: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vulnerability {
    pub severity: String,
    pub package: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopilotReviewResult {
    pub comments: Vec<String>,
    pub suggestions: Vec<String>,
    pub overall_score: u8,
    pub focus_areas: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceAnalysisResult {
    pub metrics: serde_json::Value,
    pub bottlenecks: Vec<String>,
    pub suggestions: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCoverageResult {
    pub coverage_percentage: f64,
    pub uncovered_files: Vec<String>,
    pub report: String,
    pub suggestions: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentationResult {
    pub content: String,
    pub generated_files: Vec<String>,
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tool_registry() {
        let workflow_engine = Arc::new(RwLock::new(WorkflowEngine::new(
            AgentRegistry::new(),
            ConfigManager::new("test.toml"),
        )));
        let config_manager = Arc::new(RwLock::new(ConfigManager::new("test.toml")));
        
        let registry = MCPToolRegistry::new(workflow_engine, config_manager);
        
        assert!(registry.list_tools().contains(&"execute_claude_copilot_workflow"));
        assert!(registry.list_tools().contains(&"detect_project_type"));
        assert!(registry.list_tools().contains(&"generate_workflow_config"));
    }
    
    #[tokio::test]
    async fn test_project_detection_tool() {
        let workflow_engine = Arc::new(RwLock::new(WorkflowEngine::new(
            AgentRegistry::new(),
            ConfigManager::new("test.toml"),
        )));
        let config_manager = Arc::new(RwLock::new(ConfigManager::new("test.toml")));
        
        let registry = MCPToolRegistry::new(workflow_engine, config_manager);
        
        let input = serde_json::json!({
            "project_path": "/nonexistent/path"
        });
        
        let result = registry.execute_tool("detect_project_type", input).await;
        
        // 预期失败，因为路径不存在
        assert!(result.is_err());
    }
}