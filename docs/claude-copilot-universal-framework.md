# Claude Code + Copilot Universal Workflow Framework

这是一个通用的PR审查工作流框架，可以适配任何项目类型和编程语言。

## 🏗️ 架构设计

### 核心组件
```
claude-code-workflows/
├── templates/           # 项目类型模板
├── analyzers/          # 代码分析器
├── integrations/       # 外部集成
├── config/            # 配置文件
└── scripts/           # 通用脚本
```

### 工作流程
1. **项目检测** → 自动识别项目类型
2. **模板选择** → 匹配最适合的审查模板
3. **配置生成** → 根据项目特点生成配置
4. **执行分析** → 运行Claude Code分析
5. **Copilot调用** → 智能调用Copilot审查
6. **报告生成** → 综合审查报告

## 🎯 支持的项目类型

### 后端项目
- **Rust**: Cargo项目，安全审计，性能优化
- **Node.js**: npm/yarn项目，依赖安全，测试覆盖
- **Python**: pip/poetry项目，类型检查，代码质量
- **Java**: Maven/Gradle项目，静态分析，单元测试
- **Go**: Go模块项目，竞态检测，基准测试

### 前端项目
- **React**: 组件审查，性能优化，可访问性
- **Vue**: 组件设计，状态管理，响应式
- **Angular**: 架构审查，模块化，依赖注入
- **Svelte**: 组件优化，状态管理，性能

### 移动端项目
- **React Native**: 组件审查，原生桥接，性能
- **Flutter**: Widget审查，状态管理，动画性能
- **iOS Swift**: 架构模式，内存管理，SwiftUI
- **Android Kotlin**: 架构组件，Kotlin协程，Jetpack

### DevOps项目
- **Terraform**: 基础设施即代码审查
- **Docker**: 容器优化，安全检查
- **Kubernetes**: YAML配置，最佳实践
- **CI/CD**: 流水线优化，安全检查

## 📋 配置文件模板

### 通用配置 (claude-workflow.yml)
```yaml
# Claude Code + Copilot 工作流配置
version: "1.0"

# 项目信息
project:
  name: "your-project"
  type: "rust|nodejs|python|java|go|react|vue|angular|terraform|docker"
  language: "rust|javascript|python|java|go|typescript|kotlin|swift|hcl"
  
# 分析配置
analysis:
  # 基础分析
  basics:
    code_quality: true
    security_scan: true
    dependency_check: true
    test_coverage: true
    
  # 高级分析
  advanced:
    performance_analysis: true
    architecture_review: true
    documentation_check: true
    best_practices: true
    
  # 自定义规则
  custom_rules:
    - name: "rust_safety"
      description: "Rust安全检查"
      enabled: true
    - name: "async_patterns"
      description: "异步模式检查"
      enabled: true

# Copilot配置
copilot:
  # 审查重点
  focus_areas:
    - "code_quality"
    - "security"
    - "performance"
    - "maintainability"
    - "testing"
  
  # 审查模板
  template: "comprehensive"
  
  # 自定义提示词
  custom_prompts:
    - "请关注内存安全"
    - "检查并发处理"
    - "验证错误处理"

# 工作流触发器
triggers:
  on_pr: true
  on_push: false
  on_schedule: false
  
# 通知配置
notifications:
  slack:
    enabled: false
    webhook: ""
  email:
    enabled: false
    recipients: []
```

## 🔧 分析器模板

### Rust分析器
```yaml
# templates/rust.yml
language: rust
tools:
  - cargo: clippy
  - cargo: fmt
  - cargo: audit
  - cargo: outdated
  - cargo: test
  - cargo: doc

copilot_focus:
  - "Rust所有权和借用检查"
  - "并发安全性"
  - "错误处理模式"
  - "性能优化"
  - "内存管理"

quality_metrics:
  - compilation_warnings
  - clippy_lints
  - security_vulnerabilities
  - test_coverage
  - documentation_completeness
```

### Node.js分析器
```yaml
# templates/nodejs.yml
language: javascript
tools:
  - npm: audit
  - npm: outdated
  - npm: test
  - eslint: check
  - prettier: check
  - typescript: build

copilot_focus:
  - "JavaScript最佳实践"
  - "异步编程模式"
  - "错误处理"
  - "安全性考虑"
  - "性能优化"

quality_metrics:
  - eslint_errors
  - security_vulnerabilities
  - test_coverage
  - dependency_health
  - typescript_errors
```

## 🤖 智能项目检测

### 检测逻辑
```bash
detect_project_type() {
    if [ -f "Cargo.toml" ]; then
        echo "rust"
    elif [ -f "package.json" ]; then
        if grep -q "react" package.json; then
            echo "react"
        elif grep -q "vue" package.json; then
            echo "vue"
        else
            echo "nodejs"
        fi
    elif [ -f "requirements.txt" ] || [ -f "pyproject.toml" ]; then
        echo "python"
    elif [ -f "pom.xml" ] || [ -f "build.gradle" ]; then
        echo "java"
    elif [ -f "go.mod" ]; then
        echo "go"
    elif [ -f "main.tf" ]; then
        echo "terraform"
    elif [ -f "Dockerfile" ]; then
        echo "docker"
    else
        echo "generic"
    fi
}
```

## 🎯 模板生成器

### 交互式配置
```bash
#!/bin/bash
# scripts/setup-workflow.sh

echo "🚀 Claude Code + Copilot 工作流设置向导"
echo "================================"

# 项目检测
PROJECT_TYPE=$(detect_project_type)
echo "📋 检测到项目类型: $PROJECT_TYPE"

# 配置选项
read -p "🔍 启用安全扫描? (y/n): " security
read -p "📊 启用性能分析? (y/n): " performance
read -p "🧪 运行测试套件? (y/n): " testing
read -p "📚 检查文档? (y/n): " documentation

# 生成配置
generate_workflow_config() {
    cat > claude-workflow.yml << EOF
version: "1.0"
project:
  name: "$(basename $(pwd))"
  type: "$PROJECT_TYPE"
  language: "$(get_language $PROJECT_TYPE)"

analysis:
  basics:
    code_quality: true
    security_scan: $security
    dependency_check: true
    test_coverage: $testing
  advanced:
    performance_analysis: $performance
    architecture_review: true
    documentation_check: $documentation
    best_practices: true

copilot:
  focus_areas:
    - "code_quality"
    - "security"
    - "performance"
  template: "comprehensive"
  
triggers:
  on_pr: true
  on_push: false
EOF
}

echo "✅ 配置文件已生成: claude-workflow.yml"
```

## 🔄 工作流执行引擎

### 通用执行脚本
```bash
#!/bin/bash
# scripts/run-analysis.sh

# 加载配置
load_config() {
    if [ -f "claude-workflow.yml" ]; then
        yaml2json < claude-workflow.yml > config.json
    else
        echo "❌ 配置文件不存在，运行 setup-workflow.sh"
        exit 1
    fi
}

# 执行分析
run_analysis() {
    echo "🔍 开始代码分析..."
    
    # 基础分析
    if [ "$(jq '.analysis.basics.code_quality' config.json)" = "true" ]; then
        run_code_quality_analysis
    fi
    
    if [ "$(jq '.analysis.basics.security_scan' config.json)" = "true" ]; then
        run_security_analysis
    fi
    
    # 高级分析
    if [ "$(jq '.analysis.advanced.performance_analysis' config.json)" = "true" ]; then
        run_performance_analysis
    fi
}

# 调用Copilot
call_copilot() {
    echo "🤖 调用GitHub Copilot..."
    
    # 生成定制化的Copilot提示词
    generate_copilot_prompt > copilot-prompt.txt
    
    # 发送评论
    gh pr comment $PR_NUMBER --body "$(cat copilot-prompt.txt)"
}

# 生成报告
generate_report() {
    echo "📊 生成综合报告..."
    
    cat > analysis-report.md << EOF
# Claude Code + Copilot 审查报告

## 📊 分析摘要
- **项目类型**: $(jq '.project.type' config.json)
- **分析时间**: $(date)
- **PR号码**: $PR_NUMBER

## 🧠 Claude Code 分析
$(cat analysis-results.md)

## 🤖 Copilot 审查
已请求Copilot进行审查，请在PR页面查看回复。

## 📈 建议和后续步骤
$(cat recommendations.txt)

---
*由Claude Code + Copilot工作流自动生成*
EOF
}
```

## 🎨 自定义模板

### 创建项目特定模板
```yaml
# templates/custom-rust-web.yml
extends: "rust"
specialization: "web-framework"

tools:
  - cargo: clippy
  - cargo: fmt
  - cargo: audit
  - cargo: test
  - cargo: doc
  
  # Web特定工具
  - warp: lint
  - axum: security-check
  - sqlx: migration-check

copilot_focus:
  - "Web API设计"
  - "数据库查询优化"
  - "请求处理性能"
  - "错误处理策略"
  - "中间件安全性"

custom_rules:
  - name: "api_consistency"
    description: "API端点一致性检查"
    enabled: true
  - name: "database_n+1"
    description: "数据库N+1查询检测"
    enabled: true
```

## 📊 报告模板

### 通用报告格式
```markdown
# 🎯 PR审查综合报告

## 📊 基本信息
- **项目**: {{project.name}}
- **类型**: {{project.type}}
- **语言**: {{project.language}}
- **PR**: #{{pr.number}}
- **作者**: {{pr.author}}

## 🧠 Claude Code 分析

### ✅ 通过项
- 代码质量检查
- 安全性扫描
- 依赖健康检查

### ⚠️ 需要关注
- 测试覆盖率: {{coverage.percent}}%
- 性能优化建议
- 文档完整性

## 🤖 Copilot 审查建议
{{copilot.suggestions}}

## 📈 改进建议
{{recommendations}}

## 🔧 后续步骤
1. 修复标记的问题
2. 更新测试用例
3. 完善文档
4. 性能优化

---
*由Claude Code + Copilot工作流生成*
```

## 🚀 部署和使用

### 1. 快速开始
```bash
# 克隆工作流模板
git clone https://github.com/claude-code/workflows.git
cd workflows

# 运行设置向导
./scripts/setup-workflow.sh

# 提交到你的项目
git add .
git commit -m "Add Claude Code + Copilot workflow"
git push
```

### 2. 自定义配置
```bash
# 编辑配置文件
nano claude-workflow.yml

# 测试工作流
./scripts/test-workflow.sh

# 部署到GitHub
./scripts/deploy.sh
```

这个通用框架可以适配任何项目类型，提供智能化的PR审查体验！