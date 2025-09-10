use claude_agent::ClaudeAgent;
use tempfile::TempDir;
use std::fs;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🤖 Claude Code Agent System - Basic Usage Example");
    println!("=" .repeat(50));
    
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();
    
    // Create a simple Rust project for testing
    create_test_rust_project(project_path).await?;
    
    println!("📁 Created test project at: {:?}", project_path);
    
    // Create a temporary config file
    let config_path = temp_dir.path().join("claude-config.json");
    let config_content = serde_json::json!({
        "version": "1.0.0",
        "agents": {
            "code_analysis": {
                "name": "code_analysis",
                "version": "1.0.0",
                "description": "Code analysis agent",
                "enabled": true,
                "settings": {},
                "dependencies": []
            },
            "github_integration": {
                "name": "github_integration",
                "version": "1.0.0",
                "description": "GitHub integration agent",
                "enabled": true,
                "settings": {
                    "dry_run": true
                },
                "dependencies": []
            }
        },
        "workflows": {},
        "global_settings": {
            "log_level": "info",
            "max_concurrent_agents": 4
        }
    });
    
    fs::write(&config_path, serde_json::to_string_pretty(&config_content)?)?;
    
    println!("⚙️  Created configuration file");
    
    // Initialize the Claude Agent system
    println!("🚀 Initializing Claude Agent system...");
    let agent = ClaudeAgent::new(&config_path).await?;
    
    println!("✅ Agent system initialized successfully!");
    
    // List available agents
    println!("\n📋 Available Agents:");
    for agent_name in agent.registry.list_agents() {
        println!("  - {}", agent_name);
    }
    
    // Detect project type
    println!("\n🔍 Detecting project type...");
    let project_info = agent.detect_project(project_path).await?;
    println!("✅ Project detected:");
    println!("  Type: {:?}", project_info.project_type);
    println!("  Language: {}", project_info.language);
    println!("  Build System: {:?}", project_info.build_system);
    
    // Execute a simple code analysis
    println!("\n🔬 Executing code analysis...");
    let workflow = claude_agent::create_full_analysis_workflow()?;
    let context = claude_agent::WorkflowContext {
        project_path: project_path.to_path_buf(),
        environment: claude_agent::Environment::Development,
        variables: std::collections::HashMap::new(),
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("example".to_string(), "basic_usage".to_string());
            meta
        },
    };
    
    match agent.execute_workflow(workflow, context).await {
        Ok(result) => {
            println!("✅ Workflow execution completed!");
            println!("  Success: {}", result.success);
            println!("  Execution time: {:?}", result.execution_time);
            println!("  Outputs: {} items", result.outputs.len());
            
            if !result.logs.is_empty() {
                println!("\n📝 Execution Logs:");
                for log in result.logs {
                    println!("  [{}] {}: {}", log.timestamp, log.level, log.message);
                }
            }
        }
        Err(e) => {
            println!("❌ Workflow execution failed: {}", e);
        }
    }
    
    // List available MCP tools
    println!("\n🛠️  Available MCP Tools:");
    for tool_name in agent.get_available_tools() {
        println!("  - {}", tool_name);
    }
    
    println!("\n🎉 Example completed successfully!");
    println!("📊 Test project will be cleaned up automatically.");
    
    Ok(())
}

async fn create_test_rust_project(project_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    
    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
"#;
    
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create src directory
    fs::create_dir_all(project_path.join("src"))?;
    
    // Create main.rs with some code to analyze
    let main_rs = r#"use tokio;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    
    // Some test code for analysis
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    
    println!("Sum: {}", sum);
    
    // This will trigger some clippy warnings
    let mut x = 5;
    x = x + 1;
    println!("x = {}", x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        let numbers = vec![1, 2, 3];
        assert_eq!(numbers.iter().sum::<i32>(), 6);
    }
}
"#;
    
    fs::write(project_path.join("src/main.rs"), main_rs)?;
    
    Ok(())
}