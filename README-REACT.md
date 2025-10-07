# RS-AC-BG - React Frontend

## Преглед

Проектът е мигриран от Leptos (Rust) към React (JavaScript) frontend за по-добра стабилност и развитие.

## Структура

```
rs-ac-bg/
├── backend/                 # Rust backend с GraphQL
├── frontend/               # React frontend с Vite
├── frontend-leptos-old/    # Стар Leptos код (запазен)
├── shared/                 # Споделени модели
├── migration/              # Миграции за базата данни
├── run-dev.sh              # Стартиране на dev сървърите
└── stop-dev.sh             # Спиране на dev сървърите
```

## Изисквания

- **Node.js**: v20.19.4+ (инсталиран с nvm)
- **Rust**: 1.70+ 
- **PostgreSQL**: 14+

## Стартиране

### Бързо стартиране:
```bash
./run-dev.sh
```

Това ще стартира:
- Backend на http://localhost:8080
- React Frontend на http://localhost:5173
- GraphQL Playground на http://localhost:8080/graphiql

### Ръчно стартиране:

1. **Backend**:
```bash
cd backend
cargo run
```

2. **Frontend**:
```bash
# Първо настрой Node.js:
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
nvm use 20

# След това стартирай React:
cd frontend
npm run dev
```

## Спиране
```bash
./stop-dev.sh
```

## React Frontend

### Технологии:
- **React 18** - UI framework
- **Vite** - Build tool и dev server
- **React Router v7** - Клиентско маршрутизиране  
- **Tailwind CSS** - Utility-first CSS

### Ключови компоненти:
- `Layout` - Основен layout с sidebar, header, footer
- `Sidebar` - Навигационно меню
- `Dashboard` - Табло с статистики
- `Settings` - Настройки (потребители, фирми, система)

### Развитие:
```bash
cd frontend
npm install              # Инсталиране на dependencies
npm run dev             # Development server
npm run build           # Production build
npm run preview         # Preview на build
```

## Миграция от Leptos

- ✅ Layout компоненти (Header, Sidebar, Footer)
- ✅ Dashboard страница с widgets
- ✅ Settings страници (Users, Companies, System)  
- ✅ Routing и navigation
- ✅ Tailwind CSS стилове
- 🚧 Останали страници (в процес на разработка)

## API Интеграция

Frontend комуникира с Rust backend чрез:
- GraphQL API на port 8080
- REST endpoints за специфични операции

## Порт промени

- **Стар Leptos**: http://localhost:3000  
- **Нов React**: http://localhost:5173
- **Backend**: http://localhost:8080 (непроменен)

## Бележки

- Стария Leptos код е запазен в `frontend-leptos-old/`
- Node.js версията е обновена от v18 към v20 за съвместимост
- Всички оригинални функции са запазени и работят