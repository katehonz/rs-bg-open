# Ръководство за инсталация на RS-AC-BG

## Системни изисквания

- PostgreSQL 14+ 
- Rust 1.75+
- Node.js 22 LTS (напр. 22.20.0)
- npm или yarn

## Стъпка 1: Инсталиране на PostgreSQL

### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
```

### MacOS
```bash
brew install postgresql
brew services start postgresql
```

### Windows
Изтеглете и инсталирайте от: https://www.postgresql.org/download/windows/

## Стъпка 2: Създаване на база данни

```bash
# Влезте в PostgreSQL
sudo -u postgres psql

# Създайте база данни
CREATE DATABASE rs_ac_bg;

# Създайте потребител (ако е необходимо)
CREATE USER postgres WITH PASSWORD 'pas+123';

# Дайте права на потребителя
GRANT ALL PRIVILEGES ON DATABASE rs_ac_bg TO postgres;

# Излезте
\q
```

## Стъпка 3: Конфигурация на системата

### 3.1 Създайте конфигурационен файл

Копирайте примерния конфигурационен файл:
```bash
cp configdb.example.json configdb.json
```

### 3.2 Редактирайте `configdb.json`

```json
{
  "database": {
    "host": "localhost",
    "port": 5432,
    "database": "rs_ac_bg",
    "username": "postgres",
    "password": "pas+123",  // Вашата парола тук
    "max_connections": 10,
    "min_connections": 5,
    "connect_timeout": 10,
    "acquire_timeout": 10,
    "idle_timeout": 600
  },
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "workers": 4,
    "enable_cors": true,
    "cors_origins": ["http://localhost:3000", "http://localhost:5173"]
  },
  "logging": {
    "level": "info",
    "file": "logs/backend.log",
    "stdout": true
  },
  "initial_setup": {
    "create_demo_company": true,  // При първо стартиране създава демо фирма
    "demo_company_name": "Демо Фирма ООД",
    "demo_company_eik": "123456789",
    "demo_company_vat": "BG123456789",
    "load_chart_of_accounts": true  // Автоматично зарежда сметкоплана
  }
}
```

## Стъпка 4: Компилиране на backend

```bash
cd backend
cargo build --release
```

Компилираният бинарен файл ще се намира в: `target/release/backend`

## Стъпка 5: Първоначално зареждане на системата

### Какво се случва при първо стартиране?

1. **SeaORM миграции** - Системата автоматично изпълнява всички миграции от папка `migration/src/`:
   - Създава таблици за фирми, потребители, сметки, журнални записи, контрагенти, ДДС и др.
   - Добавя необходимите индекси и връзки между таблиците
   - НЕ се изискват SQL скриптове - всичко се управлява от SeaORM

2. **Демо фирма** - Ако `create_demo_company` е `true` в конфигурацията:
   - Създава "Демо Фирма ООД" с ЕИК и ДДС номер
   - Автоматично зарежда пълен български сметкоплан

3. **Сметкоплан** - При създаване на нова фирма:
   - Автоматично се зарежда йерархичен сметкоплан
   - Включва всички класове (1-7) с подсметки
   - Настройва ДДС приложимост за съответните сметки

### Стартиране на системата

```bash
# От главната директория
./target/release/backend

# Или с cargo
cargo run --release
```

При първо стартиране ще видите в лога:
```
INFO: Configuration loaded from file
INFO: Database connected and migrations applied
INFO: Successfully loaded chart of accounts for company 1
INFO: Server running at http://0.0.0.0:8080
```

## Стъпка 6: Инсталиране на frontend

```bash
cd frontend
npm install
npm run dev
```

Frontend-ът ще стартира на: http://localhost:5173

## Стъпка 7: Проверка на инсталацията

1. Отворете браузър на http://localhost:8080/graphiql
2. Изпълнете тестова заявка:

```graphql
query {
  companies {
    id
    name
    eik
    vatNumber
  }
}
```

Трябва да видите демо фирмата.

3. Проверка на сметкоплана:

```graphql
query {
  accounts(companyId: 1) {
    id
    code
    name
    accountType
    isAnalytical
  }
}
```

## Структура на файловете

```
rs-ac-bg/
├── configdb.json           # Конфигурационен файл (създайте от example)
├── configdb.example.json   # Примерен конфигурационен файл
├── backend/
│   ├── src/
│   │   ├── config.rs      # Чете конфигурацията
│   │   ├── db.rs          # Инициализира базата и миграциите
│   │   ├── data/
│   │   │   └── chart_of_accounts.rs  # Вграден сметкоплан
│   │   └── main.rs
│   └── target/
│       └── release/
│           └── backend    # Компилиран бинарен файл
├── migration/
│   └── src/              # SeaORM миграции (автоматични)
└── frontend/
```

## Важни бележки

1. **Без SQL скриптове** - Системата използва SeaORM миграции, които автоматично създават и обновяват схемата на базата данни

2. **Конфигурация** - Системата търси `configdb.json` в следния ред:
   - В текущата директория
   - В родителската директория
   - Ако не намери, използва `configdb.example.json`

3. **Сигурност** - НЕ включвайте `configdb.json` в git repository-то, тъй като съдържа пароли

4. **Логове** - Проверявайте `logs/backend.log` за подробна информация при проблеми

## Отстраняване на проблеми

### Грешка при връзка с базата данни
- Проверете дали PostgreSQL работи
- Проверете потребителското име и паролата в `configdb.json`
- Проверете дали базата данни `rs_ac_bg` съществува

### Миграциите не се изпълняват
- Проверете логовете за грешки
- Уверете се, че потребителят има права върху базата данни
- Опитайте: `DROP DATABASE rs_ac_bg; CREATE DATABASE rs_ac_bg;` и стартирайте отново

### Сметкопланът не се зарежда
- Проверете `initial_setup.load_chart_of_accounts` в конфигурацията
- Проверете логовете за грешки при зареждането
