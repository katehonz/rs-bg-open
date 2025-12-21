# 🚀 VPS Deployment Instructions

Това са инструкциите за качване и deployment на проекта на VPS сървър.

## ⚠️ ВАЖНО: Версии на софтуера

**Преди deployment прочети `VERSIONS.md`!** Проектът изисква точни версии:
- **Rust**: 1.91.0
- **Node.js**: 22.20.0

Dockerfile-овете са конфигурирани да използват тези версии автоматично.

## 📋 Предварителни изисквания

### На локалната машина:
- Git
- rsync или scp

### На VPS сървъра:
- Docker (version 20.10+)
- Docker Compose (version 2.0+)
- Достатъчно място (минимум 10GB)

## 🔧 Стъпка 1: Подготовка на кода (локално)

```bash
# В проектната директория
./deploy-prepare.sh [user] [group]

# Примери:
./deploy-prepare.sh dvg dvg      # За потребител dvg
./deploy-prepare.sh deploy deploy # За потребител deploy
./deploy-prepare.sh               # Без промяна на owner
```

Скриптът ще:
- ✅ Оправи file permissions (644 за файлове, 755 за директории)
- ✅ Промени owner на файловете (ако е зададен)
- ✅ Изчисти build артефакти (target/, node_modules/, dist/)
- ✅ Провери за чувствителни файлове

## 📤 Стъпка 2: Качване на код на VPS

### Вариант A: С rsync (препоръчително)

```bash
# Синхронизира само промените, много по-бързо
rsync -avz --progress \
  --exclude 'backend/target' \
  --exclude 'frontend/node_modules' \
  --exclude 'frontend/dist' \
  --exclude 'postgres-data' \
  --exclude '.git' \
  . user@your-vps-ip:/opt/accounting
```

### Вариант B: С scp

```bash
# Копира целия проект
scp -r . user@your-vps-ip:/opt/accounting
```

### Вариант C: С Git (за production)

```bash
# На VPS:
git clone https://github.com/your-repo/accounting.git /opt/accounting
cd /opt/accounting
git checkout main  # или production branch
```

## ⚙️ Стъпка 3: Конфигурация на VPS

### 3.1. Създаване на .env файл

```bash
# На VPS:
cd /opt/accounting
nano .env

# Добави следното:
DATABASE_URL=postgres://postgres:CHANGE_ME_IN_PRODUCTION@db:5432/accounting
  POSTGRES_PASSWORD=CHANGE_ME_IN_PRODUCTION
POSTGRES_DB=accounting

JWT_SECRET=CHANGE_ME_IN_PRODUCTION_MIN_32_CHARS
RUST_LOG=info

REDIS_URL=redis://redis:6379

# Optional: Mistral AI
MISTRAL_API_KEY=CHANGE_ME_IN_PRODUCTION

# Optional: Contragent API
CONTRAGENT_API_URL=https://api.example.com
CONTRAGENT_API_KEY=CHANGE_ME_IN_PRODUCTION
```

**⚠️ ВАЖНО:** Смени паролите и секретите!

### 3.2. Генериране на случаен JWT_SECRET

```bash
openssl rand -hex 32
# или
head /dev/urandom | tr -dc A-Za-z0-9 | head -c 64
```

## 🐳 Стъпка 4: Build и Deploy

```bash
# На VPS:
cd /opt/accounting
./deploy-on-vps.sh
```

Скриптът ще:
- ✅ Провери за Docker и Docker Compose
- ✅ Направи backup на текущата база данни (ако има)
- ✅ Спре старите контейнери
- ✅ Build-не новите контейнери (backend Rust, frontend Vite)
- ✅ Стартира контейнерите
- ✅ Провери здравето на услугите

### Ръчен Build (алтернатива)

```bash
# Спиране на стари контейнери
docker compose down

# Build (ще отнеме време - Rust компилация + npm build)
docker compose build

# Стартиране
docker compose up -d

# Проверка на логовете
docker compose logs -f
```

## 📊 Мониторинг

### Проверка на статус

```bash
# Статус на контейнерите
docker compose ps

# Логове (real-time)
docker compose logs -f

# Логове само от backend
docker compose logs -f accounting-service

# Ресурси (CPU/RAM)
docker stats

# Здраве на backend
curl http://localhost:8080/health

# Достъп до frontend
curl http://localhost/
```

### Backup на база данни

```bash
# Ръчен backup
docker compose exec db pg_dump -U postgres accounting > backup_$(date +%Y%m%d).sql

# Restore
cat backup_20250101.sql | docker compose exec -T db psql -U postgres accounting
```

## 🔄 Update (след промени)

```bash
# 1. На локална машина
./deploy-prepare.sh dvg dvg
rsync -avz --progress . user@vps:/opt/accounting

# 2. На VPS
cd /opt/accounting
docker compose build
docker compose up -d
```

## 🔧 Полезни команди

```bash
# Рестарт на услуга
docker compose restart accounting-service

# Рестарт на всички
docker compose restart

# Спиране и изтриване
docker compose down

# Спиране и изтриване с volumes (⚠️ ИЗТРИВА ДАННИ!)
docker compose down -v

# Влизане в контейнер
docker compose exec accounting-service sh
docker compose exec db psql -U postgres accounting

# Преглед на заетото място
docker system df

# Почистване на неизползвани images
docker system prune -a
```

## 🌐 SSL/HTTPS с Caddy

### Автоматично SSL с Caddy (използва се в production)

Приложението използва **Caddy** като reverse proxy с автоматично Let's Encrypt SSL.

**Caddy конфигурация:**
- Локация: `/path/to/caddy-proxy/`
- Конфигурационен файл: `Caddyfile`
- Автоматично SSL за всички домейни

```bash
# Провери Caddy конфигурацията
cd /path/to/caddy-proxy
docker compose exec caddy caddy validate --config /etc/caddy/Caddyfile

# Рестартирай Caddy след промени
docker compose restart caddy

# Виж Caddy логове
docker compose logs -f caddy
```

**Предимства на Caddy:**
- ✅ Автоматично Let's Encrypt SSL
- ✅ Автоматично обновяване на сертификати
- ✅ HTTP/3 поддръжка
- ✅ Проста конфигурация

## 🐛 Troubleshooting

### Backend не стартира

```bash
# Провери логовете
docker compose logs accounting-service

# Провери дали БД е готова
docker compose exec db pg_isready -U postgres

# Рестартирай
docker compose restart accounting-service
```

### Frontend не се зарежда

```bash
# Провери Caddyfile конфигурацията във frontend контейнера
docker compose exec frontend cat /etc/caddy/Caddyfile

# Провери frontend логовете
docker compose logs frontend

# Провери Caddy reverse proxy
cd /path/to/caddy-proxy
docker compose logs caddy | grep rust.cyberbuch.org
```

### База данни грешки

```bash
# Влез в базата
docker compose exec db psql -U postgres accounting

# Провери връзката
docker compose exec db pg_isready -U postgres

# Провери миграциите
docker compose logs accounting-service | grep migration
```

### Свързване отвън

```bash
# Провери firewall
ufw status
ufw allow 80/tcp
ufw allow 443/tcp

# Провери дали портовете са отворени
netstat -tulpn | grep LISTEN
```

## 📁 Структура на deployment

```
/opt/accounting/
├── backend/              # Rust backend код
├── frontend/             # React frontend код
├── docker-compose.yml    # Docker Compose конфигурация
├── .env                  # Environment променливи (НЕ в Git!)
├── deploy-prepare.sh     # Подготовка скрипт
├── deploy-on-vps.sh      # Deploy скрипт
└── postgres-data/        # БД данни (Docker volume)
```

## 🔐 Security Checklist

- [ ] Сменени всички default пароли
- [ ] JWT_SECRET е случаен string
- [ ] .env файл НЕ е в Git
- [ ] Firewall е конфигуриран (само 80, 443, 22)
- [ ] SSH ключове вместо password
- [ ] SSL сертификат инсталиран
- [ ] Auto-updates за security patches
- [ ] Regular backups на база данни

## 📞 Support

За проблеми проверете логовете:
```bash
docker compose logs -f
```

Или се свържете с dev team.
