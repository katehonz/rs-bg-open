# IPV6 & WireGuard Configuration

Тази папка съдържа конфигурацията за IPv6 и WireGuard VPN.

## Структура

```
IPV6/
├── wireguard/
│   ├── wg0.conf              # WireGuard конфигурация от ipv6.rs
│   ├── privatekey            # Private key
│   └── publickey             # Public key
├── ssl/
│   ├── certificate.crt       # SSL сертификат от ZeroSSL
│   ├── ca_bundle.crt         # CA bundle от ZeroSSL
│   └── private.key           # Private key за SSL
└── README.md                 # Този файл
```

## Инструкции

### 1. WireGuard конфигурация (от ipv6.rs)

Сложи конфигурационния файл от ipv6.rs в `wireguard/wg0.conf`

Примерен формат:
```ini
[Interface]
PrivateKey = <your-private-key>
Address = <your-ipv6-address>/64
DNS = 2001:4860:4860::8888

[Peer]
PublicKey = <server-public-key>
Endpoint = <server-endpoint>:51820
AllowedIPs = ::/0
PersistentKeepalive = 25
```

### 2. SSL Сертификати (от ZeroSSL)

Сложи следните файлове от ZeroSSL в `ssl/`:

- `certificate.crt` - Основният сертификат
- `ca_bundle.crt` - CA bundle (ако имаш)
- `private.key` - Private key за сертификата

### 3. Permissions

```bash
# WireGuard конфигурация
chmod 600 wireguard/wg0.conf
chmod 600 wireguard/privatekey

# SSL файлове
chmod 644 ssl/certificate.crt
chmod 644 ssl/ca_bundle.crt
chmod 600 ssl/private.key
```

## Домейн

Домейн: **rs.ddns-ip.net**

Уверете се, че DNS записите сочат към:
- A запис: IPv4 адреса
- AAAA запис: IPv6 адреса (от WireGuard tunnel)

## След конфигурация

След като сложиш файловете, стартирай:

```bash
cd rs-ac-bg
docker-compose up -d
```

Приложението ще бъде достъпно на:
- https://rs.ddns-ip.net
