# SSL Сертификати от ZeroSSL

Сложи следните файлове от ZeroSSL в тази папка:

## Необходими файлове:

1. **certificate.crt** - Основният SSL сертификат за rs.ddns-ip.net
2. **ca_bundle.crt** - CA Bundle (Certificate Authority chain)
3. **private.key** - Private key за сертификата

## Как да получиш сертификатите от ZeroSSL:

1. Login в ZeroSSL dashboard
2. Намери сертификата за `rs.ddns-ip.net`
3. Download сертификата
4. Извлечи файловете и ги преименувай:
   - `certificate.crt` (или може да е `domain.crt`)
   - `ca_bundle.crt` (или може да е `ca_bundle.pem`)
   - `private.key` (този трябва да го имаш от генерирането на CSR)

## Permissions:

След като сложиш файловете, задай правилните permissions:

```bash
chmod 644 certificate.crt
chmod 644 ca_bundle.crt
chmod 600 private.key
```

## Проверка:

Провери дали сертификатите са валидни:

```bash
# Провери сертификата
openssl x509 -in certificate.crt -text -noout

# Провери private key
openssl rsa -in private.key -check

# Провери дали private key и certificate match-ват
openssl x509 -noout -modulus -in certificate.crt | openssl md5
openssl rsa -noout -modulus -in private.key | openssl md5
# Двете MD5 hash-ове трябва да са еднакви
```

## Автоматично обновяване (optional):

За автоматично обновяване на сертификатите, можеш да използваш certbot:

```bash
# Install certbot
sudo apt-get install certbot

# Генерирай нов сертификат
sudo certbot certonly --webroot \
  -w /path/to/rs-ac-bg/nginx/certbot \
  -d rs.ddns-ip.net

# Сертификатите ще бъдат в /etc/letsencrypt/live/rs.ddns-ip.net/
```

Но понеже използваш ZeroSSL, вероятно ще трябва да обновяваш ръчно преди да изтекат (обикновено след 90 дни).
