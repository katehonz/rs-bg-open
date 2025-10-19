# ✅ Чеклист за миграция към микросервизи

## 🔒 КРИТИЧНО: Преди да започнеш

- [ ] **BACKUP на production база** - изпълни `./backup_critical_data.sh`
- [ ] **Тестване на backup** - възстанови на тестова база и провери
- [ ] **Копие на цялата папка** - архивирай цялия проект
- [ ] **Git commit** на всички промени
- [ ] **Уведоми потребителите** за планирания maintenance

## 🏗️ Подготовка на средата

### 1. База данни
- [x] Документиране на схемата в `MIGRATION_PLAN.md`
- [x] Backup скрипт `backup_critical_data.sh` 
- [ ] Export на всички migration файлове
- [ ] Тест на restore процедури

### 2. Контейнеризация  
- [x] `docker-compose.yml` за всички сервизи
- [x] `Dockerfile.accounting` за Rust сервиза
- [x] `.env.example` за конфигурация
- [ ] Dockerfiles за Java сервизите (когато се създадат)

### 3. Сметкоплан (КРИТИЧНО!)
- [ ] Export в CSV: `chart_of_accounts.csv`
- [ ] SQL dump: `chart_of_accounts_backup.sql`  
- [ ] Проверка на всички родителски връзки
- [ ] Бройка записи преди/след миграция

## 🚀 Поетапна миграция

### Фаза 1: Контейнеризация на съществуващото
```bash
# 1. Backup
./backup_critical_data.sh

# 2. Стартиране в Docker
cp .env.example .env
# Редактирай .env с правилни пароли
docker-compose up -d db redis
# Чакай db да стартира
docker-compose up accounting-service
```

- [ ] DB контейнер стартира успешно
- [ ] Rust сервиз се свързва към DB
- [ ] Всички migration файлове се прилагат
- [ ] Frontend се свързва към backend API
- [ ] Тестване на основни функции

### Фаза 2: Java авторизационен сервиз
```bash
# Създаване на Spring Boot проект
mkdir auth-service
cd auth-service
# ... Spring Boot setup
```

- [ ] Spring Boot setup с PostgreSQL
- [ ] JWT authentication
- [ ] User management API
- [ ] Company access control
- [ ] Integration с Rust сервиза

### Фаза 3: Java контрагенти сервиз  
```bash
mkdir counterparty-service
# ... Spring Boot setup с VIES и AI
```

- [ ] Counterparties CRUD API
- [ ] VAT VIES проверка
- [ ] AI адресна обработка за SAF-T
- [ ] Геокодиране
- [ ] Integration с main accounting сервиз

### Фаза 4: API рефакториране
- [ ] Разделяне на endpoints между сервизите
- [ ] Междусервизна комуникация (REST/gRPC)
- [ ] Error handling и circuit breakers
- [ ] Distributed logging

## 🔍 Тестове и валидация

### Функционални тестове
- [ ] Вход и оторизация работят
- [ ] Счетоводни записи се създават правилно
- [ ] НАП експорт функционира
- [ ] Контролиси импорт работи
- [ ] ДДС декларации се генерират
- [ ] INTRASTAT модул функционира

### Данни тестове  
- [ ] Всички таблици са налични
- [ ] Сметкопланът е изцяло мигриран
- [ ] Броят записи съответства на backup-а
- [ ] Връзките между таблиците са запазени
- [ ] UTF-8 кодировката е правилна

### Performance тестове
- [ ] Response времена са приемливи  
- [ ] DB connections са оптимизирани
- [ ] Memory usage е в норма
- [ ] Load balancing работи (ако се използва)

## 🚨 Rollback план

Ако нещо се обърка:

1. **Спри всички нови контейнери**
   ```bash
   docker-compose down
   ```

2. **Възстанови базата данни**
   ```bash
   # Възстанови от пълния backup
   psql -h HOST -U USER -d DATABASE -f migration_backups/LATEST/full_database_backup.sql
   ```

3. **Стартирай стария код**
   ```bash
   # Стартирай Rust backend както преди
   cargo run
   ```

## 📋 Production deployment

### CI/CD Pipeline
- [ ] GitLab CI/CD за всички сервизи
- [ ] Automated testing
- [ ] Docker image builds
- [ ] Database migrations в pipeline

### Monitoring & Logging
- [ ] Health checks за всички сервизи
- [ ] Structured logging (JSON)
- [ ] Metrics collection (Prometheus?)
- [ ] Alert setup за критични грешки

### Security
- [ ] Secrets management (не в git!)
- [ ] HTTPS setup с валиден SSL
- [ ] Database connection encryption
- [ ] API rate limiting

### Backup automation
- [ ] Automated daily backups
- [ ] Offsite backup storage
- [ ] Backup testing procedures
- [ ] Recovery time objectives (RTO)

## ⚡ Аварийни контакти

- **DB Admin**: [име и телефон]  
- **DevOps**: [име и телефон]
- **Business owner**: [име и телефон]

## 📝 След миграцията

- [ ] Документиране на новата архитектура
- [ ] Обучение на екипа за новите процеси
- [ ] Performance baseline измерения
- [ ] Post-migration review meeting
- [ ] Планиране на следващи оптимизации

---

## 🎯 Ключови показатели за успех

✅ **Успешна миграция когато:**
- Всички функции работят както преди
- Данните са изцяло запазени (особено сметкопланът!)  
- Response времената са приемливи
- Няма загуба на функционалност
- Екипът може да поддържа новата архитектура

⚠️ **Червени флагове:**
- Липсват записи в критични таблици
- Сметкопланът има грешни връзки
- API response-ите са много бавни
- Често крашване на сервизи  
- Загуба на user sessions