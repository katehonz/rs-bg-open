# Hetzner Object Storage - Конфигурация за RS-AC-BG

## Обща информация

RS-AC-BG поддържа автоматично backup на PostgreSQL базата данни към Hetzner Object Storage (S3-compatible).

## Предварителни изисквания

1. Hetzner Object Storage проект с:
   - Access Key
   - Secret Key
   - Bucket (напр. `rsacbackup`)
   - Регион (напр. `nbg1` - Nuremberg)

2. Достъп до PostgreSQL базата данни на приложението

## Конфигурация стъпка по стъпка

### Стъпка 1: Свързване към базата данни

```bash
docker compose exec db psql -U app -d accounting
```

### Стъпка 2: Проверка на текущите настройки

```sql
SELECT key, value FROM contragent_settings
WHERE key LIKE 'maintenance.object_storage%'
ORDER BY key;
```

Трябва да видите **8 реда** (или 0 ако няма настройки).

### Стъпка 3: Добавяне на настройките

```sql
-- 1. Включване на Object Storage
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.enabled', 'true', 'Enable S3 backups', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- 2. Endpoint (БЕЗ bucket prefix!)
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.endpoint', 'https://nbg1.your-objectstorage.com', 'Hetzner S3 endpoint', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- 3. Access Key
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.access_key', 'YOUR_ACCESS_KEY_HERE', 'Hetzner access key', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- 4. Secret Key
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.secret_key', 'YOUR_SECRET_KEY_HERE', 'Hetzner secret key', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- 5. Region
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.region', 'eu-central', 'S3 region', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- 6. Bucket name
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.bucket', 'rsacbackup', 'S3 bucket name', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- 7. Prefix (папка в bucket-а)
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.prefix', 'backup', 'S3 object prefix', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- 8. Force path style (ВАЖНО: false за Hetzner!)
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.force_path_style', 'false', 'Use virtual-hosted-style URLs', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;
```

### Стъпка 4: Проверка на конфигурацията

```sql
SELECT COUNT(*) as total FROM contragent_settings
WHERE key LIKE 'maintenance.object_storage%';
```

Трябва да покаже: `total = 8`

### Стъпка 5: Рестартиране на backend

```bash
docker compose restart accounting-service
```

### Стъпка 6: Проверка че работи

```bash
# Изчакайте 5 секунди за startup
sleep 5

# Проверете health status
docker compose ps accounting-service

# Трябва да видите: STATUS = Up X seconds (healthy)
```

## Тестване на backup

1. Влезте в приложението: `https://your-domain.com/settings/system`
2. Кликнете на "Optimize Database" или "Create Backup"
3. След няколко секунди трябва да видите:
   - `accounting_YYYYMMDD_HHMMSS.dump`
   - Storage type: **S3** (не Local)

## Проверка на качените файлове

```bash
# С AWS CLI (ако е инсталиран):
AWS_ACCESS_KEY_ID=YOUR_KEY AWS_SECRET_ACCESS_KEY=YOUR_SECRET \
aws s3 ls s3://rsacbackup/backup/ \
  --endpoint-url=https://nbg1.your-objectstorage.com \
  --region=eu-central

# Трябва да видите списък с .dump файлове
```

## Важни бележки

### ⚠️ Hetzner специфика

1. **force_path_style ТРЯБВА да е `false`**
   - Hetzner използва virtual-hosted-style URLs
   - Правилен URL: `https://rsacbackup.nbg1.your-objectstorage.com/backup/file.dump`
   - Грешен URL: `https://nbg1.your-objectstorage.com/rsacbackup/backup/file.dump`

2. **Endpoint БЕЗ bucket prefix**
   - ✅ Правилно: `https://nbg1.your-objectstorage.com`
   - ❌ Грешно: `https://rsacbackup.nbg1.your-objectstorage.com`

3. **Всички 8 настройки са задължителни**
   - Липсваща настройка води до "dispatch failure" грешка
   - Frontend не може да зареди S3 settings ако липсва някоя

### Региони на Hetzner

| Регион | Endpoint |
|--------|----------|
| Nuremberg | `https://nbg1.your-objectstorage.com` |
| Falkenstein | `https://fsn1.your-objectstorage.com` |
| Helsinki | `https://hel1.your-objectstorage.com` |

## Troubleshooting

### Грешка: "dispatch failure"

**Причина:** Липсва bucket или друга настройка

**Решение:**
```sql
-- Проверете кои settings има:
SELECT key FROM contragent_settings
WHERE key LIKE 'maintenance.object_storage%'
ORDER BY key;

-- Трябва да видите точно тези 8:
-- maintenance.object_storage.access_key
-- maintenance.object_storage.bucket
-- maintenance.object_storage.enabled
-- maintenance.object_storage.endpoint
-- maintenance.object_storage.force_path_style
-- maintenance.object_storage.prefix
-- maintenance.object_storage.region
-- maintenance.object_storage.secret_key
```

### Backup не се качва на S3

**Проверка 1:** Има ли bucket в Hetzner?
```bash
AWS_ACCESS_KEY_ID=YOUR_KEY AWS_SECRET_ACCESS_KEY=YOUR_SECRET \
aws s3 ls --endpoint-url=https://nbg1.your-objectstorage.com
```

**Проверка 2:** Логове на backend
```bash
docker compose logs --tail=100 accounting-service | grep -i "s3\|upload\|object"
```

**Проверка 3:** Настройките в базата
```sql
SELECT key, value, LENGTH(value) FROM contragent_settings
WHERE key LIKE 'maintenance.object_storage%'
ORDER BY key;
```

### Authentication error

**Причина:** Грешни access_key или secret_key

**Решение:**
1. Проверете credentials в Hetzner Console
2. Актуализирайте в базата:
```sql
UPDATE contragent_settings
SET value = 'NEW_ACCESS_KEY', updated_at = CURRENT_TIMESTAMP
WHERE key = 'maintenance.object_storage.access_key';

UPDATE contragent_settings
SET value = 'NEW_SECRET_KEY', updated_at = CURRENT_TIMESTAMP
WHERE key = 'maintenance.object_storage.secret_key';
```
3. Рестартирайте: `docker compose restart accounting-service`

## Готово!

Системата вече автоматично качва всички backups на Hetzner Object Storage. Backup файловете са с формат:

```
s3://rsacbackup/backup/accounting_20251105_232958.dump
```

Можете да видите списък с backups в Settings > System.
