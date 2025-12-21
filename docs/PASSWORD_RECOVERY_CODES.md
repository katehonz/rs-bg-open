# Система за възстановяване на парола с кодове

## Общ преглед

Системата за възстановяване на парола позволява на потребителите да възстановят достъп до акаунта си без необходимост от email верификация. Вместо това, потребителите генерират код за възстановяване, който съхраняват на сигурно място и използват при нужда.

## Функционалност

### 1. Генериране на код за възстановяване

**Местоположение:** `http://localhost/settings/profile`

Потребителите могат да генерират код за възстановяване от страницата на профила си:

1. Влезте в системата
2. Отидете на **Настройки → Моят профил**
3. Изберете раздела **"Код за възстановяване"**
4. Кликнете на бутона **"🔑 Генерирай код"**

#### Характеристики на кода:
- Формат: `XXXX-XXXX` (8 символа, разделени с тире)
- Използва само не-двусмислени символи: `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`
- Изключени символи: `I`, `L`, `O`, `0`, `1` (за да се избегне объркване)
- Всеки код е уникален и генериран случайно

#### Опции за запазване:
- **📋 Копирай** - Копира кода в клипборда
- **💾 Изтегли** - Създава текстов файл с кода и информация за потребителя

### 2. Използване на код за възстановяване

**Местоположение:** `http://localhost/login`

Когато забравите паролата си:

1. Отидете на страницата за вход
2. Кликнете на **"Забравена парола? Използвайте код за възстановяване"**
3. Въведете следната информация:
   - **Потребителско име**
   - **Код за възстановяване** (във формат XXXX-XXXX)
   - **Нова парола** (минимум 6 символа)
   - **Потвърждение на нова парола**
4. Кликнете **"Възстанови паролата"**

След успешно възстановяване, можете да влезете с новата парола.

## Компоненти

### Frontend компоненти

#### 1. UserProfile.jsx
**Файл:** `frontend/src/components/UserProfile.jsx`

Главен компонент за страницата на профила с три раздела:
- **👤 Профил** - Показва информация за потребителя
- **🔒 Смяна на парола** - Позволява смяна на паролата
- **🔑 Код за възстановяване** - Генериране и управление на кодове

**Ключови функции:**

```javascript
// Генериране на код
const generateRecoveryCode = () => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  let code = '';
  for (let i = 0; i < 8; i++) {
    code += chars.charAt(Math.floor(Math.random() * chars.length));
    if (i === 3) code += '-';
  }
  return code;
};

// Изтегляне на код като файл
const downloadRecoveryCode = (code) => {
  const content = `RS-AC-BG Код за възстановяване на парола\n\n` +
                 `Потребител: ${user?.username}\n` +
                 `Име: ${user?.firstName} ${user?.lastName}\n` +
                 `Email: ${user?.email}\n\n` +
                 `КОД: ${code}\n\n` +
                 `Генериран на: ${new Date().toLocaleString('bg-BG')}\n\n`;

  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `recovery-code-${user?.username}-${Date.now()}.txt`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
};
```

#### 2. Login.jsx
**Файл:** `frontend/src/pages/Login.jsx`

Разширен компонент за вход с поддръжка на възстановяване на парола.

**Ключови функции:**

```javascript
const handleRecoverySubmit = async (e) => {
  e.preventDefault();

  // Валидация
  if (newPassword.length < 6) {
    setError('Новата парола трябва да е поне 6 символа');
    return;
  }

  if (newPassword !== confirmPassword) {
    setError('Паролите не съвпадат');
    return;
  }

  // Изпращане към backend
  const response = await fetch('/api/auth/recover-password', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      username,
      recoveryCode: recoveryCode.toUpperCase(),
      newPassword
    }),
  });
};
```

### Интеграция в Settings.jsx

**Файл:** `frontend/src/pages/Settings.jsx`

```javascript
import UserProfile from '../components/UserProfile';

// ...

<Routes>
  <Route path="/" element={<UserProfile />} />
  <Route path="/profile" element={<UserProfile />} />
  {/* ... други routes */}
</Routes>
```

## Backend имплементация (TODO)

### Необходими промени

#### 1. Добавяне на поле в базата данни

Добавете ново поле в таблицата `users`:

```sql
ALTER TABLE users
ADD COLUMN recovery_code_hash VARCHAR(255),
ADD COLUMN recovery_code_created_at TIMESTAMP;
```

#### 2. GraphQL мутация за генериране на код

**Файл:** `backend/src/graphql/mutations.rs`

```rust
async fn generate_recovery_code(
    ctx: &Context<'_>,
    input: String, // Хеширан код от клиента
) -> Result<bool> {
    let user_id = get_current_user_id(ctx)?;

    // Хеширай кода преди запис
    let hash = hash_password(&input)?;

    // Запази в базата данни
    let db = ctx.data::<DatabaseConnection>()?;
    User::update_recovery_code(db, user_id, hash).await?;

    Ok(true)
}
```

#### 3. REST endpoint за възстановяване

**Файл:** `backend/src/routes/auth.rs`

```rust
#[derive(Deserialize)]
struct RecoverPasswordRequest {
    username: String,
    recovery_code: String,
    new_password: String,
}

async fn recover_password(
    Json(payload): Json<RecoverPasswordRequest>,
    State(state): State<AppState>,
) -> Result<Json<RecoverPasswordResponse>, ApiError> {
    // Намери потребителя по username
    let user = User::find_by_username(&state.db, &payload.username)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

    // Провери кода за възстановяване
    if !verify_password(&payload.recovery_code, &user.recovery_code_hash)? {
        return Err(ApiError::InvalidRecoveryCode);
    }

    // Провери дали кодът не е изтекъл (например, след 90 дни)
    if let Some(created) = user.recovery_code_created_at {
        if created < Utc::now() - Duration::days(90) {
            return Err(ApiError::RecoveryCodeExpired);
        }
    }

    // Обнови паролата
    let new_hash = hash_password(&payload.new_password)?;
    User::update_password(&state.db, user.id, new_hash).await?;

    // Изтрий кода за възстановяване (еднократна употреба)
    User::clear_recovery_code(&state.db, user.id).await?;

    Ok(Json(RecoverPasswordResponse { success: true }))
}
```

#### 4. Migration за database schema

**Файл:** `migration/src/m20251101_000002_add_recovery_codes.rs`

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::RecoveryCodeHash)
                            .string()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(Users::RecoveryCodeCreatedAt)
                            .timestamp()
                            .null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::RecoveryCodeHash)
                    .drop_column(Users::RecoveryCodeCreatedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    RecoveryCodeHash,
    RecoveryCodeCreatedAt,
}
```

## Сигурност

### Препоръки за сигурност

1. **Хеширане на кодове**
   - Никога не съхранявайте кодове в plain text
   - Използвайте bcrypt или argon2 за хеширане
   - Добавете salt към всяко хеширане

2. **Срок на валидност**
   - Кодовете трябва да изтичат след определен период (например 90 дни)
   - При генериране на нов код, стария се инвалидира

3. **Еднократна употреба**
   - След успешно използване, кодът се изтрива от базата данни
   - Потребителят трябва да генерира нов код

4. **Rate limiting**
   - Ограничете броя опити за възстановяване (например 5 на час)
   - Блокирайте IP адреси при многократни неуспешни опити

5. **Логване**
   - Записвайте всички опити за възстановяване на парола
   - Известявайте потребителя при успешна смяна на паролата

### Примерна имплементация на хеширане

```rust
use bcrypt::{hash, verify, DEFAULT_COST};

// При генериране на код
pub fn hash_recovery_code(code: &str) -> Result<String, Error> {
    hash(code, DEFAULT_COST).map_err(|e| Error::HashingError(e))
}

// При проверка на код
pub fn verify_recovery_code(code: &str, hash: &str) -> Result<bool, Error> {
    verify(code, hash).map_err(|e| Error::VerificationError(e))
}
```

## Потребителски интерфейс

### Раздел "Код за възстановяване"

#### Преди генериране:
```
╔══════════════════════════════════════════════════╗
║                      🔑                          ║
║                                                  ║
║  Генерирайте код за възстановяване, който       ║
║  можете да използвате за достъп до профила си.  ║
║                                                  ║
║          [ 🔑 Генерирай код ]                   ║
╚══════════════════════════════════════════════════╝
```

#### След генериране:
```
╔══════════════════════════════════════════════════╗
║  ⚠️ Вашият код за възстановяване                 ║
║                                                  ║
║  ┌──────────────────────────────────────────┐   ║
║  │  ABCD-EFGH      [ 📋 Копирай ] [ 💾 Изтегли ] │
║  └──────────────────────────────────────────┘   ║
║                                                  ║
║  ⚠️ ВАЖНО:                                       ║
║  • Запишете този код на сигурно място           ║
║  • Никога не го споделяйте с никого              ║
║  • Този код ще се покаже само веднъж             ║
║  • Използвайте го за възстановяване при нужда    ║
║                                                  ║
║         [ 🔄 Генерирай нов код ]                ║
╚══════════════════════════════════════════════════╝
```

### Форма за възстановяване на паролата

```
╔══════════════════════════════════════════════════╗
║  Възстановяване на парола                        ║
║                                                  ║
║  Въведете вашето потребителско име, кода за     ║
║  възстановяване и нова парола.                   ║
║                                                  ║
║  Потребителско име:                              ║
║  [ ____________________ ]                        ║
║                                                  ║
║  Код за възстановяване:                          ║
║  [ XXXX-XXXX ]                                   ║
║                                                  ║
║  Нова парола (минимум 6 символа):                ║
║  [ ____________________ ]                        ║
║                                                  ║
║  Потвърди новата парола:                         ║
║  [ ____________________ ]                        ║
║                                                  ║
║  [ Назад ]    [ Възстанови паролата ]            ║
╚══════════════════════════════════════════════════╝
```

## Тестване

### Ръчно тестване

1. **Генериране на код:**
   - Влезте като потребител
   - Отидете на `/settings/profile`
   - Кликнете раздел "Код за възстановяване"
   - Генерирайте код и копирайте/изтеглете го

2. **Възстановяване на парола:**
   - Излезте от системата
   - Отидете на `/login`
   - Кликнете "Забравена парола?"
   - Въведете username, код и нова парола
   - Потвърдете смяната
   - Влезте с новата парола

3. **Валидация:**
   - Тествайте с невалиден код
   - Тествайте с несъвпадащи пароли
   - Тествайте с парола под 6 символа

## Често задавани въпроси (FAQ)

### Какво е код за възстановяване?
Код за възстановяване е алтернатива на email-базираното възстановяване на парола. Той позволява достъп до акаунта ви без нужда от email.

### Колко кода мога да генерирам?
Можете да генерирате нов код по всяко време, но само последният генериран код е валиден. Предишните кодове се инвалидират автоматично.

### Какво става ако загубя кода си?
Ако загубите кода си, просто генерирайте нов от страницата на профила си (след като влезете с паролата си).

### Изтича ли кодът?
Да, кодовете за възстановяване изтичат след 90 дни от генерирането им за допълнителна сигурност.

### Мога ли да използвам един код многократно?
Не, кодовете са за еднократна употреба. След успешно възстановяване на паролата, кодът се изтрива и трябва да генерирате нов.

### Колко сигурни са кодовете?
Кодовете са:
- Генерирани случайно с 8 символа (над 1 трилион комбинации)
- Хеширани в базата данни (не се съхраняват в plain text)
- Валидни само за определен период
- За еднократна употреба

## Следващи стъпки

- [ ] Имплементация на backend endpoint `/api/auth/recover-password`
- [ ] Добавяне на database migration за `recovery_code_hash` поле
- [ ] Добавяне на GraphQL мутация `generateRecoveryCode`
- [ ] Имплементация на rate limiting за опити за възстановяване
- [ ] Добавяне на email нотификация при смяна на парола
- [ ] Добавяне на audit log за действия с кодове
- [ ] Имплементация на срок на валидност (90 дни)
- [ ] Unit тестове за генериране и проверка на кодове
- [ ] Integration тестове за цялата система

## Версия

- **Версия:** 1.0.0
- **Дата:** 01.11.2025
- **Статус:** Frontend имплементиран, Backend TODO
