#!/bin/bash
set -e

echo "🚀 Deployment на VPS..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Проверка дали сме на правилното място
if [ ! -f "docker-compose.yml" ]; then
    echo -e "${RED}❌ docker-compose.yml не е намерен!${NC}"
    echo "Моля изпълнете скрипта в root директорията на проекта"
    exit 1
fi

# 1. Проверка на Docker
echo -e "${YELLOW}🐳 Проверка на Docker...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker не е инсталиран!${NC}"
    echo "Инсталирай Docker с:"
    echo "  curl -fsSL https://get.docker.com -o get-docker.sh"
    echo "  sudo sh get-docker.sh"
    exit 1
fi

if ! command -v docker compose &> /dev/null; then
    echo -e "${RED}❌ Docker Compose не е инсталиран!${NC}"
    exit 1
fi

echo -e "${GREEN}  ✓ Docker и Docker Compose са налични${NC}"

# 2. Проверка на .env файл
echo -e "${YELLOW}🔧 Проверка на конфигурация...${NC}"
if [ ! -f ".env" ]; then
    echo -e "${RED}❌ .env файл липсва!${NC}"
    echo ""
    echo "Създай .env файл с необходимите променливи:"
    echo ""
    cat << 'EOF'
# Database
DATABASE_URL=postgres://postgres:CHANGE_ME_IN_PRODUCTION@db:5432/accounting
POSTGRES_PASSWORD=CHANGE_ME_IN_PRODUCTION
POSTGRES_DB=accounting

# Backend
JWT_SECRET=CHANGE_ME_IN_PRODUCTION_MIN_32_CHARS
RUST_LOG=info

# Redis
REDIS_URL=redis://redis:6379

# Optional: Mistral AI
MISTRAL_API_KEY=CHANGE_ME_IN_PRODUCTION

# Optional: Contragent API
CONTRAGENT_API_URL=https://your-api-url
CONTRAGENT_API_KEY=your-api-key
EOF
    echo ""
    exit 1
fi

echo -e "${GREEN}  ✓ .env файл намерен${NC}"

# 3. Backup на съществуваща база данни (ако има)
if docker ps -a | grep -q accounting_db; then
    echo -e "${YELLOW}💾 Backup на съществуваща база данни...${NC}"
    BACKUP_FILE="backup_$(date +%Y%m%d_%H%M%S).sql"
    docker compose exec -T db pg_dump -U postgres accounting > "$BACKUP_FILE" 2>/dev/null || true
    if [ -f "$BACKUP_FILE" ]; then
        echo -e "${GREEN}  ✓ Backup създаден: $BACKUP_FILE${NC}"
    fi
fi

# 4. Спиране на стари контейнери
echo -e "${YELLOW}⏸️  Спиране на стари контейнери...${NC}"
docker compose down || true

# 5. Изтегляне на базови images
echo -e "${YELLOW}📥 Изтегляне на базови Docker images...${NC}"
docker compose pull || true

# 6. Build на контейнерите
echo -e "${YELLOW}🔨 Build на контейнери (това може да отнеме време)...${NC}"
echo ""
echo "  Backend: Компилиране на Rust код..."
echo "  Frontend: npm install + vite build..."
echo ""

# Build with progress
docker compose build --progress=plain

# 7. Стартиране на контейнерите
echo -e "${YELLOW}▶️  Стартиране на контейнери...${NC}"
docker compose up -d

# 8. Изчакване на инициализация
echo -e "${YELLOW}⏳ Изчакване на инициализация...${NC}"
sleep 10

# 9. Проверка на статус
echo -e "${YELLOW}📊 Статус на контейнерите:${NC}"
docker compose ps

# 10. Проверка на здравето
echo ""
echo -e "${YELLOW}🏥 Проверка на здравето на услугите...${NC}"

# Проверка на backend
if curl -f http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${GREEN}  ✓ Backend е здрав (http://localhost:8080)${NC}"
else
    echo -e "${RED}  ✗ Backend не отговаря${NC}"
fi

# Проверка на frontend
if curl -f http://localhost/ > /dev/null 2>&1; then
    echo -e "${GREEN}  ✓ Frontend е достъпен (http://localhost)${NC}"
else
    echo -e "${RED}  ✗ Frontend не е достъпен${NC}"
fi

# 11. Показване на логове
echo ""
echo -e "${YELLOW}📋 Последни логове:${NC}"
docker compose logs --tail=20

# 12. Полезни команди
echo ""
echo -e "${GREEN}✅ Deployment завършен успешно!${NC}"
echo ""
echo "Полезни команди:"
echo "  docker compose ps              - Преглед на контейнери"
echo "  docker compose logs -f         - Real-time логове"
echo "  docker compose logs backend    - Логове само от backend"
echo "  docker compose restart         - Рестарт на всички услуги"
echo "  docker compose down            - Спиране и премахване"
echo "  docker compose exec db psql -U postgres accounting - Достъп до БД"
echo ""
echo "Мониторинг:"
echo "  docker stats                   - CPU/RAM статистики"
echo "  docker compose top             - Процеси в контейнерите"
echo ""
echo "Nginx SSL/HTTPS:"
echo "  Ако използваш Nginx + Let's Encrypt, добави:"
echo "  - Certbot контейнер в docker-compose.yml"
echo "  - SSL конфигурация в nginx.conf"
