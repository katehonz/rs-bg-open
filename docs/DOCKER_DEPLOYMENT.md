# Docker Deployment с WireGuard IPv6

## Преглед

Пълна production deployment конфигурация с:
- **WireGuard VPN** - IPv6 tunnel connectivity
- **PostgreSQL** - База данни (порт 5433 за избягване на конфликт с локален PostgreSQL)
- **Redis** - Caching и sessions
- **Rust Backend** - Счетоводно приложение с JWT authentication
- **React Frontend** - Single Page Application
- **Nginx** - Reverse proxy със SSL от ZeroSSL
- **IPv6 мрежа** - Full IPv6 support

## Домейн

**rs.ddns-ip.net** - SSL сертификат от ZeroSSL

## Структура

```
rs-ac-bg/
├── IPV6/
│   ├── wireguard/
│   │   └── wg0.conf              # WireGuard конфигурация от ipv6.rs
│   └── ssl/
│       ├── certificate.crt       # SSL сертификат
│       ├── ca_bundle.crt         # CA bundle
│       └── private.key           # Private key
├── nginx/
│   ├── nginx.conf                # Nginx конфигурация
│   └── certbot/                  # За ACME challenges
├── backend/                      # Rust backend
├── frontend/                     # React frontend
├── docker-compose.yml            # Docker orchestration
├── .env                          # Environment variables
└── scripts/
    ├── init_admin.sh             # Create admin user
    └── setup-wireguard.sh        # Setup WireGuard
```

## Предварителни изисквания

1. **Docker & Docker Compose**
   ```bash
   sudo apt-get update
   sudo apt-get install docker.io docker-compose
   sudo usermod -aG docker $USER
   # Logout и login отново
   ```

2. **IPv6 Support**
   ```bash
   # Enable IPv6
   sudo sysctl -w net.ipv6.conf.all.disable_ipv6=0
   sudo sysctl -w net.ipv6.conf.default.disable_ipv6=0
   sudo sysctl -w net.ipv6.conf.all.forwarding=1

   # Make persistent
   echo "net.ipv6.conf.all.disable_ipv6=0" | sudo tee -a /etc/sysctl.conf
   echo "net.ipv6.conf.all.forwarding=1" | sudo tee -a /etc/sysctl.conf
   sudo sysctl -p
   ```

3. **Docker IPv6 Configuration**

   Редактирай `/etc/docker/daemon.json`:
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

## Setup

### 1. WireGuard Конфигурация

Сложи WireGuard конфигурацията от ipv6.rs:

```bash
cd rs-ac-bg/IPV6/wireguard
nano wg0.conf
```

Примерна конфигурация:
```ini
[Interface]
PrivateKey = <your-private-key-from-ipv6.rs>
Address = <your-ipv6-address>/64

[Peer]
PublicKey = <server-public-key>
Endpoint = <server-endpoint>:51820
AllowedIPs = ::/0
PersistentKeepalive = 25
```

### 2. SSL Сертификати

Сложи сертификатите от ZeroSSL:

```bash
cd rs-ac-bg/IPV6/ssl

# Copy your ZeroSSL files here:
# certificate.crt
# ca_bundle.crt
# private.key

# Set permissions
chmod 644 certificate.crt ca_bundle.crt
chmod 600 private.key
```

### 3. Environment Variables

```bash
cd rs-ac-bg
cp .env.production.example .env
nano .env
```

Попълни:
```bash
# Database
DB_PASSWORD=your_strong_password

# JWT
JWT_SECRET=your_very_strong_random_secret_min_32_chars
JWT_EXPIRATION_HOURS=24

# Domain
DOMAIN=rs.ddns-ip.net
```

### 4. Автоматичен Setup

```bash
cd rs-ac-bg/scripts
./setup-wireguard.sh
```

Скриптът ще:
- Провери конфигурационните файлове
- Настрои permissions
- Enable IPv6
- Покаже инструкции за стартиране

### 5. Създай Admin потребител

```bash
sudo ./init_admin.sh
```

Запази генерираната парола!

### 6. Стартирай приложението

```bash
cd rs-ac-bg
docker-compose up -d
```

## Проверка на статус

### WireGuard

```bash
# Провери дали WireGuard tunnel е активен
docker exec wireguard wg show

# Тествай IPv6 connectivity
docker exec wireguard ping6 -c 3 google.com

# Провери IP адрес
docker exec wireguard ip -6 addr show wg0
```

### Containers

```bash
# Провери всички контейнери
docker-compose ps

# Logs
docker-compose logs -f

# Конкретен контейнер
docker-compose logs -f accounting-service
docker-compose logs -f nginx
docker-compose logs -f wireguard
```

### Nginx SSL

```bash
# Провери Nginx конфигурация
docker exec accounting_nginx nginx -t

# Провери SSL сертификат
echo | openssl s_client -connect rs.ddns-ip.net:443 -servername rs.ddns-ip.net 2>/dev/null | openssl x509 -noout -dates
```

### Application Health

```bash
# Health check
curl http://localhost/health

# HTTPS check
curl -k https://rs.ddns-ip.net/health

# Login test
curl -X POST https://rs.ddns-ip.net/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'
```

## Networking

### Ports

| Service | Internal | External | Protocol |
|---------|----------|----------|----------|
| WireGuard | - | 51820 | UDP |
| PostgreSQL | 5432 | 5433 | TCP |
| Redis | 6379 | 6379 | TCP |
| Backend | 8080 | - | TCP (internal) |
| Frontend | 3000 | - | TCP (internal) |
| Nginx HTTP | 80 | 80 | TCP |
| Nginx HTTPS | 443 | 443 | TCP |

### Network Topology

```
Internet (IPv6)
    ↓
WireGuard Container (51820/udp)
    ↓
Docker Network (accounting_network)
    ├── IPv4: 172.20.0.0/16
    └── IPv6: fd00:172:20::/48
         ├── Nginx (172.20.0.100)
         ├── Frontend (172.20.0.30)
         ├── Backend (172.20.0.20)
         ├── PostgreSQL (172.20.0.10)
         ├── Redis (172.20.0.11)
         └── WireGuard (172.20.0.2)
```

## Troubleshooting

### WireGuard не се свързва

```bash
# Провери WireGuard logs
docker logs wireguard

# Провери firewall
sudo ufw status
sudo ufw allow 51820/udp

# Restart WireGuard
docker-compose restart wireguard
```

### SSL грешки

```bash
# Провери сертификатите
openssl x509 -in IPV6/ssl/certificate.crt -text -noout

# Провери private key
openssl rsa -in IPV6/ssl/private.key -check

# Провери дали certificate и key match-ват
openssl x509 -noout -modulus -in IPV6/ssl/certificate.crt | openssl md5
openssl rsa -noout -modulus -in IPV6/ssl/private.key | openssl md5
```

### Database connection issues

```bash
# Провери PostgreSQL logs
docker logs accounting_db

# Провери дали базата е достъпна
docker exec accounting_db psql -U app -d accounting -c "SELECT version();"

# Test connection from backend
docker exec accounting_service nc -zv db 5432
```

### IPv6 не работи

```bash
# Провери IPv6 на host
ip -6 addr show

# Proveri Docker IPv6
docker network inspect rs-ac-bg_accounting_network | grep IPv6

# Enable IPv6 в Docker daemon
sudo nano /etc/docker/daemon.json
# Add: {"ipv6": true, "fixed-cidr-v6": "fd00::/80"}
sudo systemctl restart docker
```

### Nginx errors

```bash
# Провери Nginx конфигурация
docker exec accounting_nginx nginx -t

# Reload Nginx
docker exec accounting_nginx nginx -s reload

# Провери logs
docker logs accounting_nginx
```

## Maintenance

### Backup

```bash
# Backup PostgreSQL
docker exec accounting_db pg_dump -U app accounting > backup_$(date +%Y%m%d).sql

# Backup volumes
docker run --rm \
  -v rs-ac-bg_postgres_data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/postgres_backup_$(date +%Y%m%d).tar.gz /data
```

### Restore

```bash
# Restore PostgreSQL
docker exec -i accounting_db psql -U app accounting < backup_20240101.sql
```

### Update

```bash
# Pull latest changes
git pull

# Rebuild and restart
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

### SSL Certificate Renewal

ZeroSSL сертификатите са валидни 90 дни. За обновяване:

1. Login в ZeroSSL dashboard
2. Renew сертификата за rs.ddns-ip.net
3. Download новите файлове
4. Replace файловете в IPV6/ssl/
5. Reload Nginx:
   ```bash
   docker exec accounting_nginx nginx -s reload
   ```

## Monitoring

### Log Aggregation

```bash
# Tail all logs
docker-compose logs -f

# Tail specific service
docker-compose logs -f accounting-service

# Save logs
docker-compose logs > logs_$(date +%Y%m%d).txt
```

### Resource Usage

```bash
# Container stats
docker stats

# Disk usage
docker system df

# Clean up
docker system prune -a
```

## Security Checklist

- [ ] Strong JWT_SECRET (32+ random characters)
- [ ] Strong DB_PASSWORD
- [ ] SSL certificates from ZeroSSL installed
- [ ] WireGuard properly configured
- [ ] Firewall configured (allow 80, 443, 51820)
- [ ] Regular backups scheduled
- [ ] Logs monitored
- [ ] Admin password secure and saved
- [ ] HTTPS enforced (HTTP redirects to HTTPS)
- [ ] Rate limiting enabled in Nginx
- [ ] Security headers configured

## Production Checklist

- [ ] .env file configured with production values
- [ ] WireGuard tunnel active and tested
- [ ] IPv6 connectivity verified
- [ ] SSL certificates valid
- [ ] Database migrations run
- [ ] Admin user created
- [ ] Health checks passing
- [ ] Logs accessible
- [ ] Backups configured
- [ ] Monitoring setup
- [ ] DNS configured (A and AAAA records)
- [ ] Firewall rules applied

## Support

За проблеми:
1. Провери logs: `docker-compose logs`
2. Провери health checks: `docker-compose ps`
3. Провери мрежата: `docker network inspect rs-ac-bg_accounting_network`
4. Провери документацията в `docs/`

## Useful Commands

```bash
# Start all services
docker-compose up -d

# Stop all services
docker-compose down

# Restart specific service
docker-compose restart accounting-service

# View logs
docker-compose logs -f

# Execute command in container
docker exec -it accounting_service sh

# Check network
docker network ls
docker network inspect rs-ac-bg_accounting_network

# Clean up
docker-compose down -v  # Warning: removes volumes!
docker system prune -a
```
