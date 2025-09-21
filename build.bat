@echo off
REM BitSacco WhatsApp Bot Website Build Script for Windows

echo 🚀 Building BitSacco WhatsApp Bot Website...

REM Clean previous build
echo 🧹 Cleaning previous build...
if exist dist rmdir /s /q dist

REM Install dependencies if node_modules doesn't exist
if not exist node_modules (
    echo 📦 Installing dependencies...
    npm install
)

REM Compile TypeScript
echo 🔨 Compiling TypeScript...
npx tsc

REM Check if compilation was successful
if %errorlevel% equ 0 (
    echo ✅ TypeScript compilation successful!
    echo 📁 Compiled files are in the dist/ directory
    echo 🌐 Website is ready for deployment!
) else (
    echo ❌ TypeScript compilation failed!
    exit /b 1
)

echo 🎉 Build completed successfully!
pause
