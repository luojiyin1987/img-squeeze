#!/bin/bash

# mogrify 最佳实践脚本
# mogrify 本质上是原地修改工具，所以我们复制后用 mogrify 批量处理

set -e

# 参数设置
SOURCE_DIR="${1:-imgs}"
TARGET_DIR="${2:-imgs_mogrify}"
QUALITY="${QUALITY:-80}"
MAX_WIDTH="${MAX_WIDTH:-1920}"
MAX_HEIGHT="${MAX_HEIGHT:-1080}"

echo "🗜️  ImageMagick mogrify 最佳实践脚本"
echo "========================================"
echo "📁 源文件夹: $SOURCE_DIR"
echo "📁 目标文件夹: $TARGET_DIR"
echo "🎯 质量设置: $QUALITY"
echo "📏 最大尺寸: ${MAX_WIDTH}x${MAX_HEIGHT}"
echo "========================================"

# 检查源文件夹
if [ ! -d "$SOURCE_DIR" ]; then
    echo "❌ 错误: 源文件夹 '$SOURCE_DIR' 不存在"
    exit 1
fi

# 目录关系校验
require_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "❌ 缺少依赖: $1"; exit 1; }; }
require_cmd realpath
src_abs=$(realpath "$SOURCE_DIR")
dst_abs=$(realpath "$TARGET_DIR")
if [ "$src_abs" = "$dst_abs" ] || [[ "$dst_abs" == "$src_abs/"* ]]; then
  echo "❌ 错误: 目标目录不可与源目录相同，亦不可嵌套于源目录内"
  exit 1
fi

# 创建目标文件夹
mkdir -p "$TARGET_DIR"
echo "✅ 目标文件夹已创建: $TARGET_DIR"

# 记录原始大小
# 兼容 macOS 和 Linux 系统
if command -v numfmt >/dev/null 2>&1; then
    NUMFMT_CMD="numfmt --to=iec --suffix=B"
else
    NUMFMT_CMD="awk '{function human(x) { x[1]/=1024; x[2]/=1048576; x[3]/=1073741824; if (x[1]<1000) {printf \"%.1fK\", x[1]} else if (x[2]<1000) {printf \"%.1fM\", x[2]} else {printf \"%.1fG\", x[3]}} {split($1,x,\"\"); human(x[1])}'"
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS 系统
    original_size=$(find "$SOURCE_DIR" -type f \( \
        -iname "*.jpg"  -o -iname "*.jpeg" -o -iname "*.png"  -o -iname "*.gif"  -o \
        -iname "*.bmp"  -o -iname "*.tif"  -o -iname "*.tiff" -o -iname "*.webp" -o \
        -iname "*.heic" -o -iname "*.heif" \
    \) -exec stat -f %z {} \; | awk '{sum+=$1} END {print sum+0}')
else
    # Linux 系统
    original_size=$(find "$SOURCE_DIR" -type f \( \
        -iname "*.jpg"  -o -iname "*.jpeg" -o -iname "*.png"  -o -iname "*.gif"  -o \
        -iname "*.bmp"  -o -iname "*.tif"  -o -iname "*.tiff" -o -iname "*.webp" -o \
        -iname "*.heic" -o -iname "*.heif" \
    \) -exec du -b {} \; | awk '{sum+=$1} END {print sum+0}')
fi
echo "📊 原始大小: $(echo "$original_size" | eval $NUMFMT_CMD)"

# 统计文件数量
total_files=$(find "$SOURCE_DIR" -type f \( \
  -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" -o -iname "*.gif" -o \
  -iname "*.bmp" -o -iname "*.tif"  -o -iname "*.tiff" -o -iname "*.webp" -o \
  -iname "*.heic" -o -iname "*.heif" \
\) | wc -l)
echo "📊 处理文件数: $total_files"

# 复制文件到目标文件夹（保留目录层级）
echo "📋 正在复制文件..."
start_time=$(date +%s)

# 使用 cp --parents 或 install -D 来保留目录层级
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS 使用自定义方法
    find "$src_abs" -type f \( \
        -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" -o -iname "*.gif" -o \
        -iname "*.bmp" -o -iname "*.tif"  -o -iname "*.tiff" -o -iname "*.webp" -o \
        -iname "*.heic" -o -iname "*.heif" \
    \) -exec sh -c '
        for file do
            rel_path="${file#"$1"/}"
            mkdir -p "$2/$(dirname "$rel_path")"
            cp "$file" "$2/$rel_path"
        done
    ' sh {} "$src_abs" "$dst_abs" +
else
    # Linux 使用 cp --parents
    (
        cd "$src_abs" && find . -type f \( \
            -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" -o -iname "*.gif" -o \
            -iname "*.bmp" -o -iname "*.tif"  -o -iname "*.tiff" -o -iname "*.webp" -o \
            -iname "*.heic" -o -iname "*.heif" \
        \) -exec cp --parents -t "$dst_abs" {} +
    )
fi

copy_time=$(date +%s)
echo "✅ 文件复制完成，耗时 $((copy_time - start_time)) 秒"

# 进入目标文件夹
cd "$dst_abs"

echo "🗜️  开始压缩图片..."
compress_start_time=$(date +%s)

# 递归压缩各格式文件
echo "📸 压缩 JPG/JPEG 文件..."
find . -type f \( -iname "*.jpg" -o -iname "*.jpeg" \) -exec mogrify \
    -auto-orient \
    -resize "${MAX_WIDTH}x${MAX_HEIGHT}>" \
    -quality "$QUALITY" \
    -strip \
    -interlace Plane \
    -sampling-factor 4:2:0 \
    {} + 2>/dev/null || true

echo "🎨 压缩 PNG 文件..."
find . -type f -iname "*.png" -exec mogrify \
    -auto-orient \
    -resize "${MAX_WIDTH}x${MAX_HEIGHT}>" \
    -quality "$QUALITY" \
    -strip \
    -depth 8 \
    {} + 2>/dev/null || true

echo "🌐 压缩 WebP 文件..."
find . -type f -iname "*.webp" -exec mogrify \
    -auto-orient \
    -resize "${MAX_WIDTH}x${MAX_HEIGHT}>" \
    -quality "$QUALITY" \
    -strip \
    {} + 2>/dev/null || true

echo "🎭 压缩 GIF 文件..."
find . -type f -iname "*.gif" -exec mogrify \
    -auto-orient \
    -resize "${MAX_WIDTH}x${MAX_HEIGHT}>" \
    -quality "$QUALITY" \
    -strip \
    {} + 2>/dev/null || true

echo "🖼️  压缩 BMP 文件..."
find . -type f -iname "*.bmp" -exec mogrify \
    -auto-orient \
    -resize "${MAX_WIDTH}x${MAX_HEIGHT}>" \
    -quality "$QUALITY" \
    -strip \
    {} + 2>/dev/null || true

echo "📄 压缩 TIFF/TIF 文件..."
find . -type f \( -iname "*.tiff" -o -iname "*.tif" \) -exec mogrify \
    -auto-orient \
    -resize "${MAX_WIDTH}x${MAX_HEIGHT}>" \
    -quality "$QUALITY" \
    -strip \
    -compress lzw \
    {} + 2>/dev/null || true

echo "🍎 压缩 HEIC/HEIF 文件..."
# HEIC/HEIF 是现代格式，使用更优化的压缩参数
find . -type f \( -iname "*.heic" -o -iname "*.heif" \) -exec mogrify \
    -auto-orient \
    -resize "${MAX_WIDTH}x${MAX_HEIGHT}>" \
    -quality "$QUALITY" \
    -strip \
    {} + 2>/dev/null || true

compress_end_time=$(date +%s)
echo "✅ 图片压缩完成，耗时 $((compress_end_time - compress_start_time)) 秒"

cd ..

# 计算压缩后大小
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS 系统
    compressed_size=$(find "$dst_abs" -type f -exec stat -f %z {} \; | awk '{sum+=$1} END {print sum+0}')
else
    # Linux 系统
    compressed_size=$(find "$dst_abs" -type f -exec du -b {} \; | awk '{sum+=$1} END {print sum+0}')
fi
echo "📊 压缩后大小: $(echo "$compressed_size" | eval $NUMFMT_CMD)"

# 计算压缩率
if [ $original_size -gt 0 ]; then
    saved_size=$((original_size - compressed_size))
    if [ $saved_size -gt 0 ]; then
        saved_ratio=$(echo "scale=2; $saved_size * 100 / $original_size" | bc)
        echo "💰 节省空间: $(echo "$saved_size" | eval $NUMFMT_CMD)"
        echo "📉 压缩率: ${saved_ratio}%"
    else
        increased_size=$((compressed_size - original_size))
        echo "⚠️  文件变大: $(echo "$increased_size" | eval $NUMFMT_CMD)"
        echo "📈 变化率: $(echo "scale=2; $increased_size * 100 / $original_size" | bc)%"
    fi
fi

total_time=$((compress_end_time - start_time))
echo "========================================"
echo "🎉 压缩完成！"
echo "📁 压缩后的文件保存在: $dst_abs/"
echo "⏱️  总耗时: $total_time 秒"
echo "📊 处理文件数: $total_files"
echo "========================================"