#!/bin/bash
# Generate placeholder icons for LEAF app
# Requires ImageMagick (brew install imagemagick)

cd "$(dirname "$0")"

# Create a simple green leaf shape on white background using ImageMagick
# The icon is a stylized leaf shape

# Create 32x32
convert -size 32x32 xc:white \
  -fill '#22c55e' \
  -draw "polygon 16,4 28,16 16,28 8,20 8,12" \
  -draw "line 16,8 16,24" \
  32x32.png

# Create 128x128
convert -size 128x128 xc:white \
  -fill '#22c55e' \
  -draw "polygon 64,16 112,64 64,112 32,80 32,48" \
  -draw "line 64,32 64,96" \
  -strokewidth 4 \
  128x128.png

# Create 128x128@2x (256x256)
convert -size 256x256 xc:white \
  -fill '#22c55e' \
  -draw "polygon 128,32 224,128 128,224 64,160 64,96" \
  -draw "line 128,64 128,192" \
  -strokewidth 8 \
  128x128@2x.png

# Create icon.icns for macOS (requires iconutil)
# First create iconset directory
mkdir -p icon.iconset
cp 32x32.png icon.iconset/icon_32x32.png
cp 128x128.png icon.iconset/icon_128x128.png
cp 128x128@2x.png icon.iconset/icon_128x128@2x.png

# Create 16x16 and others
convert 32x32.png -resize 16x16 icon.iconset/icon_16x16.png
convert 128x128.png -resize 64x64 icon.iconset/icon_32x32@2x.png
convert 128x128@2x.png -resize 256x256 icon.iconset/icon_256x256.png
convert 128x128@2x.png -resize 512x512 icon.iconset/icon_256x256@2x.png
convert 128x128@2x.png -resize 512x512 icon.iconset/icon_512x512.png
convert 128x128@2x.png -resize 1024x1024 icon.iconset/icon_512x512@2x.png

# Generate .icns
iconutil -c icns icon.iconset -o icon.icns

# Clean up
rm -rf icon.iconset

echo "Icons generated successfully!"
