use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub triggers: Vec<WorkflowTriggerConfig>,
    pub steps: Vec<WorkflowStepConfig>,
    pub variables: HashMap<String, WorkflowVariableConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTriggerConfig {
    pub r#type: String,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepConfig {
    pub name: String,
    pub agent: String,
    pub input: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<String>,
    pub timeout_seconds: u64,
    pub retry_policy: RetryPolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicyConfig {
    pub max_attempts: u32,
    pub backoff_strategy: String,
    pub max_delay_seconds: u64,
}

impl Default for RetryPolicyConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_strategy: "exponential".to_string(),
            max_delay_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVariableConfig {
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub version: String,
    pub agents: HashMap<String, AgentConfig>,
    pub workflows: HashMap<String, WorkflowConfig>,
    pub global_settings: HashMap<String, serde_json::Value>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            agents: HashMap::new(),
            workflows: HashMap::new(),
            global_settings: HashMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct ConfigManager {
    config_path: PathBuf,
    config: GlobalConfig,
}

impl ConfigManager {
    pub fn new<P: AsRef<std::path::Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            config: GlobalConfig::default(),
        }
    }
    
    pub async fn load_config(&mut self) -> Result<(), ConfigError> {
        if !self.config_path.exists() {
            warn!("Config file not found: {:?}, using defaults", self.config_path);
            return Ok(());
        }
        
        let content = fs::read_to_string(&self.config_path).await?;
        self.config = serde_json::from_str(&content)?;
        
        info!("Loaded configuration from: {:?}", self.config_path);
        Ok(())
    }
    
    pub async fn save_config(&self) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content).await?;
        
        info!("Saved configuration to: {:?}", self.config_path);
        Ok(())
    }
    
    pub fn get_config(&self) -> &GlobalConfig {
        &self.config
    }
    
    pub fn get_config_mut(&mut self) -> &mut GlobalConfig {
        &mut self.config
    }
    
    pub fn get_agent_config(&self, name: &str) -> Option<&AgentConfig> {
        self.config.agents.get(name)
    }
    
    pub fn get_workflow_config(&self, name: &str) -> Option<&WorkflowConfig> {
        self.config.workflows.get(name)
    }
    
    pub fn add_agent_config(&mut self, config: AgentConfig) {
        self.config.agents.insert(config.name.clone(), config);
    }
    
    pub fn add_workflow_config(&mut self, config: WorkflowConfig) {
        self.config.workflows.insert(config.name.clone(), config);
    }
    
    pub fn remove_agent_config(&mut self, name: &str) -> Option<AgentConfig> {
        self.config.agents.remove(name)
    }
    
    pub fn remove_workflow_config(&mut self, name: &str) -> Option<WorkflowConfig> {
        self.config.workflows.remove(name)
    }
    
    pub fn validate_config(&self) -> Result<(), ConfigError> {
        // Validate agent configurations
        for (name, agent) in &self.config.agents {
            if agent.name.is_empty() {
                return Err(ConfigError::ValidationError(format!("Agent name cannot be empty: {}", name)));
            }
            if agent.version.is_empty() {
                return Err(ConfigError::ValidationError(format!("Agent version cannot be empty: {}", name)));
            }
        }
        
        // Validate workflow configurations
        for (name, workflow) in &self.config.workflows {
            if workflow.name.is_empty() {
                return Err(ConfigError::ValidationError(format!("Workflow name cannot be empty: {}", name)));
            }
            if workflow.steps.is_empty() {
                return Err(ConfigError::ValidationError(format!("Workflow must have at least one step: {}", name)));
            }
            
            // Validate step dependencies
            for step in &workflow.steps {
                for dep in &step.dependencies {
                    if !workflow.steps.iter().any(|s| s.name == *dep) {
                        return Err(ConfigError::ValidationError(format!("Invalid dependency '{}' in step '{}' of workflow '{}': step not found", dep, step.name, name)));
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_config_manager() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_path_buf();
        
        let test_config = GlobalConfig {
            version: "1.0.0".to_string(),
            agents: {
                let mut agents = HashMap::new();
                agents.insert("test_agent".to_string(), AgentConfig {
                    name: "test_agent".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Test agent".to_string(),
                    enabled: true,
                    settings: HashMap::new(),
                    dependencies: Vec::new(),
                });
                agents
            },
            workflows: HashMap::new(),
            global_settings: HashMap::new(),
        };
        
        let config_json = serde_json::to_string_pretty(&test_config).unwrap();
        
        // Write to a separate file since temp_file will be dropped
        tokio::fs::write(&config_path, config_json.as_bytes()).await.unwrap();
        
        let mut manager = ConfigManager::new(&config_path);
        manager.load_config().await.unwrap();
        
        assert_eq!(manager.get_config().version, "1.0.0");
        assert!(manager.get_agent_config("test_agent").is_some());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = GlobalConfig::default();
        
        // Add valid agent
        config.agents.insert("test_agent".to_string(), AgentConfig {
            name: "test_agent".to_string(),
            version: "1.0.0".to_string(),
            description: "Test agent".to_string(),
            enabled: true,
            settings: HashMap::new(),
            dependencies: Vec::new(),
        });
        
        let manager = ConfigManager::new("test.json");
        assert!(manager.validate_config().is_ok());
    }
    
    #[test]
    fn test_config_validation_empty_name() {
        let mut config = GlobalConfig::default();
        
        // Add agent with empty name
        config.agents.insert("test_agent".to_string(), AgentConfig {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            description: "Test agent".to_string(),
            enabled: true,
            settings: HashMap::new(),
            dependencies: Vec::new(),
        });
        
        let manager = ConfigManager::new("test.json");
        assert!(manager.validate_config().is_err());
    }
}