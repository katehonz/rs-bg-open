# Бързо ръководство за deployment

## За Frontend - ВАЖНО! 

### Кои файлове НЕ са нужни за production:

❌ **НЕ качвайте тези папки/файлове:**
- `frontend/node_modules/` - **500+ MB** зависимости!
- `frontend/src/` - изходен код
- `frontend/package.json`
- `frontend/vite.config.js`

✅ **САМО това е нужно:**
- `frontend/dist/` - **под 5MB** компилирани файлове

## Пълен процес за deployment

### 1. Подготовка на Backend

```bash
# Компилирайте backend-а
cd backend
cargo build --release

# Резултат: target/release/backend (единичен бинарен файл ~20MB)
```

### 2. Подготовка на Frontend

```bash
# От главната директория
./build-frontend.sh

# Или ръчно:
cd frontend
npm install
npm run build

# Резултат: frontend/dist/ папка (~5MB)
```

### 3. Файлове за production сървър

```
production/
├── backend             # Rust бинарен файл (20MB)
├── configdb.json       # Конфигурация (1KB)
└── www/               # Frontend файлове от dist/ (5MB)
    ├── index.html
    └── assets/
        ├── index-xxx.js
        └── index-xxx.css

Общо: ~25MB (вместо 500+ MB с node_modules)
```

### 4. Копиране към сървър

```bash
# Създайте deployment пакет
mkdir -p deployment/www
cp backend/target/release/backend deployment/
cp configdb.json deployment/
cp -r frontend/dist/* deployment/www/

# Архивирайте за трансфер
tar -czf rs-ac-bg-deploy.tar.gz deployment/

# Качете на сървъра
scp rs-ac-bg-deploy.tar.gz user@server:/path/
```

### 5. На production сървъра

```bash
# Разархивирайте
tar -xzf rs-ac-bg-deploy.tar.gz

# Стартирайте backend
./deployment/backend

# Настройте nginx за frontend
sudo cp -r deployment/www/* /var/www/html/
```

## Файлова структура - обяснение

### Development (голямо - 500+ MB):
```
frontend/
├── node_modules/     # 500+ MB - хиляди npm пакети
├── src/              # 2MB - вашият код
├── dist/             # 5MB - компилиран код
└── package.json      # конфигурация
```

### Production (малко - 5MB):
```
www/
├── index.html        # 2KB - входна точка
└── assets/          
    ├── index.js      # 3MB - целият код в 1 файл
    └── index.css     # 100KB - всички стилове
```

## Защо е така?

1. **node_modules/** съдържа ВСИЧКИ зависимости за разработка:
   - React развойни инструменти
   - Vite build система
   - ESLint, Prettier
   - Хиляди под-зависимости

2. **dist/** съдържа САМО компилиран, минифициран код:
   - Всичко е събрано в няколко файла
   - Премахнат е development код
   - Оптимизиран е за бързо зареждане

## TL;DR - Супер кратко

```bash
# Build
cd frontend && npm run build

# Deploy - копирайте САМО:
frontend/dist/* → /var/www/html/

# НЕ копирайте:
frontend/node_modules/ ❌
frontend/src/ ❌
```