# Инсталация

Пълна инструкция за инсталация на RS-AC-BG системата.

## 📋 Предварителни изисквания

### System Requirements

- **OS**: Linux, macOS, Windows (с WSL2)
- **RAM**: Минимум 4GB, препоръчително 8GB+
- **Storage**: 2GB свободно пространство
- **Network**: Интернет връзка за dependencies

### Required Software

1. **Rust** (1.70+)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

2. **Node.js** (22 LTS+) и npm
```bash
# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# macOS
brew install node

# Windows
# Свали от https://nodejs.org
```

3. **PostgreSQL** (13+)
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# macOS
brew install postgresql
brew services start postgresql

# Windows
# Свали от https://postgresql.org
```

4. **Git**
```bash
# Ubuntu/Debian
sudo apt install git

# macOS
brew install git

# Windows - свали Git for Windows
```

## 📦 Clone Repository

```bash
git clone https://github.com/your-org/rs-ac-bg.git
cd rs-ac-bg
```

## 🗄️ Database Setup

### 1. Създаване на база данни

```bash
# Влез в PostgreSQL
sudo -u postgres psql

# Създай база данни и потребител
CREATE DATABASE rsacbg;
CREATE USER rsacbg_user WITH PASSWORD 'your_password_here';
GRANT ALL PRIVILEGES ON DATABASE rsacbg TO rsacbg_user;
\q
```

### 2. Environment Variables

Създай `.env` файл в root директорията:

```bash
# Database
DATABASE_URL=postgresql://rsacbg_user:CHANGE_ME_IN_PRODUCTION@localhost:5432/rsacbg

# Server
HOST=127.0.0.1
PORT=8080

# External APIs
BNB_API_URL=https://www.bnb.bg/Statistics/StExternalSector/StExchangeRates/StERForeignCurrencies/

# Logging
RUST_LOG=info
```

### 3. Run Migrations

```bash
cd migration
cargo run
```

## 🦀 Backend Setup

### 1. Build Backend

```bash
cd backend
cargo build --release
```

### 2. Run Backend

```bash
# Development
cargo run

# Production
cargo run --release
```

Backendът ще се стартира на `http://localhost:8080`

### 3. Test GraphQL

Отвори браузъра на: `http://localhost:8080/graphql`

Тест query:
```graphql
{
  accountHierarchy(companyId: 1) {
    code
    name
  }
}
```

## ⚛️ Frontend Setup

### 1. Install Dependencies

```bash
cd frontend
npm install
```

### 2. Environment Variables

Създай `frontend/.env` файл:

```bash
VITE_API_URL=http://localhost:8080
VITE_APP_TITLE=RS-AC-BG
VITE_APP_VERSION=0.2.0
```

### 3. Run Frontend

```bash
# Development
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

Frontend ще се стартира на `http://localhost:5173`

## 🚀 Production Deployment

### Docker Setup

1. **Create docker-compose.yml**:

```yaml
version: '3.8'

services:
  db:
    image: postgres:15
    restart: always
    environment:
      POSTGRES_DB: rsacbg
      POSTGRES_USER: rsacbg_user
      POSTGRES_PASSWORD: CHANGE_ME_IN_PRODUCTION
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  backend:
    build: 
      context: .
      dockerfile: backend/Dockerfile
    restart: always
    environment:
      DATABASE_URL: postgresql://rsacbg_user:CHANGE_ME_IN_PRODUCTION@db:5432/rsacbg
      HOST: 0.0.0.0
      PORT: 8080
      RUST_LOG: info
    ports:
      - "8080:8080"
    depends_on:
      - db

  frontend:
    build:
      context: .
      dockerfile: frontend/Dockerfile
    restart: always
    ports:
      - "80:80"
    depends_on:
      - backend

volumes:
  postgres_data:
```

2. **Backend Dockerfile**:

```dockerfile
# backend/Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY backend/ ./
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/backend .
EXPOSE 8080
CMD ["./backend"]
```

3. **Frontend Dockerfile**:

```dockerfile
# frontend/Dockerfile
FROM node:22.20-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . ./
RUN npm run build

FROM caddy:2-alpine
COPY --from=builder /app/dist /usr/share/caddy
COPY Caddyfile /etc/caddy/Caddyfile
EXPOSE 3000
# Caddy starts automatically with the default entrypoint
```

4. **Run with Docker**:

```bash
docker-compose up -d
```

### Manual Production

1. **Build Backend**:
```bash
cd backend
cargo build --release
```

2. **Build Frontend**:
```bash
cd frontend
npm run build
```

3. **Setup Reverse Proxy**:

**Препоръчително: Caddy (автоматично SSL)**

Приложението в production използва Caddy за автоматично Let's Encrypt SSL:

```caddy
your-domain.com {
    encode gzip

    # GraphQL API
    handle /graphql {
        reverse_proxy localhost:8080
    }

    # GraphiQL playground
    handle /graphiql {
        reverse_proxy localhost:8080
    }

    # Static frontend
    handle {
        reverse_proxy localhost:3000
    }
}
```

**Алтернативно: Nginx**
```nginx
server {
    listen 80;
    server_name your-domain.com;

    # Frontend
    location / {
        proxy_pass http://localhost:3000;
    }

    # Backend API
    location /graphql {
        proxy_pass http://localhost:8080/graphql;
    }
}
```

4. **Setup systemd service** (/etc/systemd/system/rsacbg-backend.service):
```ini
[Unit]
Description=RS-AC-BG Backend
After=network.target postgresql.service

[Service]
Type=simple
User=rsacbg
WorkingDirectory=/opt/rsacbg/backend
Environment=DATABASE_URL=postgresql://rsacbg_user:password@localhost:5432/rsacbg
Environment=HOST=127.0.0.1
Environment=PORT=8080
ExecStart=/opt/rsacbg/backend/target/release/backend
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable rsacbg-backend
sudo systemctl start rsacbg-backend
```

## 🔒 Security Configuration

### Database Security

1. **Change default passwords**
2. **Restrict network access**:
```postgresql
# pg_hba.conf
local   rsacbg          rsacbg_user                             md5
host    rsacbg          rsacbg_user     127.0.0.1/32            md5
```

3. **Enable SSL**:
```bash
# postgresql.conf
ssl = on
ssl_cert_file = 'server.crt'
ssl_key_file = 'server.key'
```

### Application Security

1. **Environment variables**:
```bash
# Production .env
DATABASE_URL=postgresql://user:password@localhost:5432/rsacbg
JWT_SECRET=your-256-bit-secret-key-here
ALLOWED_ORIGINS=https://your-domain.com
RUST_LOG=warn
```

2. **Firewall rules**:
```bash
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 22/tcp
sudo ufw deny 5432/tcp  # Block external DB access
sudo ufw deny 8080/tcp  # Block external API access
sudo ufw enable
```

## 📊 Initial Data

### 1. Create Company

```graphql
mutation CreateCompany {
  createCompany(input: {
    name: "Моята Фирма ООД"
    eik: "123456789"
    vatNumber: "BG123456789"
    address: "ул. Витоша 1"
    city: "София"
    country: "BG"
  }) {
    id
    name
  }
}
```

### 2. Import Chart of Accounts

```bash
# Използвай universal import за CSV файл със сметкоплан
cd frontend
# Отвори /imports -> Universal -> Upload accounts.csv
```

### 3. Add Initial Counterparts

```csv
# counterparts.csv
Name,EIK,VATNumber,Address,City,Country
НАП,000695132,BG000695132,ул. Дунав 33,София,BG
БНБ,831479347,BG831479347,пл. Александър Батенберг 1,София,BG
```

## 🧪 Testing Installation

### Backend Tests

```bash
cd backend
cargo test
```

### Frontend Tests

```bash
cd frontend
npm test
```

### Integration Test

1. **Backend health check**:
```bash
curl http://localhost:8080/health
```

2. **Frontend access**:
```bash
curl http://localhost:5173
```

3. **GraphQL query**:
```bash
curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ __schema { types { name } } }"}'
```

## 🚨 Troubleshooting

### Common Issues

#### "Database connection failed"
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check connection
psql -h localhost -U rsacbg_user -d rsacbg

# Check .env file
cat .env | grep DATABASE_URL
```

#### "Port already in use"
```bash
# Check what's using port 8080
sudo netstat -tulpn | grep 8080

# Kill process
sudo kill -9 <PID>
```

#### "Migration failed"
```bash
# Drop and recreate database
sudo -u postgres psql
DROP DATABASE rsacbg;
CREATE DATABASE rsacbg;
\q

# Run migrations again
cd migration
cargo run
```

#### "Frontend build fails"
```bash
# Clear npm cache
cd frontend
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

#### "CORS errors"
```bash
# Check backend CORS configuration
# Add to backend main.rs:
let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([hyper::header::CONTENT_TYPE]);
```

### Debug Mode

```bash
# Backend debug
RUST_LOG=debug cargo run

# Frontend debug
npm run dev -- --mode development --debug
```

### Log Files

- **Backend logs**: `backend/logs/app.log`
- **PostgreSQL logs**: `/var/log/postgresql/postgresql-15-main.log`
- **Caddy logs** (production): `/path/to/caddy-proxy/logs/`
- **Docker logs**: `docker compose logs -f [service_name]`

## 📞 Support

За проблеми с инсталацията:

1. Провери [Troubleshooting](#-troubleshooting)
2. Прегледай логовете
3. Създай issue в GitHub repository-то
4. Консултирай се с документацията на зависимостите

## 🔄 Updates

### Updating the Application

```bash
# Pull latest changes
git pull origin main

# Update backend
cd backend
cargo build --release

# Update frontend
cd frontend
npm install
npm run build

# Run migrations if any
cd migration
cargo run
```

### Database Backup

```bash
# Create backup
pg_dump rsacbg > backup_$(date +%Y%m%d).sql

# Restore backup
psql rsacbg < backup_20240101.sql
```
