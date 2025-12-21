# Модул Контрагенти (Counterparts)

## Описание

Модулът Контрагенти предоставя функционалност за управление на клиенти, доставчици и други контрагенти в системата. Модулът е пълно SAF-T съвместим и поддържа всички изисквания за финансово отчитане.

## Функционалности

### Основни функции
- **Създаване и редактиране на контрагенти**
- **Търсене и филтриране** по име, ЕИК, ДДС номер или тип
- **Групиране по типове** (Клиент, Доставчик, Служител, Банка, Държавна институция, Друго)
- **SAF-T роли** - флагове за клиент/доставчик
- **ДДС статус** - регистрация в ДДС регистъра
- **Статистики** - общ брой контрагенти по типове

### SAF-T Съвместимост
Модулът включва всички полета, изисквани от SAF-T стандарта:
- **Основни данни**: име, ЕИК, ДДС номер
- **Адресни данни**: адрес, град, държава, пощенски код
- **Контактни данни**: телефон, имейл, лице за контакт
- **Банкови данни**: IBAN, BIC, име на банка
- **SAF-T роли**: is_customer, is_supplier флагове

## Файлове

### Frontend
- `frontend/src/pages/Counterparts.jsx` - основен компонент за управление на контрагенти

### Backend
- `backend/src/entities/counterpart.rs` - entity модел
- `backend/src/graphql/*_resolvers.rs` - GraphQL resolvers (актуализирани за новите полета)
- `migration/src/m20240915_000001_add_customer_supplier_flags.rs` - миграция за SAF-T полета

## API

### GraphQL Queries
```graphql
# Зареждане на контрагенти за фирма
query GetCounterparts($companyId: Int!) {
  counterparts(companyId: $companyId) {
    id
    name
    eik
    vatNumber
    address
    city
    country
    phone
    email
    contactPerson
    counterpartType
    isCustomer
    isSupplier
    isVatRegistered
    isActive
    companyId
    createdAt
    updatedAt
  }
}
```

### GraphQL Mutations
```graphql
# Създаване на нов контрагент
mutation CreateCounterpart($input: CreateCounterpartInput!) {
  createCounterpart(input: $input) {
    id
    name
    eik
    vatNumber
    counterpartType
    isVatRegistered
  }
}

# Актуализиране на контрагент
mutation UpdateCounterpart($id: Int!, $input: UpdateCounterpartInput!) {
  updateCounterpart(id: $id, input: $input) {
    id
    name
    eik
    vatNumber
    counterpartType
    isVatRegistered
  }
}
```

## Използване

### Достъп до модула
Модулът е достъпен на адрес: `http://localhost:5173/counterparts`

### Работен процес
1. **Избор на фирма** - изберете фирма от падащото меню
2. **Преглед на контрагенти** - виждате списъка с всички контрагенти за избраната фирма
3. **Търсене и филтриране** - използвайте полетата за търсене и филтър по тип
4. **Създаване на нов контрагент** - натиснете "Нов контрагент" и попълнете формата
5. **Редактиране** - натиснете "Редактиране" до желания контрагент

### Полета във формата
- **Име/Наименование*** (задължително)
- **Тип контрагент*** (падащо меню)
- **ЕИК** - единен идентификационен код
- **ДДС номер** - номер в ДДС регистъра
- **Адресни данни** - адрес, град, държава
- **Контактни данни** - телефон, имейл, лице за контакт
- **SAF-T роли** - чекбоксове за клиент/доставчик
- **ДДС статус** - чекбокс за ДДС регистрация

## Технически детайли

### База данни
Таблица: `counterparts`

Основни колони:
- `id` - първичен ключ
- `name` - име/наименование
- `eik` - ЕИК
- `vat_number` - ДДС номер
- `counterpart_type` - тип (CUSTOMER, SUPPLIER, EMPLOYEE, BANK, GOVERNMENT, OTHER)
- `is_customer` - булев флаг за клиент (SAF-T)
- `is_supplier` - булев флаг за доставчик (SAF-T)
- `is_vat_registered` - ДДС регистрация
- `company_id` - връзка към фирма

### Валидации
- Име/наименование е задължително
- Тип контрагент е задължителен
- ЕИК и ДДС номер са опционални, но се валидират ако са попълнени
- Email адрес се валидира за формат
- Всички текстови полета имат максимална дължина

## Миграции

### m20240915_000001_add_customer_supplier_flags
Добавя SAF-T съвместимите полета:
```sql
ALTER TABLE counterparts
ADD COLUMN is_customer bool NOT NULL DEFAULT FALSE,
ADD COLUMN is_supplier bool NOT NULL DEFAULT FALSE;
```

## Съвместимост

- **React 18+**
- **Tailwind CSS** за стилизиране
- **GraphQL** за API комуникация
- **PostgreSQL** за база данни
- **Sea-ORM** за ORM в Rust backend

## Известни проблеми

Няма известни проблеми към момента.

## Планирани подобрения

1. Експорт в Excel/CSV формат
2. Импорт от файл
3. История на промените
4. Дублиране на контрагенти
5. Прикачени файлове/документи