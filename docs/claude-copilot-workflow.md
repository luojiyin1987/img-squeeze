# Claude Code + Copilot PR 工作流

这个工作流将Claude Code的架构分析能力与GitHub Copilot的代码审查能力相结合，为你的Rust项目提供全面的PR审查体验。

## 🚀 功能特性

- **自动化代码分析**: 使用Claude Code进行架构、安全性和性能分析
- **智能Copilot集成**: 自动调用GitHub Copilot进行详细的代码审查
- **多维度检查**: 覆盖代码质量、安全性、测试覆盖率、依赖管理
- **实时反馈**: 在PR页面提供实时的审查建议和改进意见
- **综合报告**: 生成详细的审查报告和后续步骤指导

## 📋 工作流程

### 1. 自动触发
- PR创建时自动启动
- PR更新时重新运行
- PR批准后进行最终检查

### 2. Claude Code分析阶段
```bash
# 代码质量检查
cargo clippy -- -D warnings
cargo fmt --check

# 安全审计
cargo audit

# 测试执行
cargo test --lib

# 依赖检查
cargo outdated
```

### 3. Copilot审查阶段
自动在PR评论中调用`@copilot`：
```
@copilot 请对这个PR进行全面的代码审查，重点关注：
1. 代码质量和Rust最佳实践
2. 安全性和性能优化
3. 测试覆盖和文档完整性
4. 项目特定的功能要求
```

### 4. 综合报告生成
- 汇总所有分析结果
- 提供明确的后续步骤
- 标记审查状态和进度

## 🛠️ 使用方法

### 方法1: GitHub Actions (推荐)

1. **启用工作流**
   - 将`.github/workflows/claude-copilot-review.yml`添加到你的仓库
   - 确保`GITHUB_TOKEN`有足够的权限

2. **创建PR**
   - 正常创建PR，工作流会自动运行
   - 在PR页面查看Claude Code分析和Copilot审查结果

### 方法2: 手动脚本

1. **运行脚本**
   ```bash
   ./scripts/copilot-pr-review.sh <PR_NUMBER>
   ```

2. **查看结果**
   - 脚本会自动在PR页面添加评论
   - 生成详细的审查报告文件

## 📊 输出示例

### Claude Code分析评论
```
## 🧠 Claude Code 架构分析

### 📊 代码质量评估
- **代码结构**: 符合Rust最佳实践
- **错误处理**: 完善的thiserror集成
- **性能**: Rayon并行处理优化良好
- **安全性**: 路径验证和文件大小检查到位

### 🔧 建议改进
1. **内存管理**: 考虑为大文件添加流式处理
2. **测试覆盖**: 增加集成测试用例
3. **文档**: 添加更多内联文档
```

### Copilot审查请求
```
@copilot 请对这个PR进行全面的代码审查，重点关注：

1. **代码质量**: Rust代码风格和最佳实践
2. **性能优化**: 图片处理算法的效率
3. **错误处理**: 是否所有错误情况都被妥善处理
4. **安全性**: 文件处理和路径安全
5. **测试覆盖**: 是否需要额外的测试用例
6. **文档**: API文档和注释的完整性

请提供具体的改进建议和代码示例。
```

## 🔧 配置选项

### 环境变量
```bash
# GitHub Token (必需)
GITHUB_TOKEN=your_token

# 仓库设置 (可选)
GITHUB_REPOSITORY=owner/repo

# 自定义设置
CLADE_CODE_ANALYSIS_DEPTH=deep
COPILOT_REVIEW_FOCUS=security,performance
```

### 工作流配置
```yaml
# 在 .github/workflows/claude-copilot-review.yml 中
env:
  # 分析深度 (basic|deep|comprehensive)
  ANALYSIS_DEPTH: 'deep'
  
  # 审查重点 (security|performance|quality|all)
  REVIEW_FOCUS: 'all'
  
  # 是否运行测试
  RUN_TESTS: 'true'
  
  # 是否检查依赖
  CHECK_DEPENDENCIES: 'true'
```

## 🎯 最佳实践

### 1. 定期维护
- 更新工作流配置
- 维护依赖版本
- 优化分析脚本

### 2. 团队协作
- 确保所有开发者了解工作流
- 根据团队需求调整审查重点
- 建立响应Copilot建议的流程

### 3. 质量保证
- 不要完全依赖自动化审查
- 定期手动验证审查结果
- 根据项目特点调整审查标准

## 🐛 故障排除

### 常见问题

1. **权限不足**
   ```bash
   # 确保GITHUB_TOKEN有pr:write权限
   # 在仓库设置中启用Actions
   ```

2. **Copilot无响应**
   ```bash
   # 检查仓库是否启用了GitHub Copilot
   # 确保PR作者有Copilot访问权限
   ```

3. **分析失败**
   ```bash
   # 检查Rust环境配置
   # 确保所有依赖都正确安装
   ```

### 调试技巧
```bash
# 手动运行脚本
./scripts/copilot-pr-review.sh <PR_NUMBER>

# 检查工作流日志
gh run list

# 查看特定工作流运行
gh run view <run-id>
```

## 📈 扩展功能

### 自定义分析步骤
```yaml
- name: Custom analysis
  run: |
    # 添加自定义分析脚本
    ./scripts/custom-analysis.sh
    
    # 生成自定义报告
    ./scripts/generate-report.sh
```

### 集成其他工具
```yaml
- name: Security scan
  uses: securecodewarrior/github-action-add-sarif@v1
  
- name: Performance test
  run: |
    cargo bench
```

### 通知集成
```yaml
- name: Notify team
  run: |
    # Slack通知
    curl -X POST -H 'Content-type: application/json' \
      --data '{"text":"PR review completed"}' \
      ${{ secrets.SLACK_WEBHOOK }}
```

## 🤝 贡献

欢迎贡献改进建议和新的分析步骤！请遵循以下步骤：

1. Fork本仓库
2. 创建功能分支
3. 提交你的改进
4. 创建Pull Request

## 📄 许可证

本项目采用与主项目相同的许可证。

---

**注意**: 此工作流需要你的仓库启用了GitHub Copilot功能。如果没有启用，Copilot评论部分将不会工作，但Claude Code分析部分仍然可以正常运行。