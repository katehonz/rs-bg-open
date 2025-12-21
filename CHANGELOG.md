# Changelog

Всички значими промени в този проект ще бъдат документирани в този файл.

## [Unreleased]

### Changed - 2025-11-21

#### 🔐 Company Access Control за Администратори

Подобрен контрол на достъпа до компании за различни типове потребители.

**Backend Changes:**
- `backend/src/graphql/admin_resolvers.rs`:
  - Имплементирано специално поведение за `companies()` query
  - Superadmin и admin потребители виждат всички компании в системата
  - Обикновени потребители виждат само компании, към които са добавени в `user_companies`
  - Добавена проверка дали списъкът с company_ids е празен преди заявка към базата данни

**Поведение:**
```rust
// Superadmin/Admin: Връщат всички компании
if user_group.name == "superadmin" || user_group.name == "admin" {
    return CompanyEntity::find().all(db).await;
}

// Други потребители: Само компании от user_companies
let user_companies = UserCompanyEntity::find()
    .filter(user_company::Column::UserId.eq(current_user.id))
    .filter(user_company::Column::IsActive.eq(true))
    .all(db).await;
```

**Impact:**
- Административните потребители могат да управляват всички компании без да се добавят ръчно към всяка
- Подобрена сигурност - обикновените потребители виждат само компаниите, до които имат достъп
- Съответствие с принципа на least privilege за не-административни потребители

**Документация:**
- Актуализиран `docs/AUTHENTICATION.md` с подробно описание на поведението

### Added - 2025-11-20

#### 📈 Корекции на стойността на ДМА (Asset Value Adjustments)

Имплементирана пълна функционалност за корекции на стойността на дълготрайните активи съгласно изискванията на SAF-T и ЗКПО.

**Backend Features:**

**Database & Entities:**
- `migration/src/m20251120_000001_create_asset_value_adjustments.rs` - Нова таблица за корекции
- `backend/src/entities/asset_value_adjustment.rs` - Entity модел с типове корекции:
  - Увеличение (Improvement)
  - Намаление (Impairment)
  - Придобиване (Acquisition)
  - Преоценка (Revaluation)
  - Отписване (Disposal)
  - Вътрешен трансфер (Internal Transfer)
  - Брак (Scrap)

**Services:**
- `backend/src/services/asset_adjustment_service.rs` - Бизнес логика:
  - `apply_improvement()` - Увеличение на стойността с автоматично изчисляване на амортизация
  - `apply_impairment()` - Намаление на стойността
  - Автоматично създаване на счетоводни операции (журнални записи)
  - Поддръжка на промяна на амортизационни норми при увеличение
  - Проследяване на счетоводна и данъчна стойност (преди/след)

**GraphQL API:**
- `backend/src/graphql/fixed_assets_resolvers.rs`:
  - Mutations:
    - `applyAssetImprovement(input: ApplyImprovementInput!)` - Прилагане на увеличение
    - `applyAssetImpairment(input: ApplyImpairmentInput!)` - Прилагане на намаление
  - Queries:
    - `assetAdjustments(fixedAssetId: Int!)` - История на корекции за актив
    - `companyAssetAdjustments(companyId: Int!, fromDate, toDate, adjustmentType)` - Филтрирани корекции

**Frontend Features:**

**Компоненти:**
- `frontend/src/components/fixedAssets/AssetAdjustmentModal.jsx` - Modal за прилагане на корекции:
  - Визуализация на текуща счетоводна и данъчна стойност
  - Избор на тип корекция (увеличение/намаление)
  - Въвеждане на сума, документ, основание
  - За увеличения: възможност за промяна на норми и срок на амортизация
  - Предупреждение за автоматично изчисляване на амортизация

- `frontend/src/components/fixedAssets/AssetAdjustmentsHistory.jsx` - История на корекциите:
  - Филтриране по тип, период (от/до дата)
  - Статистика: общо увеличения, намаления, брой корекции
  - Таблица със стойности преди/след (счетоводни и данъчни)
  - Статус: приключен/неприключен
  - Индикация за променени норми

**UI Updates:**
- `frontend/src/components/fixedAssets/AssetsList.jsx`:
  - Нови бутони за всеки актив: 📈 Увеличение, 📉 Намаление
  - Интеграция с AssetAdjustmentModal

- `frontend/src/pages/FixedAssets.jsx`:
  - Нов таб "Корекции" в модула за ДМА
  - Визуализация на историята на корекциите

**Счетоводни операции:**
- Увеличение: Dt Актив / Ct Доставчици (или конфигуриран сметка)
- Намаление: Dt Разходи / Ct Амортизация

**SAF-T Compliance:**
- Пълна съвместимост със SAF-T изискванията за Asset Transactions
- Проследяване на всички видове движения на активи
- Запазване на одитна следа (before/after values)

**Bug Fixes:**
- Поправени E0716 lifetime errors в 6 файла:
  - `backend/src/services/depreciation_service.rs`
  - `backend/src/services/asset_adjustment_service.rs`
  - `backend/src/entities/company.rs`
  - `backend/src/entities/counterpart.rs`
  - `backend/src/entities/entry_line.rs`
  - `backend/src/entities/account.rs`
- Поправени E0308 type mismatch errors в `asset_adjustment_service.rs`

### Added - 2025-11-18

#### 🔧 Production Module - Backend Implementation

Имплементиран пълен backend за производствения модул с GraphQL API.

**Backend Entity Models:**
- `backend/src/entities/technology_card.rs` - Технологични карти
- `backend/src/entities/technology_card_stage.rs` - Етапи на картите
- `backend/src/entities/production_batch.rs` - Производствени партиди
- `backend/src/entities/production_batch_stage.rs` - Етапи на партидите

**GraphQL API:**
- `backend/src/graphql/production_resolvers.rs` - Resolvers за production модула

**Queries:**
- `technologyCards(companyId: Int!)` - Списък с технологични карти
- `technologyCard(id: Int!)` - Една карта с етапи
- `technologyCardStages(technologyCardId: Int!)` - Етапи на карта

**Mutations:**
- `createTechnologyCardWithStages(input: CreateTechnologyCardWithStagesInput!)` - Създаване на карта с етапи
- `updateTechnologyCard(input: UpdateTechnologyCardInput!)` - Обновяване на карта
- `updateTechnologyCardStages(technologyCardId: Int!, stages: [StageInput!]!)` - Обновяване на етапи
- `deleteTechnologyCard(id: Int!)` - Изтриване на карта

**Frontend Updates:**
- Свързан `TechnologyCards.jsx` с GraphQL API
- Пълна CRUD функционалност (Create, Read, Update, Delete)
- Real-time зареждане на данни от backend
- Валидация и error handling

### Added - 2025-11-08

#### 🏭 Производствен модул

Добавен нов модул за производствено счетоводство с технологични карти и автоматично създаване на счетоводни операции.

**Функционалност:**
- Технологични карти (рецепти) с множество етапи
- Автоматично създаване на счетоводни операции при завършване на етап
- Формули за изчисление на количества и суми
- Производствени партиди с проследяване на статус
- Справки за производство по период и технологична карта

**Database Schema:**
- **Migration:** `migration/production_module.sql`
  - Таблица `technology_cards` - Технологични карти
  - Таблица `technology_card_stages` - Етапи на картите
  - Таблица `production_batches` - Производствени партиди
  - Таблица `production_batch_stages` - Етапи на партидите с връзка към journal_entries

**Frontend компоненти:**
- **Технологични карти:** `frontend/src/pages/production/TechnologyCards.jsx`
  - Създаване и редакция на карти
  - Дефиниране на етапи с дебит/кредит сметки
  - Задаване на формули за количества и суми

- **Производствени партиди:** `frontend/src/pages/production/ProductionBatches.jsx`
  - Създаване на партида по технологична карта
  - Въвеждане на входящо количество
  - Проследяване на статус

- **Справки:** `frontend/src/pages/production/ProductionReports.jsx`
  - Справка за партиди по период
  - Производство по технологични карти
  - Счетоводни операции от производство

**Навигация:**
- Добавено меню "Производство" в главната навигация
- 3 таба: Технологични карти, Производствени партиди, Справки

**Backend (TODO):**
- GraphQL schema и resolvers
- Formula evaluator за изчисления
- Автоматично създаване на journal entries

**Документация:**
- Добавена пълна документация: `docs/PRODUCTION_MODULE.md`
- Актуализиран `docs/README.md` с линк към модула

**Fix:**
- ❌ **Премахнат CompanyContext.jsx** - Причиняваше infinite reload loop
  - Изтрита директория `frontend/src/contexts/`
  - Премахнати import-и от JournalEntry.jsx, Banks.jsx, VATEntry.jsx
  - Проблемът беше `setInterval` създаващ нови intervals при всеки render

### Added - 2025-11-01

#### 🔐 Recovery Code Система

Добавена пълнофункционална система за възстановяване на забравени пароли чрез recovery code.

**Функционалност:**
- Генериране на персонален recovery code от User Profile
- Bcrypt хеширане за сигурно съхранение
- 90 дни срок на валидност
- One-time use механизъм (автоматично изтриване след употреба)
- Формат валидация: XXXX-XXXX (8 символа с тире)

**Backend промени:**
- **Database Migration:** `migration/src/m20251101_000002_add_recovery_code_fields.rs`
  - Добавени полета `recovery_code_hash` и `recovery_code_created_at` в `users` таблица
  - Тип: `VARCHAR NULL` и `TIMESTAMPTZ NULL`

- **Entity Updates:** `backend/src/entities/user.rs`
  - Нови полета в User модел
  - Методи `verify_recovery_code()` и `is_recovery_code_expired()`
  - Bcrypt верификация на кодове

- **GraphQL API:** `backend/src/graphql/user_resolvers.rs`
  - Мутация `generateRecoveryCode` за генериране на кодове
  - Изисква автентикация (JWT token)
  - Автоматично хеширане и съхранение

- **REST API:** `backend/src/rest/auth_api.rs`
  - Endpoint `/api/auth/recover-password` за публична употреба
  - Multi-step валидация (формат, експирация, bcrypt verify)
  - One-time use логика (изтриване след употреба)

**Frontend промени:**
- **User Profile:** `frontend/src/components/UserProfile.jsx`
  - UI за генериране на recovery code
  - Показване на кода само веднъж
  - Copy-to-clipboard функционалност
  - Предупреждения за сигурност

- **Login Page:** `frontend/src/components/Login.jsx`
  - "Forgot Password?" линк
  - Форма за въвеждане на username, recovery code и нова парола
  - Real-time валидация на формата
  - Error handling и user feedback

**Документация:**
- Добавена секция в `README.md` с преглед на Recovery Code системата
- Създаден `docs/RECOVERY_CODE.md` с пълна техническа документация
- API примери (GraphQL + REST)
- Тестови сценарии
- Security best practices

**Сигурност:**
- Bcrypt хеширане с DEFAULT_COST=12
- 90-дневна експирация
- One-time use (изтриване след успешна употреба)
- Формат валидация (защита срещу инжектиране)
- Препоръки за rate limiting

**Тестове:**
- ✅ Тестван пълен флоу (generate → recover → login)
- ✅ Валидация на всички error cases
- ✅ One-time use механизъм
- ✅ Експирация проверка
- ✅ Database schema корекция (TIMESTAMP → TIMESTAMPTZ)

**Известни проблеми (fixed):**
- ❌ GraphQL context грешка - Fixed: Използване на `AuthenticatedUser` вместо `user::Model`
- ❌ Timestamp type mismatch - Fixed: Миграция с `timestamp_with_time_zone()`
- ❌ PostgreSQL cache проблем - Fixed: Restart на service след schema промени

**Breaking Changes:**
- Няма

**Migration Required:**
- ✅ Да - Автоматична миграция при startup на backend
- Migration file: `m20251101_000002_add_recovery_code_fields.rs`

---

## Предишни версии

### [1.0.0] - 2024-XX-XX
- Initial release
- Пълен български сметкоплан
- ДДС обработка v2.0
- INTRASTAT модул
- И други функционалности...
