# Frontend Deployment Ръководство

## Общ преглед

Frontend-ът е React приложение, което използва Vite за build процеса. За production НЕ са нужни всички файлове - само компилираните.

## Развойна среда (Development)

### Необходими файлове за разработка:
```
frontend/
├── src/                 # Изходен код (НЕ е нужен за production)
├── node_modules/        # Зависимости (НЕ е нужен за production)
├── package.json         # Конфигурация
├── vite.config.js       # Vite конфигурация
└── index.html           # Входна точка
```

### Стартиране в development режим:
```bash
cd frontend
npm install  # Инсталира всички зависимости
npm run dev  # Стартира development сървър
```

## Production Build

### Стъпка 1: Създаване на production build

```bash
cd frontend
npm install        # Ако не сте инсталирали зависимостите
npm run build      # Създава оптимизирани файлове в папка dist/
```

### Стъпка 2: Какво създава build командата?

След `npm run build` се създава папка `dist/` със следната структура:

```
frontend/dist/
├── index.html           # Главен HTML файл
├── assets/
│   ├── index-[hash].js  # Компилиран JavaScript код
│   ├── index-[hash].css # Компилирани стилове
│   └── [други assets]    # Изображения, шрифтове и др.
└── favicon.ico          # Икона на сайта
```

## Кои файлове са нужни за Production?

### САМО папка `dist/` е нужна за production!

```bash
# За production копирайте САМО съдържанието на dist/
frontend/dist/  # ✅ САМО ТОВА е нужно
```

### НЕ са нужни за production:
```bash
frontend/src/          # ❌ Изходен код
frontend/node_modules/ # ❌ Node.js зависимости (огромна папка!)
frontend/package.json  # ❌ Не е нужен след build
frontend/*.config.js   # ❌ Конфигурационни файлове
```

## Методи за deployment

### Метод 1: Nginx (Препоръчителен)

1. Инсталирайте Nginx:
```bash
sudo apt install nginx
```

2. Копирайте `dist/` файловете:
```bash
sudo cp -r frontend/dist/* /var/www/html/rs-ac-bg/
```

3. Конфигурирайте Nginx (`/etc/nginx/sites-available/rs-ac-bg`):
```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /var/www/html/rs-ac-bg;
    index index.html;

    # React Router support
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy към backend
    location /graphql {
        proxy_pass http://localhost:8080/graphql;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

4. Активирайте сайта:
```bash
sudo ln -s /etc/nginx/sites-available/rs-ac-bg /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

### Метод 2: Apache

1. Копирайте файловете:
```bash
sudo cp -r frontend/dist/* /var/www/html/rs-ac-bg/
```

2. Създайте `.htaccess` файл в `/var/www/html/rs-ac-bg/`:
```apache
<IfModule mod_rewrite.c>
  RewriteEngine On
  RewriteBase /
  RewriteRule ^index\.html$ - [L]
  RewriteCond %{REQUEST_FILENAME} !-f
  RewriteCond %{REQUEST_FILENAME} !-d
  RewriteRule . /index.html [L]
</IfModule>
```

### Метод 3: Статичен хостинг (Netlify, Vercel)

Просто качете съдържанието на `dist/` папката.

### Метод 4: Обслужване от Rust backend

Добавете в backend-а:
```rust
// Сервиране на статични файлове
app.service(
    actix_files::Files::new("/", "./frontend/dist")
        .index_file("index.html")
        .use_last_modified(true)
)
```

## Конфигурация на API endpoint

Преди build, уверете се че API endpoint-ът е правилен:

### Редактирайте `frontend/src/utils/graphqlClient.js`:
```javascript
const client = new ApolloClient({
  uri: process.env.NODE_ENV === 'production' 
    ? '/graphql'  // За production (през nginx proxy)
    : 'http://localhost:8080/graphql', // За development
  cache: new InMemoryCache(),
});
```

## Build скрипт за автоматизация

Създайте `build-frontend.sh`:
```bash
#!/bin/bash

echo "🚀 Building RS-AC-BG Frontend for Production..."

# Отиди в frontend директорията
cd frontend

# Инсталирай зависимости ако липсват
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Изтрий стария build
rm -rf dist

# Създай нов production build
echo "🔨 Building production bundle..."
npm run build

# Провери дали build-ът е успешен
if [ -d "dist" ]; then
    echo "✅ Build successful!"
    echo "📁 Production files are in: frontend/dist/"
    echo ""
    echo "📊 Build size:"
    du -sh dist/
    echo ""
    echo "📝 Files created:"
    ls -la dist/
else
    echo "❌ Build failed!"
    exit 1
fi
```

## Оптимизация на размера

### Анализ на bundle размера:
```bash
npm run build -- --analyze
```

### Препоръки за намаляване на размера:
1. Използвайте dynamic imports за code splitting
2. Премахнете неизползвани зависимости
3. Активирайте gzip/brotli компресия в nginx

## Структура на deployment

```
production-server/
├── backend/
│   ├── backend         # Rust бинарен файл
│   └── configdb.json   # Конфигурация
├── frontend/           # САМО dist/ съдържанието
│   ├── index.html
│   └── assets/
│       ├── index-[hash].js
│       └── index-[hash].css
└── logs/
    └── backend.log
```

## Важни бележки

1. **НЕ качвайте `node_modules/`** - Тази папка може да е над 500MB!
2. **Само `dist/` папката е нужна** - Обикновено е под 5MB
3. **Hash в имената** - Файловете имат hash за cache busting
4. **Environment Variables** - Конфигурирайте API endpoints преди build

## Проверка на deployment

1. Отворете браузър на вашия домейн
2. Проверете Network tab в DevTools - трябва да виждате:
   - index.html (< 10KB)
   - index-[hash].js (< 500KB обикновено)
   - index-[hash].css (< 50KB)
   - /graphql заявки към backend

## Обновяване на production

```bash
# 1. Направете нов build
cd frontend && npm run build

# 2. Копирайте новите файлове
sudo rm -rf /var/www/html/rs-ac-bg/*
sudo cp -r dist/* /var/www/html/rs-ac-bg/

# 3. Рестартирайте nginx (ако използвате)
sudo systemctl restart nginx
```