use crate::core::{ProjectType, ProjectInfo, Dependency, DependencySource, AgentError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use async_trait::async_trait;
use tokio::fs as async_fs;
use walkdir::WalkDir;
use glob::Pattern;

// 项目检测器 Agent
pub struct ProjectDetector {
    project_path: PathBuf,
    config_files: Vec<PathBuf>,
    dependencies: Vec<Dependency>,
}

impl ProjectDetector {
    pub fn new<P: AsRef<Path>>(project_path: P) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
            config_files: Vec::new(),
            dependencies: Vec::new(),
        }
    }
    
    pub async fn detect_project<P: AsRef<Path>>(project_path: P) -> Result<ProjectInfo> {
        let mut detector = Self::new(project_path);
        
        // 扫描项目文件
        detector.scan_project().await?;
        
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
            root_path: detector.project_path,
            config_files: detector.config_files,
            dependencies: detector.dependencies,
        })
    }
    
    async fn scan_project(&mut self) -> Result<()> {
        let mut config_files = Vec::new();
        let mut dependencies = Vec::new();
        
        // 扫描项目目录
        for entry in WalkDir::new(&self.project_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            
            // 检查配置文件
            if Self::is_config_file(file_name) {
                config_files.push(path.to_path_buf());
                
                // 解析依赖
                if let Some(deps) = self.parse_dependencies(path).await? {
                    dependencies.extend(deps);
                }
            }
        }
        
        self.config_files = config_files;
        self.dependencies = dependencies;
        
        Ok(())
    }
    
    fn is_config_file(file_name: &str) -> bool {
        matches!(file_name.to_lowercase().as_str(),
            "cargo.toml" | "package.json" | "pom.xml" | "build.gradle" |
            "requirements.txt" | "pyproject.toml" | "go.mod" | "composer.json" |
            "gemfile" | "mix.exs" | "project.clj" | "sbt.sbt" | "build.sbt" |
            "cmakelists.txt" | "meson.build" | "makefile" | "dockerfile" |
            "docker-compose.yml" | "terraform.tf" | "main.tf" | "vars.tf"
        )
    }
    
    async fn parse_dependencies(&self, config_path: &Path) -> Result<Option<Vec<Dependency>>> {
        let file_name = config_path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        match file_name.to_lowercase().as_str() {
            "cargo.toml" => self.parse_cargo_dependencies(config_path).await,
            "package.json" => self.parse_npm_dependencies(config_path).await,
            "pom.xml" => self.parse_maven_dependencies(config_path).await,
            "build.gradle" => self.parse_gradle_dependencies(config_path).await,
            "requirements.txt" => self.parse_pip_dependencies(config_path).await,
            "pyproject.toml" => self.parse_pyproject_dependencies(config_path).await,
            "go.mod" => self.parse_go_dependencies(config_path).await,
            _ => Ok(None),
        }
    }
    
    async fn parse_cargo_dependencies(&self, config_path: &Path) -> Result<Option<Vec<Dependency>>> {
        let content = async_fs::read_to_string(config_path).await?;
        let cargo_toml: toml::Value = toml::from_str(&content)
            .map_err(|e| AgentError::Configuration(e.to_string()))?;
        
        let mut dependencies = Vec::new();
        
        if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
            for (name, version) in deps {
                let version_str = match version {
                    toml::Value::String(s) => s.clone(),
                    toml::Value::Table(table) => {
                        if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                            version.to_string()
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };
                
                dependencies.push(Dependency {
                    name: name.clone(),
                    version: version_str,
                    source: DependencySource::Cargo,
                });
            }
        }
        
        Ok(Some(dependencies))
    }
    
    async fn parse_npm_dependencies(&self, config_path: &Path) -> Result<Option<Vec<Dependency>>> {
        let content = async_fs::read_to_string(config_path).await?;
        let package_json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AgentError::Configuration(e.to_string()))?;
        
        let mut dependencies = Vec::new();
        
        // 处理 dependencies
        if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    dependencies.push(Dependency {
                        name: name.clone(),
                        version: version_str.to_string(),
                        source: DependencySource::Npm,
                    });
                }
            }
        }
        
        // 处理 devDependencies
        if let Some(deps) = package_json.get("devDependencies").and_then(|d| d.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    dependencies.push(Dependency {
                        name: name.clone(),
                        version: version_str.to_string(),
                        source: DependencySource::Npm,
                    });
                }
            }
        }
        
        Ok(Some(dependencies))
    }
    
    async fn parse_maven_dependencies(&self, config_path: &Path) -> Result<Option<Vec<Dependency>>> {
        let content = async_fs::read_to_string(config_path).await?;
        
        // 简化的 XML 解析（实际项目中应该使用 proper XML 解析器）
        let mut dependencies = Vec::new();
        let mut in_dependency = false;
        let mut current_dep: HashMap<String, String> = HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.contains("<dependency>") {
                in_dependency = true;
                current_dep.clear();
            } else if line.contains("</dependency>") {
                in_dependency = false;
                
                if let (Some(group_id), Some(artifact_id), Some(version)) = (
                    current_dep.get("groupId"),
                    current_dep.get("artifactId"),
                    current_dep.get("version"),
                ) {
                    dependencies.push(Dependency {
                        name: format!("{}:{}", group_id, artifact_id),
                        version: version.clone(),
                        source: DependencySource::Maven,
                    });
                }
                
                current_dep.clear();
            } else if in_dependency {
                if let Some(start) = line.find('>') {
                    if let Some(end) = line.rfind('<') {
                        if start < end {
                            let key = line[..start].trim().trim_start_matches('<');
                            let value = line[start + 1..end].trim();
                            
                            if !key.is_empty() && !value.is_empty() {
                                current_dep.insert(key.to_string(), value.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(Some(dependencies))
    }
    
    async fn parse_gradle_dependencies(&self, config_path: &Path) -> Result<Option<Vec<Dependency>>> {
        let content = async_fs::read_to_string(config_path).await?;
        
        let mut dependencies = Vec::new();
        
        // 简化的 Groovy 解析
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("implementation ") || line.starts_with("api ") {
                if let Some(start) = line.find('\"') {
                    if let Some(end) = line.rfind('\"') {
                        if start < end {
                            let dep_spec = line[start + 1..end].trim();
                            
                            if let Some((group_artifact, version)) = dep_spec.split_once(':') {
                                dependencies.push(Dependency {
                                    name: group_artifact.to_string(),
                                    version: version.to_string(),
                                    source: DependencySource::Gradle,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(Some(dependencies))
    }
    
    async fn parse_pip_dependencies(&self, config_path: &Path) -> Result<Option<Vec<Dependency>>> {
        let content = async_fs::read_to_string(config_path).await?;
        
        let mut dependencies = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // 跳过注释和空行
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            
            // 处理各种依赖格式
            if let Some(index) = line.find("==") {
                let name = line[..index].trim();
                let version = line[index + 2..].trim();
                
                if !name.is_empty() && !version.is_empty() {
                    dependencies.push(Dependency {
                        name: name.to_string(),
                        version: version.to_string(),
                        source: DependencySource::Pip,
                    });
                }
            } else if let Some(index) = line.find(">=") {
                let name = line[..index].trim();
                let version = line[index + 2..].trim();
                
                if !name.is_empty() && !version.is_empty() {
                    dependencies.push(Dependency {
                        name: name.to_string(),
                        version: version.to_string(),
                        source: DependencySource::Pip,
                    });
                }
            } else if let Some(index) = line.find('~') {
                let name = line[..index].trim();
                let version = line[index + 1..].trim();
                
                if !name.is_empty() && !version.is_empty() {
                    dependencies.push(Dependency {
                        name: name.to_string(),
                        version: version.to_string(),
                        source: DependencySource::Pip,
                    });
                }
            } else if !line.is_empty() {
                // 假设是最新版本
                dependencies.push(Dependency {
                    name: line.to_string(),
                    version: "latest".to_string(),
                    source: DependencySource::Pip,
                });
            }
        }
        
        Ok(Some(dependencies))
    }
    
    async fn parse_pyproject_dependencies(&self, config_path: &Path) -> Result<Option<Vec<Dependency>>> {
        let content = async_fs::read_to_string(config_path).await?;
        let pyproject_toml: toml::Value = toml::from_str(&content)
            .map_err(|e| AgentError::Configuration(e.to_string()))?;
        
        let mut dependencies = Vec::new();
        
        // 检查不同的依赖配置
        let sections = ["project.dependencies", "tool.poetry.dependencies", "build-system.requires"];
        
        for section in &sections {
            if let Some(deps) = pyproject_toml.get(section).and_then(|d| d.as_array()) {
                for dep in deps {
                    if let Some(dep_str) = dep.as_str() {
                        if let Some((name, version)) = self.parse_python_dependency(dep_str) {
                            dependencies.push(Dependency {
                                name,
                                version,
                                source: DependencySource::Pip,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(Some(dependencies))
    }
    
    fn parse_python_dependency(&self, dep_str: &str) -> Option<(String, String)> {
        // 解析 Python 依赖格式，如 "requests>=2.0.0", "django~=4.0", "numpy"
        if let Some(index) = dep_str.find(">=") {
            let name = dep_str[..index].trim();
            let version = dep_str[index + 2..].trim();
            Some((name.to_string(), version.to_string()))
        } else if let Some(index) = dep_str.find("==") {
            let name = dep_str[..index].trim();
            let version = dep_str[index + 2..].trim();
            Some((name.to_string(), version.to_string()))
        } else if let Some(index) = dep_str.find("~=>") {
            let name = dep_str[..index].trim();
            let version = dep_str[index + 2..].trim();
            Some((name.to_string(), version.to_string()))
        } else if let Some(index) = dep_str.find('~') {
            let name = dep_str[..index].trim();
            let version = dep_str[index + 1..].trim();
            Some((name.to_string(), version.to_string()))
        } else if let Some(index) = dep_str.find('>') {
            let name = dep_str[..index].trim();
            let version = dep_str[index + 1..].trim();
            Some((name.to_string(), version.to_string()))
        } else if let Some(index) = dep_str.find('<') {
            let name = dep_str[..index].trim();
            let version = dep_str[index + 1..].trim();
            Some((name.to_string(), version.to_string()))
        } else if !dep_str.is_empty() {
            // 假设是最新版本
            Some((dep_str.to_string(), "latest".to_string()))
        } else {
            None
        }
    }
    
    async fn parse_go_dependencies(&self, config_path: &Path) -> Result<Option<Vec<Dependency>>> {
        let content = async_fs::read_to_string(config_path).await?;
        
        let mut dependencies = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("require ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[1];
                    let version = parts[2];
                    
                    dependencies.push(Dependency {
                        name: name.to_string(),
                        version: version.to_string(),
                        source: DependencySource::GoMod,
                    });
                }
            }
        }
        
        Ok(Some(dependencies))
    }
    
    async fn detect_project_type(&self) -> Result<ProjectType> {
        // 检查 Rust 项目
        if self.has_file("Cargo.toml") {
            return Ok(ProjectType::Rust);
        }
        
        // 检查 Node.js 项目
        if self.has_file("package.json") {
            return self.detect_frontend_framework().await;
        }
        
        // 检查 Python 项目
        if self.has_file("requirements.txt") || self.has_file("pyproject.toml") {
            return Ok(ProjectType::Python);
        }
        
        // 检查 Java 项目
        if self.has_file("pom.xml") || self.has_file("build.gradle") {
            return Ok(ProjectType::Java);
        }
        
        // 检查 Go 项目
        if self.has_file("go.mod") {
            return Ok(ProjectType::Go);
        }
        
        // 检查 Terraform 项目
        if self.has_file("main.tf") || self.has_file("terraform.tf") {
            return Ok(ProjectType::Terraform);
        }
        
        // 检查 Docker 项目
        if self.has_file("Dockerfile") || self.has_file("docker-compose.yml") {
            return Ok(ProjectType::Docker);
        }
        
        Ok(ProjectType::Generic)
    }
    
    async fn detect_frontend_framework(&self) -> Result<ProjectType> {
        if let Some(package_json_path) = self.find_file("package.json") {
            let content = async_fs::read_to_string(package_json_path).await?;
            let package_json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| AgentError::Configuration(e.to_string()))?;
            
            // 检查依赖以确定框架
            let default_deps = serde_json::Map::new();
            let dependencies = package_json.get("dependencies")
                .and_then(|d| d.as_object())
                .unwrap_or(&default_deps);
            
            let dev_dependencies = package_json.get("devDependencies")
                .and_then(|d| d.as_object())
                .unwrap_or(&default_deps);
            
            // 检查 React
            if dependencies.contains_key("react") || dev_dependencies.contains_key("react") {
                return Ok(ProjectType::React);
            }
            
            // 检查 Vue
            if dependencies.contains_key("vue") || dev_dependencies.contains_key("vue") {
                return Ok(ProjectType::Vue);
            }
            
            // 检查 Angular
            if dependencies.contains_key("@angular/core") || dev_dependencies.contains_key("@angular/core") {
                return Ok(ProjectType::Angular);
            }
        }
        
        Ok(ProjectType::Nodejs)
    }
    
    async fn detect_language(&self) -> Result<String> {
        match self.detect_project_type().await? {
            ProjectType::Rust => Ok("rust".to_string()),
            ProjectType::Nodejs | ProjectType::React | ProjectType::Vue | ProjectType::Angular => {
                Ok("javascript".to_string())
            }
            ProjectType::Python => Ok("python".to_string()),
            ProjectType::Java => Ok("java".to_string()),
            ProjectType::Go => Ok("go".to_string()),
            ProjectType::Terraform => Ok("hcl".to_string()),
            ProjectType::Docker => Ok("dockerfile".to_string()),
            ProjectType::Generic => {
                // 基于文件扩展名推断
                self.detect_language_from_files().await
            }
        }
    }
    
    async fn detect_language_from_files(&self) -> Result<String> {
        let mut language_counts = HashMap::new();
        
        for entry in WalkDir::new(&self.project_path)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Some(extension) = entry.path().extension() {
                if let Some(ext_str) = extension.to_str() {
                    let language = match ext_str {
                        "rs" => "rust",
                        "js" | "jsx" => "javascript",
                        "ts" | "tsx" => "typescript",
                        "py" => "python",
                        "java" => "java",
                        "go" => "go",
                        "rb" => "ruby",
                        "php" => "php",
                        "cs" => "csharp",
                        "cpp" | "cxx" | "cc" => "cpp",
                        "c" => "c",
                        "sh" => "shell",
                        "sql" => "sql",
                        "html" => "html",
                        "css" | "scss" | "sass" => "css",
                        _ => continue,
                    };
                    
                    *language_counts.entry(language).or_insert(0) += 1;
                }
            }
        }
        
        // 返回最常见的语言
        Ok(language_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(language, _)| language.to_string())
            .unwrap_or_else(|| "unknown".to_string()))
    }
    
    async fn detect_framework(&self) -> Result<Option<String>> {
        match self.detect_project_type().await? {
            ProjectType::React => Ok(Some("react".to_string())),
            ProjectType::Vue => Ok(Some("vue".to_string())),
            ProjectType::Angular => Ok(Some("angular".to_string())),
            ProjectType::Nodejs => self.detect_nodejs_framework().await,
            ProjectType::Python => self.detect_python_framework().await,
            ProjectType::Java => self.detect_java_framework().await,
            ProjectType::Rust => self.detect_rust_framework().await,
            _ => Ok(None),
        }
    }
    
    async fn detect_nodejs_framework(&self) -> Result<Option<String>> {
        if let Some(package_json_path) = self.find_file("package.json") {
            let content = async_fs::read_to_string(package_json_path).await?;
            let package_json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| AgentError::Configuration(e.to_string()))?;
            
            let default_deps = serde_json::Map::new();
            let dependencies = package_json.get("dependencies")
                .and_then(|d| d.as_object())
                .unwrap_or(&default_deps);
            
            let dev_dependencies = package_json.get("devDependencies")
                .and_then(|d| d.as_object())
                .unwrap_or(&default_deps);
            
            // 检查常见框架
            let frameworks = [
                ("express", "express"),
                ("next", "next"),
                ("nuxt", "nuxt"),
                ("gatsby", "gatsby"),
                ("react", "react"),
                ("vue", "vue"),
                ("@angular/core", "angular"),
                ("nestjs", "nestjs"),
                ("fastify", "fastify"),
                ("koa", "koa"),
                ("hapi", "hapi"),
            ];
            
            for (dep_name, framework_name) in &frameworks {
                if dependencies.contains_key(*dep_name) || dev_dependencies.contains_key(*dep_name) {
                    return Ok(Some(framework_name.to_string()));
                }
            }
        }
        
        Ok(None)
    }
    
    async fn detect_python_framework(&self) -> Result<Option<String>> {
        if let Some(requirements_path) = self.find_file("requirements.txt") {
            let content = async_fs::read_to_string(requirements_path).await?;
            
            let frameworks = [
                ("django", "django"),
                ("flask", "flask"),
                ("fastapi", "fastapi"),
                ("sqlalchemy", "sqlalchemy"),
                ("pyramid", "pyramid"),
                ("tornado", "tornado"),
                ("bottle", "bottle"),
                ("cherrypy", "cherrypy"),
                ("aiohttp", "aiohttp"),
                ("sanic", "sanic"),
            ];
            
            for line in content.lines() {
                let line = line.trim().to_lowercase();
                for (dep_name, framework_name) in &frameworks {
                    if line.starts_with(dep_name) {
                        return Ok(Some(framework_name.to_string()));
                    }
                }
            }
        }
        
        if let Some(pyproject_path) = self.find_file("pyproject.toml") {
            let content = async_fs::read_to_string(pyproject_path).await?;
            let pyproject_toml: toml::Value = toml::from_str(&content)
                .map_err(|e| AgentError::Configuration(e.to_string()))?;
            
            // 检查工具配置
            if pyproject_toml.get("tool.poetry").is_some() {
                return Ok(Some("poetry".to_string()));
            }
            
            if pyproject_toml.get("tool.flit").is_some() {
                return Ok(Some("flit".to_string()));
            }
            
            if pyproject_toml.get("tool.setuptools").is_some() {
                return Ok(Some("setuptools".to_string()));
            }
        }
        
        Ok(None)
    }
    
    async fn detect_java_framework(&self) -> Result<Option<String>> {
        if let Some(pom_path) = self.find_file("pom.xml") {
            let content = async_fs::read_to_string(pom_path).await?;
            
            let frameworks = [
                ("spring-boot-starter", "spring-boot"),
                ("spring-core", "spring"),
                ("jakarta.servlet-api", "jakarta-ee"),
                ("javax.servlet-api", "java-ee"),
                ("micronaut-core", "micronaut"),
                ("quarkus-core", "quarkus"),
                ("vertx-core", "vertx"),
                ("dropwizard-core", "dropwizard"),
                ("spark-core", "spark"),
                ("javalin", "javalin"),
            ];
            
            for framework in &frameworks {
                if content.to_lowercase().contains(framework.0) {
                    return Ok(Some(framework.1.to_string()));
                }
            }
        }
        
        if let Some(gradle_path) = self.find_file("build.gradle") {
            let content = async_fs::read_to_string(gradle_path).await?;
            
            let frameworks = [
                ("org.springframework.boot", "spring-boot"),
                ("org.springframework", "spring"),
                ("io.micronaut", "micronaut"),
                ("io.quarkus", "quarkus"),
                ("io.vertx", "vertx"),
                ("io.dropwizard", "dropwizard"),
                ("com.sparkjava", "spark"),
                ("io.javalin", "javalin"),
            ];
            
            for framework in &frameworks {
                if content.contains(framework.0) {
                    return Ok(Some(framework.1.to_string()));
                }
            }
        }
        
        Ok(None)
    }
    
    async fn detect_rust_framework(&self) -> Result<Option<String>> {
        if let Some(cargo_path) = self.find_file("Cargo.toml") {
            let content = async_fs::read_to_string(cargo_path).await?;
            let cargo_toml: toml::Value = toml::from_str(&content)
                .map_err(|e| AgentError::Configuration(e.to_string()))?;
            
            let dependencies = cargo_toml.get("dependencies")
                .and_then(|d| d.as_table())
                .unwrap_or(&toml::value::Table::new());
            
            let frameworks = [
                ("actix-web", "actix-web"),
                ("axum", "axum"),
                ("rocket", "rocket"),
                ("warp", "warp"),
                ("tokio", "tokio"),
                ("async-std", "async-std"),
                ("serenity", "serenity"),
                ("tera", "tera"),
                ("askama", "askama"),
                ("yew", "yew"),
                ("seed", "seed"),
            ];
            
            for (dep_name, framework_name) in &frameworks {
                if dependencies.contains_key(*dep_name) {
                    return Ok(Some(framework_name.to_string()));
                }
            }
        }
        
        Ok(None)
    }
    
    async fn detect_build_system(&self) -> Result<Option<String>> {
        match self.detect_project_type().await? {
            ProjectType::Rust => Ok(Some("cargo".to_string())),
            ProjectType::Nodejs | ProjectType::React | ProjectType::Vue | ProjectType::Angular => {
                Ok(Some("npm".to_string()))
            }
            ProjectType::Python => {
                if self.has_file("pyproject.toml") {
                    Ok(Some("pyproject".to_string()))
                } else {
                    Ok(Some("pip".to_string()))
                }
            }
            ProjectType::Java => {
                if self.has_file("pom.xml") {
                    Ok(Some("maven".to_string()))
                } else if self.has_file("build.gradle") {
                    Ok(Some("gradle".to_string()))
                } else {
                    Ok(None)
                }
            }
            ProjectType::Go => Ok(Some("go".to_string())),
            ProjectType::Terraform => Ok(Some("terraform".to_string())),
            ProjectType::Docker => Ok(Some("docker".to_string())),
            ProjectType::Generic => Ok(None),
        }
    }
    
    async fn detect_package_manager(&self) -> Result<Option<String>> {
        match self.detect_project_type().await? {
            ProjectType::Rust => Ok(Some("cargo".to_string())),
            ProjectType::Nodejs | ProjectType::React | ProjectType::Vue | ProjectType::Angular => {
                // 检查是否有 yarn.lock 或 pnpm-lock.yaml
                if self.has_file("yarn.lock") {
                    Ok(Some("yarn".to_string()))
                } else if self.has_file("pnpm-lock.yaml") {
                    Ok(Some("pnpm".to_string()))
                } else {
                    Ok(Some("npm".to_string()))
                }
            }
            ProjectType::Python => {
                if self.has_file("pyproject.toml") {
                    let content = async_fs::read_to_string(self.project_path.join("pyproject.toml")).await?;
                    let pyproject_toml: toml::Value = toml::from_str(&content)
                        .map_err(|e| AgentError::Configuration(e.to_string()))?;
                    
                    if pyproject_toml.get("tool.poetry").is_some() {
                        Ok(Some("poetry".to_string()))
                    } else if pyproject_toml.get("tool.flit").is_some() {
                        Ok(Some("flit".to_string()))
                    } else {
                        Ok(Some("pip".to_string()))
                    }
                } else {
                    Ok(Some("pip".to_string()))
                }
            }
            ProjectType::Java => {
                if self.has_file("pom.xml") {
                    Ok(Some("maven".to_string()))
                } else if self.has_file("build.gradle") {
                    Ok(Some("gradle".to_string()))
                } else {
                    Ok(None)
                }
            }
            ProjectType::Go => Ok(Some("go".to_string())),
            _ => Ok(None),
        }
    }
    
    fn has_file(&self, file_name: &str) -> bool {
        self.project_path.join(file_name).exists()
    }
    
    fn find_file(&self, file_name: &str) -> Option<PathBuf> {
        let path = self.project_path.join(file_name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;
    
    #[tokio::test]
    async fn test_detect_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
"#;
        
        let mut file = tokio::fs::File::create(&cargo_toml_path).await.unwrap();
        file.write_all(cargo_toml_content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        
        let project_info = ProjectDetector::detect_project(temp_dir.path()).await.unwrap();
        
        assert!(matches!(project_info.project_type, ProjectType::Rust));
        assert_eq!(project_info.language, "rust");
        assert_eq!(project_info.build_system, Some("cargo".to_string()));
        assert_eq!(project_info.package_manager, Some("cargo".to_string()));
    }
    
    #[tokio::test]
    async fn test_detect_nodejs_project() {
        let temp_dir = TempDir::new().unwrap();
        let package_json_path = temp_dir.path().join("package.json");
        
        let package_json_content = r#"
{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0",
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "jest": "^29.0.0",
    "typescript": "^4.9.0"
  }
}
"#;
        
        let mut file = tokio::fs::File::create(&package_json_path).await.unwrap();
        file.write_all(package_json_content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        
        let project_info = ProjectDetector::detect_project(temp_dir.path()).await.unwrap();
        
        assert!(matches!(project_info.project_type, ProjectType::Nodejs));
        assert_eq!(project_info.language, "javascript");
        assert_eq!(project_info.build_system, Some("npm".to_string()));
        assert_eq!(project_info.package_manager, Some("npm".to_string()));
    }
    
    #[tokio::test]
    async fn test_detect_python_project() {
        let temp_dir = TempDir::new().unwrap();
        let requirements_path = temp_dir.path().join("requirements.txt");
        
        let requirements_content = r#"
requests>=2.28.0
django==4.1.0
numpy>=1.21.0
pytest>=7.0.0
"#;
        
        let mut file = tokio::fs::File::create(&requirements_path).await.unwrap();
        file.write_all(requirements_content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        
        let project_info = ProjectDetector::detect_project(temp_dir.path()).await.unwrap();
        
        assert!(matches!(project_info.project_type, ProjectType::Python));
        assert_eq!(project_info.language, "python");
        assert_eq!(project_info.build_system, Some("pip".to_string()));
        assert_eq!(project_info.package_manager, Some("pip".to_string()));
    }
    
    #[test]
    fn test_parse_python_dependency() {
        let detector = ProjectDetector::new(".");
        
        assert_eq!(
            detector.parse_python_dependency("requests>=2.28.0"),
            Some(("requests".to_string(), "2.28.0".to_string()))
        );
        
        assert_eq!(
            detector.parse_python_dependency("django==4.1.0"),
            Some(("django".to_string(), "4.1.0".to_string()))
        );
        
        assert_eq!(
            detector.parse_python_dependency("numpy"),
            Some(("numpy".to_string(), "latest".to_string()))
        );
        
        assert_eq!(detector.parse_python_dependency("# comment"), None);
    }
}