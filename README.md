# img-squeeze

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rustlang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-blue.svg)](https://crates.io/)

一个用 Rust 编写的快速、高效的图片压缩工具，支持多种图片格式和质量调整。

## ✨ 特性

- 🖼️ **多格式支持** - 支持 JPEG、PNG、WebP 格式
- 🎯 **质量调整** - 可自定义压缩质量 (1-100)
- 📏 **尺寸调整** - 可调整图片宽度和高度
- 📊 **压缩统计** - 显示压缩前后文件大小对比
- 🚀 **快速处理** - 基于 Rust 的高性能处理
- 🎨 **友好界面** - 清晰的进度提示和错误信息

## 📦 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/img-squeeze.git
cd img-squeeze

# 构建项目
cargo build --release

# 将二进制文件添加到 PATH
sudo cp target/release/img-squeeze /usr/local/bin/
```

### 使用 Cargo 安装

```bash
cargo install img-squeeze
```

## 🚀 使用方法

### 基本压缩

```bash
# 基本压缩
img-squeeze compress input.jpg output.jpg

# 查看帮助
img-squeeze --help
img-squeeze compress --help
```

### 高级选项

```bash
# 指定压缩质量 (1-100, 默认 80)
img-squeeze compress input.jpg output.jpg -q 90

# 调整图片尺寸
img-squeeze compress input.jpg output.jpg -w 800        # 设置宽度为 800px
img-squeeze compress input.jpg output.jpg -H 600        # 设置高度为 600px
img-squeeze compress input.jpg output.jpg -w 800 -H 600 # 同时设置宽度和高度

# 指定输出格式
img-squeeze compress input.png output.jpg -f jpeg
img-squeeze compress input.jpg output.webp -f webp
```

### 查看图片信息

```bash
# 查看图片详细信息
img-squeeze info image.jpg
```

输出示例：
```
📋 Getting info for: "image.jpg"
📸 Image Information:
  📏 Dimensions: 1920x1080
  🎨 Color type: Rgb8
  💾 Format: Jpeg
  📊 File size: 2,456,789 bytes
  📈 Megapixels: 2.1
```

### 批量处理示例

```bash
# 使用 shell 脚本批量压缩
for file in *.jpg; do
    img-squeeze compress "$file" "compressed_$file" -q 85 -w 1200
done
```

## 📖 命令详解

### compress 命令

压缩图片文件。

**参数：**
- `INPUT` - 输入图片文件路径
- `OUTPUT` - 输出图片文件路径

**选项：**
- `-q, --quality <QUALITY>` - 压缩质量 (1-100)，默认 80
- `-w, --width <WIDTH>` - 最大宽度（像素）
- `-H, --height <HEIGHT>` - 最大高度（像素）
- `-f, --format <FORMAT>` - 输出格式 (jpeg, png, webp)

### info 命令

显示图片的详细信息。

**参数：**
- `INPUT` - 要分析的图片文件路径

## 🛠️ 开发

### 环境要求

- Rust 1.70+
- Cargo

### 构建项目

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 检查代码
cargo check
cargo clippy
```

### 项目结构

```
img-squeeze/
├── src/
│   └── main.rs          # 主程序入口
├── Cargo.toml           # 项目配置
├── LICENSE              # MIT 许可证
├── README.md            # 项目说明
├── .gitignore           # Git 忽略文件
└── CLAUDE.md            # Claude Code 开发指南
```

## 📊 性能特点

- **内存效率** - 使用 Rust 的零成本抽象和内存安全
- **处理速度** - 基于高性能的 `image` 库
- **并行处理** - 支持多线程图片处理（未来版本）
- **流式处理** - 大文件的流式处理（未来版本）

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [Rust](https://rustlang.org/) - 强大的系统编程语言
- [image](https://github.com/image-rs/image) - Rust 图片处理库
- [clap](https://github.com/clap-rs/clap) - 命令行参数解析库
- [indicatif](https://github.com/console-rs/indicatif) - 进度条库

## 📞 支持

如果您遇到问题或有建议，请：

1. 查看 [Issues](https://github.com/yourusername/img-squeeze/issues)
2. 创建新的 Issue
3. 发送邮件至：your.email@example.com

---

**注意**：这是一个开源项目，欢迎任何形式的贡献和反馈！