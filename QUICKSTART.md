# Quick Start Guide - Production Deployment

## Всичко е готово! Следвай тези стъпки за стартиране:

### 1. Конфигурирай Environment

```bash
cd /path/to/rs-ac-bg

# .env файлът е създаден, редактирай го:
nano .env
```

**Задай силни пароли за:**
- `DB_PASSWORD` - за PostgreSQL
- `JWT_SECRET` - минимум 32 случайни символа

Пример за генериране на JWT_SECRET:
```bash
openssl rand -base64 48
```

### 2. Провери SSL файловете

Вече са налични и с правилни permissions:
- ✅ `IPV6/ssl/fullchain_your-domain.crt` - SSL сертификат (fullchain)
- ✅ `IPV6/ssl/your-domain.key` - Private key

### 3. Enable IPv6 на хоста

```bash
# Enable IPv6
sudo sysctl -w net.ipv6.conf.all.disable_ipv6=0
sudo sysctl -w net.ipv6.conf.all.forwarding=1

# Направи го permanent
echo "net.ipv6.conf.all.disable_ipv6=0" | sudo tee -a /etc/sysctl.conf
echo "net.ipv6.conf.all.forwarding=1" | sudo tee -a /etc/sysctl.conf
```

### 4. Конфигурирай Docker за IPv6

Редактирай `/etc/docker/daemon.json`:

```bash
sudo nano /etc/docker/daemon.json
```

Добави:
```json
{
  "ipv6": true,
  "fixed-cidr-v6": "fd00::/80"
}
```

Restart Docker:
```bash
sudo systemctl restart docker
```

### 5. Build и Start контейнерите

```bash
# Build образите
docker-compose build

# Стартирай в background
docker-compose up -d

# Виж логовете
docker-compose logs -f
```

### 6. Провери статуса

```bash
# Провери всички контейнери
docker-compose ps

# Провери health
curl http://localhost/health

# Тествай HTTPS
curl -k https://localhost/health
```

### 7. Създай Admin потребител

```bash
cd scripts
sudo ./init_admin.sh
```

**ВАЖНО:** Запази генерираната парола! Тя се записва също в `/root/.rs-ac-bg-admin`

### 8. Test Login

```bash
# Login с admin
curl -X POST http://localhost/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "GENERATED_PASSWORD_HERE"
  }'
```

Трябва да получиш JWT token.

### 9. Отвори firewall портове

```bash
# Allow HTTP, HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw reload
```

### 10. Тествай HTTPS

Отвори browser и отиди на:
```
https://your-domain.com
```

## Полезни команди

### Logs
```bash
# Всички
docker-compose logs -f

# Конкретен сървиз
docker-compose logs -f accounting-service
docker-compose logs -f frontend
```

### Restart
```bash
# Restart всичко
docker-compose restart

# Restart конкретен сървиз
docker-compose restart accounting-service
```

### Stop
```bash
# Stop всичко
docker-compose down

# Stop и премахни volumes (ВНИМАНИЕ: изтрива данните!)
docker-compose down -v
```

### Database
```bash
# Влез в PostgreSQL
docker exec -it accounting_db psql -U app -d accounting

# Backup
docker exec accounting_db pg_dump -U app accounting > backup.sql

# Restore
docker exec -i accounting_db psql -U app accounting < backup.sql
```

## Network Information

**Docker Network:**
- IPv4: 172.20.0.0/16
- IPv6: fd00:172:20::/48

**Container IPs:**
- PostgreSQL: 172.20.0.10
- Redis: 172.20.0.11
- Backend: 172.20.0.20
- Frontend: 172.20.0.30

**Reverse Proxy:**
- Caddy (външен контейнер в /path/to/caddy-proxy/)

**Ports:**
- PostgreSQL: 5433 (external) → 5432 (internal)
- Redis: 6379
- HTTP: 80
- HTTPS: 443

## Troubleshooting

### SSL грешки
```bash
# Провери Caddy конфигурация (Caddyfile се намира в /path/to/caddy-proxy/)
cd /path/to/caddy-proxy
docker compose exec caddy caddy validate --config /etc/caddy/Caddyfile

# Виж Caddy логове
docker compose logs caddy

# Рестартирай Caddy след промени в Caddyfile
docker compose restart caddy
```

### Database connection failed
```bash
# Провери дали PostgreSQL работи
docker exec accounting_db psql -U app -d accounting -c "SELECT version();"
```

### IPv6 не работи
```bash
# Провери на host
ip -6 addr show

# Провери Docker network
docker network inspect rs-ac-bg_accounting_network | grep -i ipv6
```

## Next Steps

1. **Customize Frontend** - Актуализирай React app в `frontend/`
2. **Add Users** - Създай потребители чрез GraphQL
3. **Configure Companies** - Добави фирми
4. **Setup Backups** - Автоматизирай backups
5. **Monitor Logs** - Setup log aggregation
6. **SSL Renewal** - Планирай обновяване на сертификата (след 90 дни)

## Security Checklist

- [x] SSL certificates installed
- [x] PostgreSQL port changed (5433)
- [x] IPv6 enabled
- [x] Strong DB_PASSWORD set
- [x] Strong JWT_SECRET set
- [x] Firewall configured
- [x] Admin password saved securely
- [ ] Regular backups scheduled

## Support

**Документация:**
- `docs/AUTHENTICATION.md` - JWT authentication
- `docs/DOCKER_DEPLOYMENT.md` - Full deployment guide

**Logs location:**
- Backend logs: `./logs/`
- Container logs: `docker-compose logs`

## Production Deployment Complete! 🎉

Приложението е готово на:
- **HTTP:** http://your-domain.com (redirect към HTTPS)
- **HTTPS:** https://your-domain.com
- **GraphiQL:** https://your-domain.com/graphiql (за testing)

Успех! 🚀
