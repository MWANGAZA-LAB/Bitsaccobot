#!/bin/bash

# BitSacco WhatsApp Bot Website Build Script
echo "🚀 Building BitSacco WhatsApp Bot Website..."

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf dist/

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Compile TypeScript
echo "🔨 Compiling TypeScript..."
npx tsc

# Check if compilation was successful
if [ $? -eq 0 ]; then
    echo "✅ TypeScript compilation successful!"
    echo "📁 Compiled files are in the dist/ directory"
    echo "🌐 Website is ready for deployment!"
else
    echo "❌ TypeScript compilation failed!"
    exit 1
fi

echo "🎉 Build completed successfully!"
