#!/bin/bash

echo "🚀 Building RS-AC-BG Frontend for Production..."
echo "================================================"

# Проверка дали сме в правилната директория
if [ ! -d "frontend" ]; then
    echo "❌ Error: frontend directory not found!"
    echo "Please run this script from the project root directory"
    exit 1
fi

# Отиди в frontend директорията
cd frontend

# Инсталирай зависимости ако липсват
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install dependencies"
        exit 1
    fi
fi

# Изтрий стария build
if [ -d "dist" ]; then
    echo "🗑️  Cleaning old build..."
    rm -rf dist
fi

# Създай нов production build
echo "🔨 Building production bundle..."
npm run build

# Провери дали build-ът е успешен
if [ -d "dist" ] && [ -f "dist/index.html" ]; then
    echo ""
    echo "✅ Build successful!"
    echo "================================================"
    echo "📁 Production files location: frontend/dist/"
    echo ""
    
    # Покажи размера на build-а
    echo "📊 Build statistics:"
    echo "-------------------"
    TOTAL_SIZE=$(du -sh dist/ | cut -f1)
    echo "Total size: $TOTAL_SIZE"
    echo ""
    
    # Покажи файловете
    echo "📝 Generated files:"
    echo "-------------------"
    find dist -type f -name "*.js" -o -name "*.css" -o -name "*.html" | while read file; do
        SIZE=$(du -h "$file" | cut -f1)
        echo "$SIZE - $(basename $file)"
    done
    echo ""
    
    echo "🚀 Ready for deployment!"
    echo "================================================"
    echo ""
    echo "Next steps:"
    echo "1. Copy dist/ contents to your web server"
    echo "2. Configure nginx/apache to serve from that directory"
    echo "3. Ensure API endpoint is correctly configured"
    echo ""
    echo "Example deployment:"
    echo "  sudo cp -r dist/* /var/www/html/rs-ac-bg/"
    
else
    echo "❌ Build failed! Check the error messages above."
    exit 1
fi