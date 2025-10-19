# INTRASTAT Модул - Инсталационна инструкция

## 🚀 Бърза инсталация

### 1. Изпълнете SQL миграцията
```bash
psql -d your_accounting_db -f migration/intrastat_schema.sql
```

### 2. Компилирайте backend-а
```bash
cd backend
cargo build --release
```

### 3. Стартирайте сървъра
```bash
cargo run
```

### 4. Проверете frontend-а
```bash
cd frontend
npm run dev
```

### 5. Отворете браузър
- Идете на `http://localhost:5173`
- В меню "Счетоводство" ще видите "🌍 INTRASTAT"

## ✅ Проверка на инсталацията

1. **Backend:** `http://localhost:8000/graphql` - трябва да се зареди GraphQL playground
2. **Frontend:** Трябва да видите INTRASTAT в навигацията
3. **База данни:** Проверете че новите таблици са създадени:

```sql
SELECT table_name FROM information_schema.tables 
WHERE table_name LIKE 'intrastat_%';
```

Трябва да видите:
- `intrastat_nomenclature`
- `intrastat_account_mapping` 
- `intrastat_declaration`
- `intrastat_declaration_item`
- `intrastat_settings`

## 🔧 Първоначална настройка

1. Влезте в **INTRASTAT > Настройки**
2. Активирайте модула
3. Въведете данните за отговорното лице
4. Импортирайте CN номенклатура от `INTRASTAT/CN_2025_NAP.csv`

## 📋 Файлове за импорт

В папка `INTRASTAT/` има примерни файлове:
- `CN_2025_NAP.csv` - CN номенклатура за 2025
- `Copy+of+Класификатор+на+вида+на+сделката_2022-1.csv` - Видове сделки
- `Format+na+XML+Bg_2022.html` - Документация на НАП

## 🎯 Следващи стъпки

1. Настройте праговете (по подразбиране 400,000 лв.)
2. При нужда свържете количествени сметки с номенклатура
3. Тествайте създаването на декларация
4. Експортирайте пробен XML файл

---
📖 За подробна документация вижте [INTRASTAT_MODULE.md](./INTRASTAT_MODULE.md)