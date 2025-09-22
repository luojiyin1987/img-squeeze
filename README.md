# img-squeeze

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rustlang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-blue.svg)](https://crates.io/)
[![Parallel](https://img.shields.io/badge/parallel-rayon-green.svg)](https://github.com/rayon-rs/rayon)

一个用 Rust 编写的快速、高效的图片压缩工具，支持多线程并行处理、批量压缩、多种图片格式和去中心化存储。

## ✨ 特性

- 🖼️ **多格式支持** - 支持 JPEG、PNG、WebP、BMP、TIFF、GIF、HEIC 格式
- 🎯 **质量调整** - 可自定义压缩质量 (1-100)
- 📏 **尺寸调整** - 可调整图片宽度和高度
- 🚀 **多线程处理** - 基于 Rayon 的高性能并行处理
- 📦 **批量处理** - 支持目录批量压缩和文件通配符
- 📊 **详细统计** - 实时进度显示和性能统计
- 🔧 **灵活配置** - 自定义线程数和递归处理
- 🎨 **友好界面** - 清晰的进度提示和错误信息
- 🚀 **PNG 优化** - 使用 oxipng 库进行高级 PNG 压缩优化
- 🌐 **Walrus 上传** - 支持上传到 Walrus 去中心化存储网络
- 📈 **智能压缩** - 针对不同格式优化压缩策略，避免文件变大

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
# 基本压缩（自动线程数）
img-squeeze compress input.jpg output.jpg

# 查看帮助
img-squeeze --help
img-squeeze compress --help
img-squeeze batch --help
```

### 多线程压缩

```bash
# 指定线程数压缩
img-squeeze compress input.jpg output.jpg -j 4        # 使用 4 个线程
img-squeeze compress input.jpg output.jpg -j 8        # 使用 8 个线程

# 自动线程数（默认，根据CPU核心数）
img-squeeze compress input.jpg output.jpg
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

# AVIF 格式压缩（推荐，压缩效果最好）
img-squeeze compress input.jpg output.avif -f avif -q 50
img-squeeze compress input.png output.avif -f avif -q 60

# HEIC 格式压缩（需要启用 heic feature）
img-squeeze compress input.heic output.jpg -q 85
img-squeeze compress input.heic output.webp -f webp

# 多线程 + 高级选项组合
img-squeeze compress input.jpg output.jpg -j 6 -q 85 -w 1200 -H 800 -f webp
```

### 批量处理（新增功能）

```bash
# 批量压缩整个目录
img-squeeze batch ./images ./compressed

# 批量压缩（递归处理子目录）
img-squeeze batch ./photos ./output -r

# 批量压缩 + 线程控制
img-squeeze batch ./images ./compressed -j 8

# 批量压缩 + 质量和尺寸调整
img-squeeze batch ./images ./compressed -q 85 -w 1200 -H 800

# 批量压缩 + 格式转换
img-squeeze batch ./images ./webp_output -f webp

# 批量压缩 + AVIF 格式（推荐，压缩效果最好）
img-squeeze batch ./images ./avif_output -f avif -q 50

# 使用通配符批量处理
img-squeeze batch "*.jpg" ./compressed
img-squeeze batch "./photos/*.png" ./compressed
img-squeeze batch "*.heic" ./compressed
```

### Walrus 上传（新增功能）

警告：目前为了节约费用，验证功能。默认上传到 Walrus 的测试网。不能保证文件的存储安全。上传成功后，返回的文件名是随机的字符串，同时不包含文件扩展名。需要用户重命名，加上对应的扩展名。用户体验不好。
有空对 Walrus 提交对应 pr。

```bash
# 上传到 Walrus（默认设置）
img-squeeze upload image.jpg

# 上传到自定义 Walrus 节点
img-squeeze upload image.jpg -a https://aggregator.walrus-testnet.walrus.space -p https://publisher.walrus-testnet.walrus.space

# 上传并设置存储时长（epochs）
img-squeeze upload image.jpg -e 20        # 存储20个epoch，约20天

# 临时上传（1 epoch，约24小时后自动删除）
img-squeeze upload image.jpg -t

# 组合选项
img-squeeze upload image.jpg -a https://aggregator.walrus-testnet.walrus.space -e 15
```

**临时文件管理：**

- 使用 `-t` 标志上传临时文件，24小时后自动删除
- 适合测试、开发、临时分享等场景
- 无需手动删除，系统自动清理

上传成功后，您将获得：

- 🆔 **Blob ID** - 用于唯一标识上传的文件
- 🌐 **访问 URL** - 直接访问上传文件的链接
- 📊 **文件信息** - 文件大小和存储详情

输出示例：

```bash
📤 Uploading to Walrus: "image.jpg"
🔗 Aggregator URL: https://aggregator.walrus-testnet.walrus.space
🔗 Publisher URL: https://publisher.walrus-testnet.walrus.space
⏰ Epochs: Some(10)
✅ Upload successful!
🆔 Blob ID: 3xAm...V7n9
🌐 Access URL: https://aggregator.walrus-testnet.walrus.space/v1/blobs/3xAm...V7n9
📊 File size: 1024 bytes
💡 You can use the blob ID to retrieve the file later
```

临时上传示例：

```bash
📤 Uploading to Walrus: "temp_image.jpg"
🔗 Aggregator URL: https://aggregator.walrus-testnet.walrus.space
🔗 Publisher URL: https://publisher.walrus-testnet.walrus.space
⏰ Epochs: Some(1)
✅ Upload successful!
🆔 Blob ID: MZwz...oSE
🌐 Access URL: https://aggregator.walrus-testnet.walrus.space/v1/blobs/MZwz...oSE
⏰ Temporary file: Will expire after 1 epoch (~24 hours)
🔄 Use without -t flag for longer storage
📊 File size: 406 bytes
💡 You can use the blob ID to retrieve the file later
```

### 查看图片信息

```bash
# 查看图片详细信息
img-squeeze info image.jpg
```

输出示例：

```bash
📋 Getting info for: "image.jpg"
📸 Image Information:
  📏 Dimensions: 1920x1080
  🎨 Color type: Rgb8
  💾 Format: Jpeg
  📊 File size: 2,456,789 bytes
  📈 Megapixels: 2.1
```

### 批量处理性能统计

批量处理完成后会显示详细的性能统计：

```bash
📊 Batch Compression Summary:
  📁 Total files processed: 150
  📊 Total original size: 456,789,123 bytes
  📊 Total compressed size: 234,567,890 bytes
  🎯 Overall compression ratio: 48.6%
  ⏱️  Total time: 45.2s
  ⚡ Average speed: 3.32 files/second
```

## 🎯 格式压缩建议

根据实际测试结果，不同格式的压缩效果差异很大：

### 📊 压缩效果排名

#### 🥇 **AVIF 格式** - 最佳压缩效果
- **压缩率**: 50-85%
- **适用场景**: 追求最大压缩比，现代浏览器和设备
- **推荐质量**: 40-60
- **优势**: 现代高效的压缩算法，文件大小显著减少
- **缺点**: 较老的设备可能不支持

#### 🥈 **JPEG 格式** - 平衡选择
- **压缩率**: 2-25%
- **适用场景**: 需要广泛兼容性
- **推荐质量**: 40-50（避免重新压缩已压缩的JPG）
- **优势**: 所有设备都支持，通用性最好
- **缺点**: 压缩率相对较低

#### 🥉 **WebP 格式** - 需要修复
- **当前状态**: 实现有待优化
- **潜力**: 理论上应该有较好压缩效果
- **建议**: 暂时不推荐使用，等待版本更新

#### ❌ **PNG 格式** - 不适合压缩已压缩图片
- **压缩效果**: 文件通常变大（8-10倍）
- **原因**: PNG是无损格式，用于压缩已压缩JPG会增大文件
- **适用场景**: 仅适用于从高质量源（如RAW）转换

### 💡 实用建议

```bash
# 追求最大压缩（推荐）
img-squeeze compress input.jpg output.avif -f avif -q 50

# 平衡压缩和兼容性
img-squeeze compress input.jpg output.jpg -q 45

# 避免重新压缩已压缩的JPG
img-squeeze compress original_photo.jpg output.jpg -q 80

# 从PNG等无损格式转换到JPG
img-squeeze compress screenshot.png output.jpg -f jpg -q 85
```

### ⚠️ 注意事项

1. **避免链式压缩**: 不要重复压缩已压缩的文件
2. **质量设置**: 对于已压缩的JPG，使用较低质量（40-60）
3. **格式选择**: 根据目标平台支持的格式选择
4. **测试验证**: 重要文件压缩前先在小样本测试

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

### upload 命令

上传图片到 Walrus 去中心化存储网络。

**参数：**

- `INPUT` - 要上传的图片文件路径

**选项：**

- `-a, --aggregator-url <AGGREGATOR_URL>` - Walrus aggregator URL
- `-p, --publisher-url <PUBLISHER_URL>` - Walrus publisher URL  
- `-e, --epochs <EPOCHS>` - 存储时长（epochs）

### info 命令

显示图片的详细信息。

**参数：**

- `INPUT` - 要分析的图片文件路径

## 🛠️ 开发

### 环境要求

- Rust 1.70+
- Cargo

#### HEIC 格式支持（可选）

要启用 HEIC 格式支持，需要安装系统依赖库：

**Ubuntu/Debian:**

```bash
# 安装构建工具
sudo apt-get update
sudo apt-get install -y build-essential cmake pkg-config

# 安装 libheif 依赖
sudo apt-get install -y libde265-dev libx265-dev libaom-dev libdav1d-dev

# 如果系统版本较旧，需要从源码构建 libheif >= 1.20.0
git clone https://github.com/strukturag/libheif.git
cd libheif
mkdir build && cd build
cmake ..
make -j$(nproc)
sudo make install
sudo ldconfig
```

**macOS:**

```bash
# 使用 Homebrew 安装
brew install libheif
```

**从源码构建（推荐）：**

```bash
# 克隆并构建 libheif 1.21.0
git clone --branch v1.21.0 https://github.com/strukturag/libheif.git
cd libheif
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)
sudo make install
sudo ldconfig
```

**启用 HEIC 支持构建：**

```bash
# 构建 HEIC 支持版本
cargo build --release --features heic

# 验证 HEIC 支持
./target/release/img-squeeze compress input.heic output.jpg
```

### 构建项目

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 发布构建（启用 HEIC 支持）
cargo build --release --features heic

# 运行测试
cargo test

# 检查代码
cargo check
cargo clippy
```

### Claude 工作流

本项目集成了简化的 Claude 工作流系统，用于自动化代码质量检查和构建流程：

```bash
# 运行完整的 Claude 工作流
./claude-workflow.sh

# 或手动执行各个阶段
cargo check          # 检查编译错误
cargo clippy         # 代码质量检查
cargo fmt --check    # 代码格式检查
cargo test --lib     # 运行单元测试
cargo build --release # 构建优化版本
```

**工作流包含：**

- 📝 代码检查 (cargo check, clippy, fmt)
- 🧪 运行测试 (35个单元测试)
- 🔨 构建项目 (release版本)
- ⚡ 性能验证 (功能测试)

### 项目结构

```shell
img-squeeze/
├── src/
│   ├── main.rs          # 主程序入口
│   ├── cli.rs           # 命令行接口
│   ├── processing.rs   # 核心压缩逻辑
│   ├── batch.rs         # 批量处理
│   ├── info.rs          # 图片信息分析
│   ├── walrus.rs        # Walrus 存储集成
│   └── error.rs         # 错误处理
├── Cargo.toml           # 项目配置
├── LICENSE              # MIT 许可证
├── README.md            # 项目说明
├── CLAUDE.md            # Claude Code 开发指南
├── .claude-workflow.yml # Claude 工作流配置
├── claude-workflow.sh   # Claude 工作流执行脚本
└── WALRUS_URLS.md       # Walrus 网络地址说明
```

## 📊 性能特点

- **内存效率** - 使用 Rust 的零成本抽象和内存安全
- **处理速度** - 基于高性能的 `image` 库
- **并行处理** - 支持多线程图片处理（基于 Rayon）
- **PNG 优化** - 使用 oxipng 进行无损 PNG 压缩，支持 Zopfli 算法
- **流式处理** - 大文件的流式处理（未来版本）
- **去中心化存储** - 集成 Walrus 网络，支持区块链存储
- **异步上传** - 基于 tokio 的异步文件上传
- **网络优化** - 智能重试和错误处理机制

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
- [oxipng](https://github.com/shssoichiro/oxipng) - 高级 PNG 压缩优化库
- [clap](https://github.com/clap-rs/clap) - 命令行参数解析库
- [indicatif](https://github.com/console-rs/indicatif) - 进度条库
- [walrus_rs](https://github.com/luojiyin1987/walrus_rs) - Walrus 去中心化存储客户端库
- [tokio](https://github.com/tokio-rs/tokio) - Rust 异步运行时
- [Walrus Network](https://walrus.com/) - 去中心化存储网络

## 📞 支持

如果您遇到问题或有建议，请：

1. 查看 [Issues](https://github.com/luojiyin1987/img-squeeze/issues)
2. 创建新的 Issue

---

**注意**：这是一个开源项目，欢迎任何形式的贡献和反馈！
