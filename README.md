# RS-BG-Open - Българска счетоводна система (Open Source)

Модерна уеб-базирана счетоводна система изградена с Rust (backend) и React (frontend) с пълна поддръжка на българското законодателство.

## 📖 Въведение

RS-BG-Open е open source счетоводна система, създадена специално за нуждите на българските фирми. Системата предоставя пълен набор от функции, необходими за водене на счетоводство, ДДС отчитания, амортизации, INTRASTAT декларации и др.
ка
Софтуерът е разработен и тестван с версия rustc 1.89.0 (29483883e 2025-08-04).

## 🚀 Основни функционалности

- ✅ **Пълен български сметкоплан** - Автоматично зареждане при създаване на фирма
- ✅ **Журнални записи** - Двустранно счетоводство
- ✅ **ДДС обработка** - Покупки, продажби, деклариране
- ✅ **Дълготрайни активи** - Амортизации и управление
- ✅ **Валутни курсове** - Автоматично обновяване от БНБ
- ✅ **Импорт от Controlisy** - XML формат
- ✅ **SAF-T експорт** - Стандартен одитен файл
- ✅ **INTRASTAT модул** - Декларации за вътреобщностна търговия
- ✅ **Отчети** - ОПР, Баланс, Оборотна ведомост

## 📋 Изисквания

- PostgreSQL 14+
- Rust 1.75+
- Node.js 22 LTS (препоръчително 22.20.0)
- 100MB свободно място (25MB за production)

## ⚡ Бърз старт

### 1. Клонирайте проекта
```bash
git clone https://github.com/katehonz/rs-bg-open.git
cd rs-bg-open
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

### Backend
```bash
cargo build --release
# Резултат: backend/target/release/backend (20MB)
```

### Frontend
```bash
./build-frontend.sh
# Резултат: frontend/dist/ (5MB)
```

**Важно:** За production са нужни САМО:
- `backend` бинарен файл
- `configdb.json`
- `frontend/dist/` съдържание

НЕ качвайте `node_modules/` (500+ MB)!

## 🐳 Docker поддръжка

Системата поддържа Docker deployment:

```bash
docker-compose up -d
```

## 📚 Документация

Подробна документация ще бъде добавена скоро.

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

## 🔧 Конфигурация

Всички настройки се правят в `configdb.json` файла. Използвайте `configdb.example.json` като шаблон.

## Лиценз

Този проект е лицензиран под LGPL-3.0 лиценза - вижте [LICENSE](LICENSE) файла за повече информация.