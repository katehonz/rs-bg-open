# Recovery Code Система - Пълна документация

## 📋 Съдържание

- [Общ преглед](#общ-преглед)
- [Архитектура](#архитектура)
- [Потребителски флоу](#потребителски-флоу)
- [API Документация](#api-документация)
- [Database Schema](#database-schema)
- [Сигурност](#сигурност)
- [Тестване](#тестване)

## Общ преглед

Recovery Code системата предоставя сигурен механизъм за възстановяване на забравени пароли без необходимост от email верификация. Всеки потребител може да генерира персонален recovery code, който може да използва за reset на паролата си.

### Ключови характеристики

- ✅ **Сигурно съхранение** - Bcrypt хеширане (като паролите)
- ✅ **Времева валидност** - 90 дни от генериране
- ✅ **One-time use** - Автоматично изтриване след употреба
- ✅ **Валидация на формат** - XXXX-XXXX (8 символа + тире)
- ✅ **GraphQL + REST API** - Двойно покритие

## Архитектура

### Backend компоненти

```
backend/
├── src/
│   ├── entities/
│   │   └── user.rs                    # User модел с recovery code полета
│   ├── graphql/
│   │   └── user_resolvers.rs          # GraphQL мутация generateRecoveryCode
│   └── rest/
│       └── auth_api.rs                 # REST endpoint /api/auth/recover-password
└── migration/
    └── src/
        └── m20251101_000002_add_recovery_code_fields.rs
```

### Frontend компоненти

```
frontend/
└── src/
    ├── components/
    │   ├── UserProfile.jsx             # Генериране на recovery code
    │   └── Login.jsx                   # Forgot Password функционалност
    └── utils/
        └── graphql.js                  # GraphQL заявки
```

### Database Schema

```sql
-- Migration m20251101_000002
ALTER TABLE users
  ADD COLUMN recovery_code_hash VARCHAR NULL,
  ADD COLUMN recovery_code_created_at TIMESTAMP WITH TIME ZONE NULL;
```

**Забележки:**
- `recovery_code_hash` - Bcrypt хеш на recovery code (nullable)
- `recovery_code_created_at` - Timestamp на генериране (nullable, UTC)

## Потребителски флоу

### 1. Генериране на Recovery Code

**UI Локация:** Settings → Profile → Recovery Code Section

**Стъпки:**
1. Потребителят натиска "Generate Recovery Code"
2. Frontend генерира random код във формат XXXX-XXXX
3. Кодът се изпраща към GraphQL mutation `generateRecoveryCode`
4. Backend хешира кода с bcrypt и го записва в базата
5. Frontend показва кода на потребителя **САМО ВЕДНЪЖ**
6. Потребителят трябва да запази кода на сигурно място

**Код (Frontend):**
```javascript
const generateRecoveryCode = () => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  const part1 = Array.from({length: 4}, () =>
    chars[Math.floor(Math.random() * chars.length)]).join('');
  const part2 = Array.from({length: 4}, () =>
    chars[Math.floor(Math.random() * chars.length)]).join('');
  return `${part1}-${part2}`;
};
```

**GraphQL Заявка:**
```graphql
mutation GenerateRecoveryCode($recoveryCode: String!) {
  generateRecoveryCode(recoveryCode: $recoveryCode)
}
```

### 2. Възстановяване на парола

**UI Локация:** Login Page → "Forgot Password?" Link

**Стъпки:**
1. Потребителят кликва "Forgot Password?"
2. Появява се форма с полета:
   - Username
   - Recovery Code (XXXX-XXXX формат)
   - New Password
3. Frontend изпраща POST заявка към `/api/auth/recover-password`
4. Backend валидира:
   - Потребителят съществува
   - Има генериран recovery code
   - Кодът не е изтекъл (< 90 дни)
   - Кодът е валиден (bcrypt verify)
5. При успех:
   - Паролата се обновява
   - Recovery code се изтрива (`recovery_code_hash = NULL`)
   - Timestamp се изтрива (`recovery_code_created_at = NULL`)
6. Frontend показва съобщение за успех

**Валидации:**
- ❌ Потребителят не е намерен
- ❌ Няма генериран recovery code
- ❌ Кодът е изтекъл (> 90 дни)
- ❌ Невалиден формат (не е XXXX-XXXX)
- ❌ Грешен recovery code
- ❌ Новата парола е < 6 символа

## API Документация

### GraphQL API

#### Mutation: generateRecoveryCode

**Описание:** Генерира и съхранява нов recovery code за текущия потребител.

**Изисква:** Authentication (JWT token)

**Параметри:**
- `recoveryCode: String!` - Recovery code във формат XXXX-XXXX

**Връща:** `Boolean` (true при успех)

**Пример:**
```graphql
mutation {
  generateRecoveryCode(recoveryCode: "ABCD-1234")
}
```

**Response:**
```json
{
  "data": {
    "generateRecoveryCode": true
  }
}
```

**Грешки:**
- "Recovery code must be in format XXXX-XXXX" - Невалиден формат
- "Failed to hash recovery code" - Грешка при хеширане
- "User not found" - Потребителят не съществува

#### Query: user

**Описание:** Проверка дали потребител има генериран recovery code.

**Пример:**
```graphql
query {
  user(id: 1) {
    id
    username
    recovery_code_hash  # null ако няма код
  }
}
```

### REST API

#### POST /api/auth/recover-password

**Описание:** Възстановяване на парола чрез recovery code.

**Authentication:** None (публичен endpoint)

**Request Body:**
```json
{
  "username": "admin",
  "recovery_code": "ABCD-1234",
  "new_password": "newpass123"
}
```

**Response (Success - 200 OK):**
```json
{
  "success": true,
  "message": "Паролата е променена успешно"
}
```

**Response (Error - 400 Bad Request):**
```json
{
  "error": "Невалиден формат на код за възстановяване"
}
```

**Възможни грешки:**
- "Потребителят не е намерен"
- "Нямате генериран код за възстановяване"
- "Кодът за възстановяване е изтекъл. Моля, генерирайте нов код."
- "Невалиден формат на код за възстановяване"
- "Невалиден код за възстановяване"
- "Новата парола трябва да е поне 6 символа"

## Database Schema

### Таблица: users

**Нови полета (Migration m20251101_000002):**

| Колона | Тип | Nullable | Описание |
|--------|-----|----------|----------|
| `recovery_code_hash` | VARCHAR | YES | Bcrypt хеш на recovery code |
| `recovery_code_created_at` | TIMESTAMPTZ | YES | Timestamp на генериране (UTC) |

### Модел (Rust - SeaORM)

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    // ... други полета

    #[graphql(skip)]  // Не се излага в GraphQL
    pub recovery_code_hash: Option<String>,

    #[graphql(skip)]
    pub recovery_code_created_at: Option<DateTimeUtc>,
}
```

### Методи за валидация

```rust
impl Model {
    /// Проверка на recovery code спрямо хеша
    pub fn verify_recovery_code(&self, code: &str) -> Result<bool> {
        if let Some(ref hash) = self.recovery_code_hash {
            Ok(verify(code, hash)?)
        } else {
            Ok(false)
        }
    }

    /// Проверка дали кодът е изтекъл (> 90 дни)
    pub fn is_recovery_code_expired(&self) -> bool {
        if let Some(created_at) = self.recovery_code_created_at {
            let now = chrono::Utc::now();
            let age = now.signed_duration_since(created_at);
            age.num_days() > 90
        } else {
            true  // Няма код = изтекъл
        }
    }
}
```

## Сигурност

### 1. Хеширане с Bcrypt

Recovery кодовете се хешират със същия алгоритъм като паролите:

```rust
use bcrypt::{hash, verify, DEFAULT_COST};

// Генериране на хеш
let code_hash = hash("ABCD-1234", DEFAULT_COST)?;

// Проверка
let is_valid = verify("ABCD-1234", &code_hash)?;
```

**Параметри:**
- `DEFAULT_COST = 12` - Силно хеширане (балансирано performance/security)
- Salt се генерира автоматично от bcrypt

### 2. Защита срещу брутфорс атаки

**Възможни атаки:**
- Опит за генериране на всички възможни комбинации (36^8 = 2.8 триллиона)

**Защити:**
- Bcrypt slow hashing (>100ms за проверка)
- Rate limiting на REST endpoint (препоръчително)
- Логване на неуспешни опити

### 3. One-time use

След успешна употреба:
```rust
user.recovery_code_hash = Set(None);
user.recovery_code_created_at = Set(None);
user::Entity::update(user).exec(db).await?;
```

Това гарантира че:
- Кодът не може да се използва повторно
- Компрометиран код става безполезен след употреба

### 4. Времева валидност

Проверка при възстановяване:
```rust
if user.is_recovery_code_expired() {
    return Err("Кодът за възстановяване е изтекъл");
}
```

**Логика:**
- Ако `recovery_code_created_at` е NULL → изтекъл
- Ако `now - created_at > 90 дни` → изтекъл

## Тестване

### Unit Tests (Backend)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_code_format() {
        let code = "ABCD-1234";
        assert_eq!(code.len(), 9);
        assert!(code.contains('-'));
    }

    #[test]
    fn test_bcrypt_hash_verify() {
        let code = "TEST-CODE";
        let hash = Model::hash_password(code).unwrap();

        let user = Model {
            recovery_code_hash: Some(hash),
            // ... други полета
        };

        assert!(user.verify_recovery_code(code).unwrap());
        assert!(!user.verify_recovery_code("WRONG-CODE").unwrap());
    }

    #[test]
    fn test_expiration() {
        let old_date = Utc::now() - Duration::days(91);
        let user = Model {
            recovery_code_created_at: Some(old_date),
            // ...
        };

        assert!(user.is_recovery_code_expired());
    }
}
```

### Integration Tests

**Сценарий 1: Пълен флоу**
```bash
# 1. Генериране на код
curl -X POST http://localhost/graphql \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"query":"mutation { generateRecoveryCode(recoveryCode: \"TEST-1234\") }"}'

# 2. Възстановяване на парола
curl -X POST http://localhost/api/auth/recover-password \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "recovery_code": "TEST-1234",
    "new_password": "newpass123"
  }'

# 3. Login с нова парола
curl -X POST http://localhost/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "newpass123"
  }'
```

**Сценарий 2: Валидации**
```bash
# Без recovery code
curl -X POST http://localhost/api/auth/recover-password \
  -d '{"username":"admin","recovery_code":"XXXX-XXXX","new_password":"pass"}'
# Expected: "Нямате генериран код за възстановяване"

# Грешен код
curl -X POST http://localhost/api/auth/recover-password \
  -d '{"username":"admin","recovery_code":"WRONG-CODE","new_password":"pass"}'
# Expected: "Невалиден код за възстановяване"

# Кратка парола
curl -X POST http://localhost/api/auth/recover-password \
  -d '{"username":"admin","recovery_code":"TEST-1234","new_password":"123"}'
# Expected: "Новата парола трябва да е поне 6 символа"
```

**Сценарий 3: One-time use**
```bash
# Първи опит (успешен)
curl -X POST http://localhost/api/auth/recover-password \
  -d '{"username":"admin","recovery_code":"TEST-1234","new_password":"pass123"}'
# Expected: success

# Втори опит (неуспешен)
curl -X POST http://localhost/api/auth/recover-password \
  -d '{"username":"admin","recovery_code":"TEST-1234","new_password":"pass456"}'
# Expected: "Нямате генериран код за възстановяване"
```

## Често срещани проблеми

### 1. GraphQL грешка: "Data `backend::entities::user::Model` does not exist"

**Причина:** Мутацията опитва да вземе `user::Model` вместо `AuthenticatedUser` от контекста.

**Решение:**
```rust
// ГРЕШНО ❌
let current_user = ctx.data::<user::Model>()?;

// ПРАВИЛНО ✅
let auth_user = ctx.data::<AuthenticatedUser>()?;
let current_user = &auth_user.user;
```

### 2. Timestamp type mismatch

**Причина:** Миграцията използва `timestamp()` вместо `timestamp_with_time_zone()`.

**Решение:**
```rust
.add_column(
    ColumnDef::new(Users::RecoveryCodeCreatedAt)
        .timestamp_with_time_zone()  // ✅ Правилно
        .null()
)
```

### 3. Recovery code не работи след restart

**Причина:** PostgreSQL prepared statement cache.

**Решение:** Restart на backend след промяна в schema:
```bash
docker compose restart accounting-service
```

## Допълнителни препоръки

### Rate Limiting

Препоръчително е да се добави rate limiting на `/api/auth/recover-password`:

```rust
// Пример с actix-web-lab
use actix_web_lab::middleware::RateLimiter;

.service(
    web::resource("/api/auth/recover-password")
        .wrap(RateLimiter::new(5, Duration::from_secs(300)))  // 5 опита на 5 мин
        .route(web::post().to(recover_password))
)
```

### Логване

Добавете logging за security audit:

```rust
log::warn!(
    "Failed recovery attempt for user: {}, IP: {}",
    input.username,
    remote_addr
);
```

### Email нотификации (опционално)

При успешна промяна на парола, изпратете email:

```rust
if let Some(email) = &user.email {
    send_password_changed_notification(email).await?;
}
```

## Ресурси

- **Migration файл:** `migration/src/m20251101_000002_add_recovery_code_fields.rs`
- **Backend entity:** `backend/src/entities/user.rs`
- **GraphQL resolver:** `backend/src/graphql/user_resolvers.rs`
- **REST API:** `backend/src/rest/auth_api.rs`
- **Frontend компонент:** `frontend/src/components/UserProfile.jsx`
- **Login компонент:** `frontend/src/components/Login.jsx`

## Версия

- **Дата:** 2025-11-01
- **Автор:** Development Team
- **Status:** Production Ready ✅
