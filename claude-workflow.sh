#!/bin/bash
# Claude工作流执行脚本

set -e  # 遇到错误时退出

echo "🚀 开始Claude工作流..."

# 代码检查
echo "📝 代码检查..."
cargo check
cargo clippy
cargo fmt --check

# 运行测试
echo "🧪 运行测试..."
cargo test --lib

# 构建项目
echo "🔨 构建项目..."
cargo build --release

# 性能验证
echo "⚡ 性能验证..."
./target/release/img-squeeze --help
./target/release/img-squeeze --version

echo "✅ 工作流完成！"