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

# 创建目标文件夹
mkdir -p "$TARGET_DIR"
echo "✅ 目标文件夹已创建: $TARGET_DIR"

# 记录原始大小
original_size=$(find "$SOURCE_DIR" -type f \( -name "*.jpg" -o -name "*.jpeg" -o -name "*.png" -o -name "*.gif" -o -name "*.bmp" -o -name "*.tiff" -o -name "*.webp" \) -exec du -b {} \; | awk '{sum+=$1} END {print sum}')
echo "📊 原始大小: $(numfmt --to=iec --suffix=B $original_size)"

# 统计文件数量
total_files=$(find "$SOURCE_DIR" -type f \( -name "*.jpg" -o -name "*.jpeg" -o -name "*.png" -o -name "*.gif" -o -name "*.bmp" -o -name "*.tiff" -o -name "*.webp" \) | wc -l)
echo "📊 处理文件数: $total_files"

# 复制文件到目标文件夹
echo "📋 正在复制文件..."
start_time=$(date +%s)
find "$SOURCE_DIR" -type f \( -name "*.jpg" -o -name "*.jpeg" -o -name "*.png" -o -name "*.gif" -o -name "*.bmp" -o -name "*.tiff" -o -name "*.webp" \) -exec cp {} "$TARGET_DIR/" \;
copy_time=$(date +%s)
echo "✅ 文件复制完成，耗时 $((copy_time - start_time)) 秒"

# 进入目标文件夹
cd "$TARGET_DIR"

echo "🗜️  开始压缩图片..."
compress_start_time=$(date +%s)

# 根据文件扩展名分别处理
echo "📸 压缩 JPG/JPEG 文件..."
mogrify -resize "${MAX_WIDTH}x${MAX_HEIGHT}" \
         -quality $QUALITY \
         -strip \
         -interlace Plane \
         -sampling-factor 4:2:0 \
         *.jpg *.jpeg 2>/dev/null || true

echo "🎨 压缩 PNG 文件..."
mogrify -resize "${MAX_WIDTH}x${MAX_HEIGHT}" \
         -quality $QUALITY \
         -strip \
         -depth 8 \
         *.png 2>/dev/null || true

echo "🌐 压缩 WebP 文件..."
mogrify -resize "${MAX_WIDTH}x${MAX_HEIGHT}" \
         -quality $QUALITY \
         -strip \
         *.webp 2>/dev/null || true

echo "🎭 压缩 GIF 文件..."
mogrify -resize "${MAX_WIDTH}x${MAX_HEIGHT}" \
         -quality $QUALITY \
         -strip \
         *.gif 2>/dev/null || true

echo "🖼️  压缩 BMP 文件..."
mogrify -resize "${MAX_WIDTH}x${MAX_HEIGHT}" \
         -quality $QUALITY \
         -strip \
         *.bmp 2>/dev/null || true

echo "📄 压缩 TIFF 文件..."
mogrify -resize "${MAX_WIDTH}x${MAX_HEIGHT}" \
         -quality $QUALITY \
         -strip \
         -compress lzw \
         *.tiff 2>/dev/null || true

compress_end_time=$(date +%s)
echo "✅ 图片压缩完成，耗时 $((compress_end_time - compress_start_time)) 秒"

cd ..

# 计算压缩后大小
compressed_size=$(find "$TARGET_DIR" -type f -exec du -b {} \; | awk '{sum+=$1} END {print sum}')
echo "📊 压缩后大小: $(numfmt --to=iec --suffix=B $compressed_size)"

# 计算压缩率
if [ $original_size -gt 0 ]; then
    saved_size=$((original_size - compressed_size))
    if [ $saved_size -gt 0 ]; then
        saved_ratio=$(echo "scale=2; $saved_size * 100 / $original_size" | bc)
        echo "💰 节省空间: $(numfmt --to=iec --suffix=B $saved_size)"
        echo "📉 压缩率: ${saved_ratio}%"
    else
        echo "⚠️  文件变大: $(numfmt --to=iec --suffix=B $((compressed_size - original_size)))"
        echo "📈 变化率: $(echo "scale=2; $((compressed_size - original_size)) * 100 / $original_size" | bc)%"
    fi
fi

total_time=$((compress_end_time - start_time))
echo "========================================"
echo "🎉 压缩完成！"
echo "📁 压缩后的文件保存在: $TARGET_DIR/"
echo "⏱️  总耗时: $total_time 秒"
echo "📊 处理文件数: $total_files"
echo "========================================"