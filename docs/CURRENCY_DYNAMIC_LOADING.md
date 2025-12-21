# Динамично зареждане на валути в счетоводни записи

## Преглед

От версия 0.1.0+ системата динамично зарежда активните валути от базата данни в формулярите за създаване на счетоводни записи. Това премахва необходимостта от хардкодени списъци с валути и осигурява автоматична синхронизация с Currency Management модула.

## Проблем

**Преди:** Формулярите JournalEntry.jsx и VATEntry.jsx използваха хардкоден списък с валути:

```javascript
const [currencies] = useState([
  { code: 'EUR', name: 'Евро' },
  { code: 'USD', name: 'Долар САЩ' },
  { code: 'GBP', name: 'Британска лира' }
]);
```

**Последици:**
- ❌ Невъзможно добавяне на нови валути без промяна на код
- ❌ Справките не показват данни за валути, които липсват в хардкодения списък
- ❌ Десинхронизация между Currency Management и записите
- ❌ Хардкодени обменни курсове вместо актуални от БНБ/ECB

## Решение

**Сега:** Валутите и курсовете се зареждат динамично при стартиране на компонента:

```javascript
const [currencies, setCurrencies] = useState([]);
const [exchangeRates, setExchangeRates] = useState({});

useEffect(() => {
  loadCurrenciesAndRates();
}, []);
```

## Архитектура

### Frontend (React)

**Файлове:**
- `/frontend/src/pages/JournalEntry.jsx` - Счетоводни записи
- `/frontend/src/pages/VATEntry.jsx` - ДДС записи

**GraphQL Заявки:**

```graphql
# Зареждане на активни валути
query GetActiveCurrencies {
  currencies {
    id
    code
    name
    nameBg
    isActive
  }
}

# Зареждане на обменни курсове
query GetExchangeRates {
  currenciesWithRates {
    currency {
      id
      code
      name
      nameBg
    }
    latestRate
  }
}
```

**State Management:**

```javascript
// Валути (филтрирани само активни, без BGN)
const [currencies, setCurrencies] = useState([]);

// Обменни курсове (map: code -> rate)
const [exchangeRates, setExchangeRates] = useState({});
```

### Backend (Rust + SeaORM)

**GraphQL Resolvers:**
- `currencies` - Връща всички валути (може да се филтрира по `isActive`)
- `currenciesWithRates` - Връща валути + последен актуален курс

**База данни:**

```sql
-- Таблица currencies
CREATE TABLE currencies (
    id SERIAL PRIMARY KEY,
    code VARCHAR(3) NOT NULL,        -- EUR, USD, GBP
    name VARCHAR NOT NULL,
    name_bg VARCHAR NOT NULL,
    is_active BOOLEAN DEFAULT true,
    bnb_code VARCHAR(3),
    -- ...
);

-- Таблица exchange_rates
CREATE TABLE exchange_rates (
    id SERIAL PRIMARY KEY,
    from_currency_id INTEGER REFERENCES currencies(id),
    to_currency_id INTEGER REFERENCES currencies(id),
    rate NUMERIC(20, 10) NOT NULL,
    reverse_rate NUMERIC(20, 10) NOT NULL,
    valid_date DATE NOT NULL,
    rate_source VARCHAR NOT NULL,  -- 'Bnb' или 'Ecb'
    -- ...
);
```

## Имплементация

### Стъпка 1: Добавяне на GraphQL заявки

**JournalEntry.jsx и VATEntry.jsx:**

```javascript
const ACTIVE_CURRENCIES_QUERY = `
  query GetActiveCurrencies {
    currencies {
      id
      code
      name
      nameBg
      isActive
    }
  }
`;

const EXCHANGE_RATES_QUERY = `
  query GetExchangeRates {
    currenciesWithRates {
      currency {
        id
        code
        name
        nameBg
      }
      latestRate
    }
  }
`;
```

### Стъпка 2: Добавяне на state променливи

```javascript
const [currencies, setCurrencies] = useState([]);
const [exchangeRates, setExchangeRates] = useState({});
```

### Стъпка 3: Функция за зареждане

```javascript
const loadCurrenciesAndRates = async () => {
  try {
    // Зареди активни валути
    const currenciesData = await graphqlRequest(ACTIVE_CURRENCIES_QUERY);
    const activeCurrencies = (currenciesData.currencies || [])
      .filter(c => c.isActive && c.code !== 'BGN')
      .map(c => ({ code: c.code, name: c.nameBg || c.name }));
    setCurrencies(activeCurrencies);

    // Зареди обменни курсове
    const ratesData = await graphqlRequest(EXCHANGE_RATES_QUERY);
    const ratesMap = {};
    (ratesData.currenciesWithRates || []).forEach(item => {
      if (item.latestRate) {
        ratesMap[item.currency.code] = parseFloat(item.latestRate);
      }
    });
    setExchangeRates(ratesMap);

  } catch (err) {
    console.error('Грешка при зареждане на валути:', err.message);

    // Fallback към 3 основни валути
    setCurrencies([
      { code: 'EUR', name: 'Евро' },
      { code: 'USD', name: 'Долар САЩ' },
      { code: 'GBP', name: 'Британска лира' }
    ]);
    setExchangeRates({
      'EUR': 1.9558,
      'USD': 1.8234,
      'GBP': 2.3567
    });
  }
};
```

### Стъпка 4: Извикване в useEffect

```javascript
useEffect(() => {
  const loadData = async () => {
    await loadAccounts();
    await loadCounterparts();
    await loadCurrenciesAndRates();  // ← ДОБАВЕНО

    // Check for edit mode from URL parameters
    const urlParams = new URLSearchParams(window.location.search);
    const editId = urlParams.get('edit');
    if (editId) {
      setEditingEntryId(parseInt(editId));
      setIsEditMode(true);
      await loadEntryForEdit(parseInt(editId));
    }
  };

  loadData();
}, []);
```

### Стъпка 5: Динамично dropdown меню

**Преди:**
```jsx
<select value={line.currencyCode} onChange={(e) => handleCurrencyChange(index, e.target.value)}>
  <option value="BGN">BGN</option>
  <option value="EUR">EUR</option>
  <option value="USD">USD</option>
  <option value="GBP">GBP</option>
</select>
```

**След:**
```jsx
<select value={line.currencyCode} onChange={(e) => handleCurrencyChange(index, e.target.value)}>
  <option value="BGN">BGN</option>
  {currencies.map(currency => (
    <option key={currency.code} value={currency.code}>
      {currency.code}
    </option>
  ))}
</select>
```

### Стъпка 6: Динамични обменни курсове

**Преди:**
```javascript
const handleCurrencyChange = async (lineIndex, currencyCode) => {
  const rates = { 'EUR': 1.9558, 'USD': 1.8234, 'GBP': 2.3567 };
  const rate = currencyCode === 'BGN' ? 1 : (rates[currencyCode] || 1);
  // ...
};
```

**След:**
```javascript
const handleCurrencyChange = async (lineIndex, currencyCode) => {
  const rate = currencyCode === 'BGN' ? 1 : (exchangeRates[currencyCode] || 1);
  // ...
};
```

## Поток на данни

```
┌─────────────────────────────────────────────────────────────┐
│  1. User отваря /accounting/entries или /vat-entry          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  2. useEffect извиква loadCurrenciesAndRates()              │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  3. GraphQL заявка към backend                              │
│     GET /graphql?query=GetActiveCurrencies                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  4. Backend Query Resolver                                  │
│     currency::Entity::find()                                │
│       .filter(is_active = true)                             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  5. Връща се JSON с валути:                                 │
│     [{ code: "EUR", name: "Евро" },                         │
│      { code: "USD", name: "Долар САЩ" }, ...]              │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  6. GraphQL заявка за курсове                               │
│     GET /graphql?query=GetExchangeRates                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  7. Backend Resolver за курсове                             │
│     exchange_rate::Entity::find()                           │
│       .order_by_desc(valid_date)                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  8. Връща се JSON с курсове:                                │
│     { "EUR": 1.9558, "USD": 1.7223, ... }                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  9. Frontend задава state:                                  │
│     setCurrencies([...])                                    │
│     setExchangeRates({...})                                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  10. React рендерира dropdown с валути                      │
└─────────────────────────────────────────────────────────────┘
```

## Fallback механизъм

Ако API заявката се провали (например сървърът е недостъпен), системата използва fallback списък с 3 основни валути:

```javascript
// Fallback currencies
setCurrencies([
  { code: 'EUR', name: 'Евро' },
  { code: 'USD', name: 'Долар САЩ' },
  { code: 'GBP', name: 'Британска лира' }
]);

// Fallback exchange rates
setExchangeRates({
  'EUR': 1.9558,
  'USD': 1.8234,
  'GBP': 2.3567
});
```

**Причини за fallback:**
- Сървърът е недостъпен
- GraphQL resolver връща грешка
- Няма активни валути в базата
- Мрежова грешка

**Поведение:**
- Показва се грешка в console.error
- Зарежда се минимален списък от 3 валути
- Потребителят може да работи с основните валути
- При възстановяване на API, рефреш на страницата ще зареди правилните данни

## Предимства

### ✅ Гъвкавост
- Лесно добавяне на нови валути чрез миграция
- Няма нужда от промени в frontend кода
- Автоматична синхронизация между модули

### ✅ Актуалност
- Използват се актуални курсове от БНБ/ECB
- Не са хардкодени стойности
- Курсовете се обновяват ежедневно

### ✅ Консистентност
- Едно място за управление на валути (Currency Management)
- Промените се отразяват веднага във всички формуляри
- Справките виждат правилните данни

### ✅ Надеждност
- Fallback механизъм при проблеми с API
- Graceful degradation
- Грешките се логват в конзолата

## Тестване

### Тест 1: Основна функционалност

1. Отворете http://localhost/currencies
2. Активирайте валута TRY (Турска лира)
3. Отворете http://localhost/accounting/entries
4. Проверете dropdown за валута → TRY трябва да се вижда
5. Изберете TRY → проверете че курсът се зарежда автоматично

### Тест 2: Деактивиране на валута

1. Отворете http://localhost/currencies
2. Деактивирайте валута GBP
3. Обновете страницата http://localhost/accounting/entries
4. Проверете dropdown → GBP не трябва да се вижда

### Тест 3: Fallback механизъм

1. Спрете backend сървъра: `docker compose stop accounting-service`
2. Обновете http://localhost/accounting/entries
3. Отворете браузър конзола → виждате грешка
4. Проверете dropdown → виждате само EUR, USD, GBP (fallback)
5. Стартирайте сървъра: `docker compose start accounting-service`
6. Обновете страницата → виждате всички активни валути

### Тест 4: Създаване на запис с нова валута

1. Активирайте валута JPY (Японска йена)
2. Обновете курсовете: http://localhost/currencies → "Update Rates"
3. Отворете http://localhost/accounting/entries
4. Създайте нов запис:
   - Сметка Дт: 503 (Каса)
   - Сметка Кт: 411 (Клиенти)
   - Валута: JPY
   - Сума: 10000 JPY
   - Курс: (автоматично зареден)
5. Запазете → Проверете че се вижда в списъка с записи

### Тест 5: Справки

1. Създайте записи в различни валути (EUR, USD, TRY)
2. Отворете http://localhost/reports
3. Генерирайте оборотна ведомост
4. Проверете че се виждат данни за всички използвани валути

## Troubleshooting

### Проблем: Не се виждат валути в dropdown

**Симптом:** Dropdown е празен, само BGN се вижда

**Причини:**
- Няма активни валути в базата
- API връща грешка
- Мрежова грешка

**Решение:**
1. Отворете браузър конзола (F12)
2. Проверете за грешки при зареждане
3. Проверете базата данни:
```sql
SELECT code, name, is_active FROM currencies;
```
4. Ако няма активни валути, активирайте чрез UI или SQL:
```sql
UPDATE currencies SET is_active = true WHERE code IN ('EUR', 'USD', 'GBP');
```

### Проблем: Курсовете са 1.000

**Симптом:** При избор на валута, курсът е 1.000 вместо актуален

**Причини:**
- Няма заредени курсове в `exchange_rates` таблицата
- Курсовете не са обновени

**Решение:**
1. Отворете http://localhost/currencies
2. Натиснете "Update BNB Rates" или "Update ECB Rates"
3. Проверете базата данни:
```sql
SELECT
  c.code,
  er.rate,
  er.valid_date
FROM exchange_rates er
JOIN currencies c ON c.id = er.from_currency_id
WHERE er.valid_date = CURRENT_DATE
ORDER BY c.code;
```

### Проблем: Fallback валути се показват постоянно

**Симптом:** Винаги виждате само EUR, USD, GBP

**Причини:**
- Backend не работи
- GraphQL endpoint не е достъпен
- CORS грешка

**Решение:**
1. Проверете че backend работи:
```bash
docker compose ps accounting-service
```
2. Проверете логове:
```bash
docker compose logs accounting-service -f
```
3. Тествайте GraphQL endpoint:
```bash
curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{ currencies { code } }"}'
```

### Проблем: Нова валута не се вижда

**Симптом:** Добавили сте нова валута, но не се вижда в dropdown

**Причини:**
- Валутата не е активирана (`is_active = false`)
- Не сте обновили страницата
- Кеширане

**Решение:**
1. Проверете дали валутата е активна:
```sql
SELECT code, is_active FROM currencies WHERE code = 'TRY';
```
2. Ако е `false`, активирайте:
```sql
UPDATE currencies SET is_active = true WHERE code = 'TRY';
```
3. Обновете страницата (Ctrl+F5 за hard refresh)

## Бъдещи подобрения

### Кеширане на валути

Валутите се променят рядко, така че може да се добави кеширане:

```javascript
// LocalStorage кеш
const loadCurrenciesAndRates = async () => {
  const cached = localStorage.getItem('currencies_cache');
  const cacheTime = localStorage.getItem('currencies_cache_time');

  // Ако кешът е по-нов от 1 час, използвай го
  if (cached && cacheTime && (Date.now() - parseInt(cacheTime) < 3600000)) {
    setCurrencies(JSON.parse(cached));
    return;
  }

  // Иначе зареди от API
  const data = await graphqlRequest(ACTIVE_CURRENCIES_QUERY);
  localStorage.setItem('currencies_cache', JSON.stringify(data));
  localStorage.setItem('currencies_cache_time', Date.now().toString());
  setCurrencies(data);
};
```

### Реално време обновяване

При активиране/деактивиране на валута в Currency Management, автоматично обновяване на dropdown в отворените записи:

```javascript
// WebSocket subscription
const { data } = useSubscription(CURRENCIES_CHANGED_SUBSCRIPTION);

useEffect(() => {
  if (data) {
    loadCurrenciesAndRates();
  }
}, [data]);
```

### Показване на последна актуализация

```jsx
<div className="text-xs text-gray-500">
  Курсове актуализирани: {lastUpdateDate}
</div>
```

## Референции

- [BASE_CURRENCY_GUIDE.md](./BASE_CURRENCY_GUIDE.md) - Базова валута на компанията
- [ECB_EXCHANGE_RATES.md](./ECB_EXCHANGE_RATES.md) - ECB интеграция
- [JournalEntry.jsx](../frontend/src/pages/JournalEntry.jsx) - Имплементация
- [VATEntry.jsx](../frontend/src/pages/VATEntry.jsx) - Имплементация

---

**Последна актуализация:** Ноември 2025
**Версия:** 1.0.0
**Статус:** Production Ready
