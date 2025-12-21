# План за миграция към микросервизна архитектура

## Обзор
Миграция от монолитна Rust/React архитектура към микросервизи с:
- Главен счетоводен сервиз (Rust)
- Авторизационен сервиз (Java)
- Контрагенти/VAT VIES сервиз с AI адресна обработка (Java)
- Контейнеризация с Docker

## Критични данни за запазване

### 1. Сметкоплан и счетоводни данни
**ПРИОРИТЕТ: МНОГО ВИСОК**
- `accounts` - сметкоплан с всички налични сметки
- `journal_entries` - всички счетоводни записи
- `entry_lines` - редове от записите
- `chart_of_accounts` - основен план на сметките

### 2. Компании и потребители
- `companies` - фирми
- `users` - потребители
- `user_groups` - групи и права
- `company_users` - връзки потребител-фирма

### 3. Контрагенти и ДДС данни
- `counterparts` - контрагенти
- `vat_returns` - ДДС декларации
- `vat_rates` - ДДС ставки

### 4. Импорти и експорти
- `controlisy_imports` - Controlisy импорти
- `bank_imports` - банкови импорти
- Всички исторически данни

## Архитектурен план

### Сервиз 1: Главен счетоводен (Rust)
**Порт: 8080**
- Запазва всички счетоводни данни
- APIs за журнални записи
- НАП експорти
- SAF-T генериране
- INTRASTAT

### Сервиз 2: Авторизация (Java Spring Boot)
**Порт: 8081** 
- JWT tokens
- User management
- Permissions & roles
- Company access control

### Сервиз 3: Контрагенти/VAT VIES (Java Spring Boot)
**Порт: 8082**
- Контрагенти CRUD
- VAT VIES проверки
- AI адресна обработка за SAF-T
- Геокодиране

## База данни стратегия

### Option A: Shared Database (за начало)
- Един PostgreSQL за всички сервизи
- Всеки сервиз достъпва само своите таблици
- По-лесна миграция

### Option B: Database per Service (идеално)
- Отделна база за всеки сервиз
- API комуникация между сервизите
- По-сложна синхронизация

## Docker конфигурация

```yaml
# docker-compose.yml
version: '3.8'
services:
  db:
    image: postgres:15
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: accounting
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${DB_PASSWORD}

  accounting-service:
    build: ./accounting-service
    ports:
      - "8080:8080"
    depends_on:
      - db

  auth-service:
    build: ./auth-service
    ports:
      - "8081:8081"
    depends_on:
      - db

  counterparty-service:
    build: ./counterparty-service
    ports:
      - "8082:8082"
    depends_on:
      - db

  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
    depends_on:
      - accounting-service
      - auth-service
      - counterparty-service
```

## Миграционни стъпки

### Фаза 1: Подготовка и backup
1. ✅ Пълен database backup
2. ✅ Документиране на схемата
3. ✅ Export на сметкоплана
4. ✅ Тестване на restore процедури

### Фаза 2: Контейнеризация на съществуващото
1. Dockerize на текущия Rust backend
2. Dockerize на React frontend
3. Docker compose setup
4. Тестване на локална среда

### Фаза 3: Създаване на Java сервизите
1. Auth service (JWT, users, permissions)
2. Counterparty service (CRUD, VAT VIES, AI)
3. API Gateway (optional)

### Фаза 4: Рефакториране на комуникацията
1. Разделяне на API endpoints
2. Междусервизна комуникация
3. Конзистентност на данните
4. Error handling & retry logic

### Фаза 5: Production deployment
1. CI/CD pipeline
2. Monitoring & logging
3. Health checks
4. Backup automation

## API граници

### Accounting Service (Rust)
```
/api/accounting/
  - /journal-entries
  - /accounts
  - /reports
  - /exports (NAP, SAF-T)
  - /intrastat
```

### Auth Service (Java)
```
/api/auth/
  - /login
  - /users
  - /companies
  - /permissions
```

### Counterparty Service (Java)
```
/api/counterparties/
  - /counterparties
  - /vat-check
  - /address-ai
  - /geo-coding
```

## Рискове и митигация

### Риск: Загуба на данни
- **Митигация**: Множествени backup-и преди всяка стъпка
- **Rollback план**: Готовност за възстановяване на предишна версия

### Риск: Downtime
- **Митигация**: Blue-green deployment
- **Monitoring**: Health checks за всички сервизи

### Риск: Неконзистентност на данните
- **Митигация**: Транзакции и eventual consistency
- **Тестване**: Integration тестове

## Следващи стъпки

1. **НЕЗАБАВНО**: Backup на production база данни
2. Създаване на Docker setup за текущия код
3. Java сервизи proof of concept
4. Тестване на локална среда
5. Поетапна миграция