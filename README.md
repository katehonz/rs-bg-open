# RS-AC-BG - Българска счетоводна система

Модерна уеб-базирана счетоводна система изградена с Rust (backend) и React (frontend).

## 🚀 Основни функционалности

- ✅ **Пълен български сметкоплан** - Автоматично зареждане при създаване на фирма
- ✅ **Журнални записи** - Двустранно счетоводство
- ✅ **ДДС обработка v2.0** - Пълна NAP интеграция (про11-про25, пок09-пок15)
- ✅ **ДДС дневници** - Автоматично генериране на DEKLAR.TXT, PRODAGBI.TXT, POKUPKI.TXT
- ✅ **VIES декларации** - ВОД/ВОП операции с автоматична валидация
- 🆕 **Производствен модул** - Технологични карти с автоматични операции
- ✅ **Дълготрайни активи** - Амортизации и управление
- ✅ **Валутни курсове** - Автоматично обновяване от БНБ
- ✅ **Импорт от Controlisy** - XML формат
- ✅ **SAF-T експорт** - Стандартен одитен файл
- ✅ **INTRASTAT модул** - Декларации за вътреобщностна търговия
- ✅ **Отчети** - ОПР, Баланс, Оборотна ведомост
- ✅ **Recovery Code система** - Сигурно възстановяване на пароли

## 📋 Изисквания

- PostgreSQL 14+
- Rust 1.75+
- Node.js 22 LTS (препоръчително 22.20.0)
- 100MB свободно място (25MB за production)

## ⚡ Бърз старт

### 1. Клонирайте проекта
```bash
git clone https://gitlab.com/your-username/rs-ac-bg.git
cd rs-ac-bg
```

### 2. Конфигурация
```bash
cp configdb.example.json configdb.json
# Редактирайте configdb.json с вашите настройки за база данни
```

### 3. База данни
```bash
createdb rs_ac_bg
```

### 4. Backend
```bash
cd backend
cargo build --release
./target/release/backend
```

### 5. Frontend
```bash
cd frontend
npm install
npm run dev
```

Отворете http://localhost:5173

## 📦 Production Deployment

### Docker Deployment (Препоръчително)

```bash
# Стартирайте приложението
docker compose up -d

# Caddy reverse proxy (автоматично SSL)
cd /path/to/caddy-proxy
docker compose up -d
```

**Production setup включва:**
- ✅ Автоматично Let's Encrypt SSL (чрез Caddy)
- ✅ PostgreSQL + Redis + Backend + Frontend
- ✅ Health checks и automatic restarts
- ✅ Оптимизиран cache control за assets

**Управление:**
```bash
# Рестартиране след промени
docker compose restart frontend
cd /path/to/caddy-proxy && docker compose restart caddy

# Виж логове
docker compose logs -f
```

### Manual Deployment

#### Backend
```bash
cargo build --release
# Резултат: backend/target/release/backend (20MB)
```

#### Frontend
```bash
./build-frontend.sh
# Резултат: frontend/dist/ (5MB)
```

**Важно:** За production са нужни САМО:
- `backend` бинарен файл
- `configdb.json`
- `frontend/dist/` съдържание

НЕ качвайте `node_modules/` (500+ MB)!

## 📚 Документация

### Общи
- [Ръководство за инсталация](docs/INSTALLATION_GUIDE.md)
- [Frontend Deployment](docs/FRONTEND_DEPLOYMENT.md)
- [API документация](docs/API.md)
- [Архитектура](docs/ARCHITECTURE.md)

### ДДС Модул
- 📖 [ДДС Модул - Пълна документация](docs/VAT-MODULE.md)
- ⚡ [ДДС Модул - Бърза референция](docs/VAT-QUICK-REFERENCE.md)
- 📋 [ДДС Модул v2.0 - Changelog](docs/CHANGELOG-VAT-2.0.md)

### Сигурност
- 🔐 [Recovery Code Система](docs/RECOVERY_CODE.md)

## 🏗️ Технологии

### Backend
- **Rust** - Actix-Web + async-graphql + SeaORM
- **PostgreSQL** - База данни
- **GraphQL** - API

### Frontend
- **React** - UI библиотека
- **Vite** - Build tool
- **TailwindCSS** - Стилове
- **Apollo Client** - GraphQL клиент

### Production Infrastructure
- **Docker** - Контейнеризация
- **Caddy** - Reverse proxy с автоматично Let's Encrypt SSL
- **Redis** - Кеширане и сесии

## 🔧 Конфигурация

Системата използва `configdb.json` за конфигурация. Копирайте `configdb.example.json` и редактирайте.

## 💰 ДДС Модул v2.0

**Пълна интеграция с NAP спецификация PPDDS_2025!**

Новият ДДС модул поддържа всички операционни кодове и колони от дневниците за продажби и покупки:

### Продажби (про11-про25)
- **про11** - Облагаеми доставки 20%
- **про17** - Облагаеми доставки 9%
- **про19** - Доставки 0% (глава 3)
- **про20** - ВОД (вътреобщностна доставка)
- **про13** - ВОП (вътреобщностно придобиване)
- И още 10+ специфични операции

### Покупки (пок09-пок15)
- **пок09** - Без право на данъчен кредит
- **пок10** - Пълен данъчен кредит
- **пок12** - Частичен данъчен кредит
- **пок14** - Годишна корекция
- **пок15** - Тристранна операция

### Експорт на NAP файлове
- ✅ **DEKLAR.TXT** - Обобщена декларация (1 запис)
- ✅ **PRODAGBI.TXT** - Дневник продажби (N записа)
- ✅ **POKUPKI.TXT** - Дневник покупки (N записа)
- ✅ **Windows-1251** кодировка
- ✅ **Fixed-width** формат

### Интелигентно въвеждане
- 🎯 Автоматично предлагане на ДДС ставка според операцията
- 🎨 Цветни индикатори (пълен ДК, частичен ДК, без ДК)
- ✅ Real-time валидация
- 🇪🇺 VIES интеграция за ВОД/ВОП

📖 **Документация:** [VAT-MODULE.md](docs/VAT-MODULE.md)
⚡ **Бърза референция:** [VAT-QUICK-REFERENCE.md](docs/VAT-QUICK-REFERENCE.md)

## 🌍 INTRASTAT Модул

Системата включва пълнофункционален INTRASTAT модул за декларации за вътреобщностна търговия:

- **Автоматично проследяване на прагове** - 400,000 лв. за входящи/изходящи операции
- **XML експорт** според формата на НАП (версия 2022)
- **Свързване на сметки с CN номенклатура** - опционално при надвишени прагове
- **Автоматично генериране** на декларации от журналните записи
- **Детайлни справки** по страни, периоди и CN кодове

📋 **Документация:** [INTRASTAT_MODULE.md](./INTRASTAT_MODULE.md)
🚀 **Инсталация:** [INTRASTAT_SETUP.md](./INTRASTAT_SETUP.md)

## 🗂️ Структура на проекта

```
rs-ac-bg/
├── backend/           # Rust backend
├── frontend/          # React frontend
├── migration/         # SeaORM миграции
├── INTRASTAT/        # INTRASTAT документи и данни
├── docs/             # Документация
├── configdb.json     # Конфигурация
└── build-frontend.sh # Build скрипт
```

## 🔐 Recovery Code Система

Системата включва защитена функционалност за възстановяване на забравени пароли чрез recovery code:

### Функционалност
- 🔑 **Генериране на код** - Всеки потребител може да генерира собствен recovery code от User Profile
- 🔒 **Bcrypt хеширане** - Кодовете се съхраняват хеширани в базата данни
- ⏰ **Срок на валидност** - 90 дни от момента на генериране
- 🔄 **One-time use** - Кодът се изтрива автоматично след употреба
- ✅ **Валидация** - Формат XXXX-XXXX (8 символа с тире)

### Как работи

1. **Генериране:**
   - Влезте в User Profile (Settings → Profile)
   - Натиснете "Generate Recovery Code"
   - Запазете кода на сигурно място (показва се само веднъж)

2. **Възстановяване:**
   - От Login страницата кликнете "Forgot Password?"
   - Въведете username, recovery code и нова парола
   - Системата валидира кода и обновява паролата

3. **Сигурност:**
   - Кодът е хеширан с bcrypt (не се съхранява в plain text)
   - След успешна употреба, кодът се изтрива
   - При изтекъл срок (>90 дни), кодът не е валиден

### GraphQL API

```graphql
# Генериране на recovery code (изисква автентикация)
mutation {
  generateRecoveryCode(recoveryCode: "ABCD-1234")
}

# Проверка дали потребител може да използва recovery code
query {
  user(id: 1) {
    recovery_code_hash  # null ако няма генериран код
  }
}
```

### REST API

```bash
# Възстановяване на парола
POST /api/auth/recover-password
Content-Type: application/json

{
  "username": "admin",
  "recovery_code": "ABCD-1234",
  "new_password": "newpass123"
}
```

### Database Schema

```sql
ALTER TABLE users ADD COLUMN recovery_code_hash VARCHAR NULL;
ALTER TABLE users ADD COLUMN recovery_code_created_at TIMESTAMP WITH TIME ZONE NULL;
```

📖 **Миграция:** `migration/src/m20251101_000002_add_recovery_code_fields.rs`

## 📄 Лиценз

MIT

## 🤝 Принос

Приемаме pull requests! За големи промени, моля отворете issue първо.
