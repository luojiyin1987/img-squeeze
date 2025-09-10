use claude_agent::ClaudeAgent;
use tempfile::TempDir;
use std::fs;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🤖 Claude Code Agent System - PR Review Example");
    println!("=" .repeat(50));
    
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();
    
    // Create a Rust project with some issues for PR review
    create_test_rust_project_with_issues(project_path).await?;
    
    println!("📁 Created test project with issues at: {:?}", project_path);
    
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
                "settings": {
                    "strict_mode": true
                },
                "dependencies": []
            },
            "github_integration": {
                "name": "github_integration",
                "version": "1.0.0",
                "description": "GitHub integration agent",
                "enabled": true,
                "settings": {
                    "dry_run": true,
                    "include_copilot": true
                },
                "dependencies": []
            }
        },
        "workflows": {
            "pr_review": {
                "name": "pr_review",
                "version": "1.0.0",
                "description": "PR review workflow",
                "enabled": true,
                "triggers": [
                    {
                        "type": "pull_request",
                        "config": {}
                    }
                ],
                "steps": [
                    {
                        "name": "project_detection",
                        "agent": "project_detector",
                        "input": {
                            "project_path": "."
                        },
                        "dependencies": [],
                        "timeout_seconds": 30
                    },
                    {
                        "name": "code_analysis",
                        "agent": "code_analysis",
                        "input": {
                            "analysis_type": "lint"
                        },
                        "dependencies": ["project_detection"],
                        "timeout_seconds": 300
                    },
                    {
                        "name": "github_review",
                        "agent": "github_integration",
                        "input": {
                            "action": "create_pr_review",
                            "dry_run": true
                        },
                        "dependencies": ["code_analysis"],
                        "timeout_seconds": 120
                    }
                ],
                "variables": {}
            }
        },
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
    
    // Simulate PR review
    let pr_number = 42;
    println!("\n🔄 Simulating PR review for PR #{}", pr_number);
    
    // Execute PR review workflow
    match agent.execute_pr_review_workflow(project_path, pr_number).await {
        Ok(result) => {
            println!("✅ PR review workflow completed!");
            println!("  Success: {}", result.success);
            println!("  Execution time: {:?}", result.execution_time);
            println!("  Outputs: {} items", result.outputs.len());
            
            if !result.logs.is_empty() {
                println!("\n📝 Execution Logs:");
                for log in result.logs {
                    let level_icon = match log.level {
                        claude_agent::LogLevel::Debug => "🔍",
                        claude_agent::LogLevel::Info => "ℹ️",
                        claude_agent::LogLevel::Warn => "⚠️",
                        claude_agent::LogLevel::Error => "❌",
                    };
                    println!("  {} [{}]: {}", level_icon, log.timestamp.format("%H:%M:%S"), log.message);
                }
            }
            
            // Show results
            if !result.outputs.is_empty() {
                println!("\n📊 Results:");
                for (key, value) in &result.outputs {
                    println!("  {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            println!("❌ PR review workflow failed: {}", e);
        }
    }
    
    // Show available agents and their capabilities
    println!("\n🤖 Available Agents and Capabilities:");
    for agent_name in agent.registry.list_agents() {
        if let Some(agent) = agent.registry.get_agent(agent_name) {
            println!("  📋 {}", agent.name());
            println!("     Version: {}", agent.version());
            println!("     Description: {}", agent.description());
            println!("     Capabilities: {:?}", agent.capabilities());
            println!("     Supported Types: {:?}", agent.supported_project_types());
            println!();
        }
    }
    
    // Demonstrate direct agent usage
    println!("🔬 Demonstrating direct code analysis...");
    
    // First, detect the project
    let project_info = agent.detect_project(project_path).await?;
    println!("  Project type: {:?}", project_info.project_type);
    
    // Create a code analysis input
    let analysis_input = claude_agent::CodeAnalysisInput {
        project_path: project_path.to_path_buf(),
        project_info: project_info.clone(),
        analysis_type: claude_agent::AnalysisType::Lint,
        options: claude_agent::AnalysisOptions {
            strict_mode: true,
            include_tests: true,
            output_format: claude_agent::OutputFormat::Json,
            custom_rules: vec![
                "avoid_clone_on_copy".to_string(),
                "needless_return".to_string()
            ],
        },
    };
    
    // Execute code analysis directly
    if let Some(analysis_agent) = agent.registry.get_agent("code_analysis") {
        let input_json = serde_json::to_value(analysis_input)?;
        match analysis_agent.execute(input_json).await {
            Ok(result_json) => {
                let result: claude_agent::CodeAnalysisResult = serde_json::from_value(result_json)?;
                println!("  ✅ Direct analysis completed!");
                println!("  📊 Issues found: {}", result.issues.len());
                println!("  📈 Success: {}", result.success);
                
                if !result.issues.is_empty() {
                    println!("  🚨 Top Issues:");
                    for (i, issue) in result.issues.iter().take(3).enumerate() {
                        let severity_icon = match issue.severity {
                            claude_agent::IssueSeverity::Error => "🔴",
                            claude_agent::IssueSeverity::Warning => "🟡",
                            claude_agent::IssueSeverity::Info => "🔵",
                            claude_agent::IssueSeverity::Style => "🟣",
                        };
                        println!("    {}. {} {}:{} - {}", 
                            i + 1, severity_icon,
                            issue.file.display(),
                            issue.line.unwrap_or(0),
                            issue.message
                        );
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Direct analysis failed: {}", e);
            }
        }
    }
    
    // Demonstrate GitHub integration (dry run)
    println!("\n🔗 Demonstrating GitHub integration (dry run)...");
    
    let github_input = claude_agent::GitHubIntegrationInput {
        project_path: project_path.to_path_buf(),
        action: claude_agent::GitHubAction::PostPRComment,
        analysis_results: None, // Would normally include analysis results
        pr_number: Some(pr_number),
        options: claude_agent::GitHubOptions {
            dry_run: true,
            include_copilot: true,
            comment_template: Some("🤖 **Claude Code Analysis**\n\nThis is a test comment from the Claude Code Agent system.".to_string()),
            ..Default::default()
        },
    };
    
    if let Some(github_agent) = agent.registry.get_agent("github_integration") {
        let input_json = serde_json::to_value(github_input)?;
        match github_agent.execute(input_json).await {
            Ok(result_json) => {
                let result: claude_agent::GitHubIntegrationResult = serde_json::from_value(result_json)?;
                println!("  ✅ GitHub integration completed!");
                println!("  📝 Action: {:?}", result.action);
                println!("  💬 Message: {}", result.message);
                println!("  🔗 PR URL: {:?}", result.comment_url);
            }
            Err(e) => {
                println!("  ❌ GitHub integration failed: {}", e);
            }
        }
    }
    
    println!("\n🎉 PR review example completed successfully!");
    println!("💡 This demonstrates how the Claude Code Agent system can:");
    println!("   • Detect project types automatically");
    println!("   • Execute comprehensive code analysis");
    println!("   • Generate PR review workflows");
    println!("   • Integrate with GitHub (including Copilot)");
    println!("   • Provide detailed feedback and suggestions");
    
    Ok(())
}

async fn create_test_rust_project_with_issues(project_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    
    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "test-project-with-issues"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
"#;
    
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create src directory
    fs::create_dir_all(project_path.join("src"))?;
    
    // Create main.rs with intentional issues for analysis
    let main_rs = r#"use tokio;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: u64,
    user_id: u64,
    amount: f64,
    items: Vec<String>,
}

// Function with some issues that linters would catch
fn calculate_total(orders: &Vec<Order>) -> f64 {
    let mut total = 0.0;
    
    // This will trigger a warning about iterating by value
    for order in orders {
        total += order.amount;
    }
    
    // Unnecessary return statement
    return total;
}

// Function with unused variable
fn process_user(user: User) -> String {
    let unused_variable = "This is unused"; // This will trigger a warning
    format!("Processed user: {}", user.name)
}

// Function that could be more efficient
fn get_adult_users(users: &Vec<User>) -> Vec<User> {
    let mut adults = Vec::new();
    
    // This creates unnecessary clones
    for user in users {
        if user.age >= 18 {
            adults.push(user.clone());
        }
    }
    
    adults
}

#[tokio::main]
async fn main() {
    println!("Starting test application...");
    
    // Create some test data
    let users = vec![
        User {
            name: "Alice".to_string(),
            age: 25,
            email: "alice@example.com".to_string(),
        },
        User {
            name: "Bob".to_string(),
            age: 17,
            email: "bob@example.com".to_string(),
        },
    ];
    
    let orders = vec![
        Order {
            id: 1,
            user_id: 1,
            amount: 29.99,
            items: vec!["item1".to_string(), "item2".to_string()],
        },
        Order {
            id: 2,
            user_id: 1,
            amount: 15.50,
            items: vec!["item3".to_string()],
        },
    ];
    
    // Process data
    let total = calculate_total(&orders);
    println!("Total amount: ${:.2}", total);
    
    let adult_users = get_adult_users(&users);
    println!("Adult users: {}", adult_users.len());
    
    for user in users {
        let processed = process_user(user);
        println!("{}", processed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total() {
        let orders = vec![
            Order {
                id: 1,
                user_id: 1,
                amount: 10.0,
                items: vec!["test".to_string()],
            },
        ];
        
        assert_eq!(calculate_total(&orders), 10.0);
    }
    
    #[test]
    fn test_get_adult_users() {
        let users = vec![
            User {
                name: "Adult".to_string(),
                age: 25,
                email: "adult@example.com".to_string(),
            },
            User {
                name: "Minor".to_string(),
                age: 16,
                email: "minor@example.com".to_string(),
            },
        ];
        
        let adults = get_adult_users(&users);
        assert_eq!(adults.len(), 1);
        assert_eq!(adults[0].name, "Adult");
    }
}
"#;
    
    fs::write(project_path.join("src/main.rs"), main_rs)?;
    
    Ok(())
}