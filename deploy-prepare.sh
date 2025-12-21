#!/bin/bash
set -e

echo "🚀 Подготовка на код за VPS deployment..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Параметри за потребител (по избор)
TARGET_USER="${1:-$USER}"
TARGET_GROUP="${2:-$TARGET_USER}"

# 1. Оправяне на ownership и permissions
echo -e "${YELLOW}📁 Оправяне на ownership и permissions...${NC}"

# Ако сме root и TARGET_USER е зададен
if [ "$EUID" -eq 0 ] && [ ! -z "$TARGET_USER" ] && [ "$TARGET_USER" != "root" ]; then
    echo -e "${BLUE}  Настройка на owner: $TARGET_USER:$TARGET_GROUP${NC}"

    # Променяне на owner
    chown -R $TARGET_USER:$TARGET_GROUP .

    # Файлове: чети/пиши за owner, чети за група и други
    find . -type f -exec chmod 644 {} \;

    # Директории: exec за всички
    find . -type d -exec chmod 755 {} \;

    # Изпълними файлове
    chmod +x deploy-prepare.sh
    chmod +x deploy-on-vps.sh 2>/dev/null || true

    echo -e "${GREEN}  ✓ Ownership променен на $TARGET_USER:$TARGET_GROUP${NC}"
else
    echo -e "${BLUE}  Настройка на permissions (без промяна на owner)${NC}"

    # Файлове: чети/пиши за owner, чети за група и други
    find . -type f -exec chmod 644 {} \;

    # Директории: exec за всички
    find . -type d -exec chmod 755 {} \;

    # Изпълними файлове
    chmod +x deploy-prepare.sh
    chmod +x deploy-on-vps.sh 2>/dev/null || true

    echo -e "${GREEN}  ✓ Permissions настроени${NC}"
fi

# 2. Почистване на build артефакти и кеш
echo -e "${YELLOW}🧹 Почистване на build артефакти...${NC}"

# Backend
if [ -d "backend/target" ]; then
    rm -rf backend/target
    echo "  ✓ Изчистен backend/target"
fi

# Frontend node_modules (ще се инсталират на VPS)
if [ -d "frontend/node_modules" ]; then
    rm -rf frontend/node_modules
    echo "  ✓ Изчистен frontend/node_modules"
fi

# Frontend dist (ще се build-не на VPS)
if [ -d "frontend/dist" ]; then
    rm -rf frontend/dist
    echo "  ✓ Изчистен frontend/dist"
fi

# Docker volumes data (ако има локални данни)
if [ -d "postgres-data" ]; then
    echo "  ⚠️  Пропускам postgres-data (локални данни)"
fi

# 3. Проверка на .dockerignore
echo -e "${YELLOW}📝 Проверка на .dockerignore...${NC}"
if [ ! -f ".dockerignore" ]; then
    echo "  ⚠️  .dockerignore липсва - създавам..."
    cat > .dockerignore << 'EOF'
# Backend
backend/target/
backend/Cargo.lock

# Frontend
frontend/node_modules/
frontend/dist/
frontend/.vite/

# Git
.git/
.gitignore

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# Logs
*.log
logs/

# Docker
.dockerignore

# Misc
.DS_Store
Thumbs.db
*.tmp
EOF
    echo "  ✓ Създаден .dockerignore"
else
    echo "  ✓ .dockerignore съществува"
fi

# 4. Проверка на чувствителни файлове
echo -e "${YELLOW}🔒 Проверка на чувствителни файлове...${NC}"
if [ -f ".env" ]; then
    echo "  ⚠️  .env файл открит - НЕ го качвай на VPS!"
    echo "  💡 Създай .env директно на VPS"
fi

if [ -f "backend/configdb.json" ]; then
    echo "  ⚠️  configdb.json открит - провери дали съдържа чувствителни данни"
fi

# 5. Показване на размер
echo -e "${YELLOW}📊 Размер на проекта...${NC}"
TOTAL_SIZE=$(du -sh . | cut -f1)
echo "  Общо: $TOTAL_SIZE"

# 6. Създаване на tar архив (по избор)
echo -e "${YELLOW}📦 Готово за качване!${NC}"
echo ""
echo -e "${GREEN}Следващи стъпки:${NC}"
echo "1. rsync -avz --progress . user@vps:/path/to/project"
echo "   (или)"
echo "   scp -r . user@vps:/path/to/project"
echo ""
echo "2. На VPS изпълни:"
echo "   cd /path/to/project"
echo "   ./deploy-on-vps.sh"
echo ""
echo "3. Или директно с docker compose:"
echo "   docker compose build"
echo "   docker compose up -d"
echo ""
echo -e "${GREEN}✅ Подготовката завърши успешно!${NC}"
