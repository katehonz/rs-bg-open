# Authentication & Authorization

## Преглед

RS AC BG използва JWT (JSON Web Token) базирана автентикация с role-based access control (RBAC) за защита на API endpoints.

## Ключови характеристики

- ✅ **JWT токени** - Безопасна автентикация с bcrypt хеширане на пароли
- ✅ **Role-Based Access Control (RBAC)** - Две нива на permissions:
  - **Global permissions** (user_groups) - системни права
  - **Company-specific roles** (user_companies) - права за конкретна фирма
- ✅ **Admin-only user management** - Само администратори могат да създават/управляват потребители
- ❌ **Без публична регистрация** - Нови потребители се създават само от администратори

## Endpoints

### Публични (без автентикация)

```
GET  /health              - Health check
POST /api/auth/login     - Login endpoint
GET  /graphiql            - GraphQL playground
```

### Защитени (изискват JWT token)

```
POST /graphql            - GraphQL API (всички queries и mutations освен login)
POST /api/controlisy/*   - Controlisy API endpoints
```

## Използване

### 1. Login

**Request:**
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "your_password"
  }'
```

**Response:**
```json
{
  "user": {
    "id": 1,
    "username": "admin",
    "email": "admin@localhost",
    "first_name": "System",
    "last_name": "Administrator",
    "group_id": 1,
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2024-01-02T00:00:00Z"
}
```

### 2. Използване на JWT токен

След login, използвайте получения токен в `Authorization` header:

```bash
curl -X POST http://localhost:8080/graphql \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { users { id username email } }"
  }'
```

### 3. GraphQL примери

**Query - List users:**
```graphql
query {
  users {
    id
    username
    email
    first_name
    last_name
    is_active
  }
}
```

**Mutation - Create user (admin only):**
```graphql
mutation {
  createUser(input: {
    username: "newuser"
    email: "newuser@example.com"
    password: "securepassword123"
    first_name: "First"
    last_name: "Last"
    group_id: 2
  }) {
    id
    username
    email
  }
}
```

**Mutation - Update user (admin only):**
```graphql
mutation {
  updateUser(id: 2, input: {
    is_active: false
  }) {
    id
    username
    is_active
  }
}
```

**Mutation - Change password (admin only):**
```graphql
mutation {
  changeUserPassword(userId: 2, newPassword: "newpassword123")
}
```

## Permission System

### Global Permissions (user_groups)

Всеки потребител принадлежи към една user group с глобални permissions:

| Permission | Описание |
|-----------|----------|
| `can_manage_users` | Може да създава, редактира и деактивира потребители |
| `can_create_companies` | Може да създава нови фирми |
| `can_edit_companies` | Може да редактира фирми |
| `can_delete_companies` | Може да изтрива фирми |
| `can_view_reports` | Може да вижда отчети |
| `can_post_entries` | Може да създава счетоводни записи |

### Company-Specific Roles (user_companies)

Всеки потребител може да има достъп до множество фирми с различни роли:

| Role | Описание |
|------|----------|
| `Admin` | Пълен административен достъп до фирмата |
| `User` | Може да редактира данни на фирмата |
| `Viewer` | Само четене на данни на фирмата |

### Permission Checks в GraphQL

В resolvers използвайте помощните функции от `graphql/context.rs`:

```rust
use crate::graphql::context::{
    require_can_manage_users,
    require_can_create_companies,
    require_company_access,
    require_company_admin,
};

// Check if user can manage users
async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> FieldResult<User> {
    require_can_manage_users(ctx).await?;
    // ... implementation
}

// Check if user has access to company
async fn get_company(&self, ctx: &Context<'_>, company_id: i32) -> FieldResult<Company> {
    require_company_access(ctx, company_id).await?;
    // ... implementation
}

// Check if user is admin of company
async fn delete_company(&self, ctx: &Context<'_>, company_id: i32) -> FieldResult<bool> {
    require_company_admin(ctx, company_id).await?;
    // ... implementation
}
```

## Initial Setup

### Създаване на първоначален администратор

**ВАЖНО:** Този скрипт трябва да се изпълни с sudo/root права!

```bash
cd rs-ac-bg/scripts
sudo ./init_admin.sh
```

Скриптът ще:
1. Създаде admin user group с всички permissions
2. Генерира силна случайна парола
3. Създаде admin потребител с генерираната парола
4. Запише credentials в `/root/.rs-ac-bg-admin` (readable само от root)

**Изход:**
```
========================================
  Admin User Created Successfully!
========================================

Admin credentials:
  Username: admin
  Password: <generated-password>

IMPORTANT: Save this password securely!
This password will NOT be shown again.
```

### Environment Variables

Добавете следните променливи в `.env`:

```bash
# JWT Configuration
JWT_SECRET=your_jwt_secret_min_32_chars_change_in_production
JWT_EXPIRATION_HOURS=24

# Database
DATABASE_URL=postgresql://app:password@localhost:5432/accounting
```

**ВАЖНО:** В продукция използвайте силен JWT_SECRET с поне 32 символа!

## Security Best Practices

1. **JWT Secret:**
   - Използвайте случаен, силен secret (минимум 32 символа)
   - Пазете secret-а в тайна (не го commit-вайте в git)
   - Сменяйте secret-а периодично

2. **Passwords:**
   - Минимална дължина: 6 символа
   - Използва bcrypt hashing с cost 12
   - Административните пароли се генерират автоматично

3. **Token Expiration:**
   - Default: 24 часа
   - Конфигурируемо чрез JWT_EXPIRATION_HOURS
   - Frontend трябва да handle изтекли токени

4. **HTTPS:**
   - В продукция използвайте САМО HTTPS
   - JWT токените са чувствителни и не трябва да се предават през HTTP

5. **CORS:**
   - Актуалната конфигурация позволява всички origins (за development)
   - В продукция ограничете CORS до конкретни domains

## Troubleshooting

### "Unauthorized" грешка

```json
{"error": "Missing or invalid Authorization header"}
```

**Решение:** Добавете валиден JWT token в Authorization header:
```
Authorization: Bearer <your-jwt-token>
```

### "Invalid token" грешка

```json
{"error": "Invalid token: ..."}
```

**Възможни причини:**
- Изтекъл token (минали са 24 часа)
- Невалиден format
- JWT_SECRET е променен след генериране на токена

**Решение:** Login отново за нов token.

### "Permission denied" грешка

```json
{"error": "Permission denied: cannot manage users"}
```

**Решение:** Потребителят няма необходимите permissions. Свържете се с администратор.

### Admin initialization script fails

```bash
Error: Failed to create or find admin group
```

**Възможни причини:**
- Database не е достъпна
- Migration не е изпълнена
- Грешни database credentials

**Решение:**
1. Проверете дали database-а работи
2. Изпълнете migrations: `cd migration && cargo run`
3. Проверете `configdb.json` за правилни credentials

## API Reference

### REST Endpoints

#### POST /api/auth/login

Автентикира потребител и връща JWT token.

**Request Body:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response:**
```json
{
  "user": {
    "id": 1,
    "username": "string",
    "email": "string",
    "first_name": "string",
    "last_name": "string",
    "group_id": 1,
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "token": "string",
  "expires_at": "2024-01-02T00:00:00Z"
}
```

**Status Codes:**
- `200 OK` - Успешен login
- `401 Unauthorized` - Грешен username/password или деактивиран потребител

### GraphQL Mutations

#### login

**DEPRECATED:** Използвайте REST endpoint `/api/auth/login` вместо това.

Все още е достъпен чрез GraphQL за обратна съвместимост:

```graphql
mutation {
  login(input: {
    username: "admin"
    password: "password"
  }) {
    user {
      id
      username
      email
    }
    token
    expires_at
  }
}
```

#### createUser (Admin Only)

Създава нов потребител.

```graphql
mutation {
  createUser(input: {
    username: "newuser"
    email: "user@example.com"
    password: "securepass123"
    first_name: "First"
    last_name: "Last"
    group_id: 2
  }) {
    id
    username
    email
  }
}
```

**Изисква:** `can_manage_users` permission

#### updateUser (Admin Only)

Актуализира потребител.

```graphql
mutation {
  updateUser(id: 2, input: {
    email: "newemail@example.com"
    is_active: false
  }) {
    id
    username
    email
    is_active
  }
}
```

**Изисква:** `can_manage_users` permission

#### changeUserPassword (Admin Only)

Променя паролата на потребител.

```graphql
mutation {
  changeUserPassword(userId: 2, newPassword: "newpassword123")
}
```

**Изисква:** `can_manage_users` permission

#### deactivateUser (Admin Only)

Деактивира потребител.

```graphql
mutation {
  deactivateUser(userId: 2)
}
```

**Изисква:** `can_manage_users` permission

## Development Tips

### Testing with curl

```bash
# Login
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your_password"}' \
  | jq -r '.token')

# Use token
curl -X POST http://localhost:8080/graphql \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query":"query { users { id username } }"}'
```

### Testing in GraphiQL

GraphiQL playground е достъпен на `http://localhost:8080/graphiql`, но **не е защитен** с authentication middleware. За да тествате защитени queries:

1. Login чрез REST endpoint и вземете токена
2. В GraphiQL, добавете HTTP header:
   ```
   {
     "Authorization": "Bearer <your-token>"
   }
   ```

### Debugging

Включете debug logging за authentication:

```bash
RUST_LOG=backend=debug,backend::auth=trace cargo run
```

## Migration Guide

Ако upgrade-вате от стара версия без authentication:

1. **Backup database:**
   ```bash
   pg_dump accounting > backup.sql
   ```

2. **Run migrations:**
   ```bash
   cd migration && cargo run
   ```

3. **Create admin user:**
   ```bash
   sudo ./scripts/init_admin.sh
   ```

4. **Update frontend:**
   - Добавете JWT token storage
   - Добавете Authorization header към всички requests
   - Добавете login form
   - Handle 401 errors и redirect към login

5. **Update .env:**
   - Добавете JWT_SECRET
   - Добавете JWT_EXPIRATION_HOURS

6. **Test:**
   - Login като admin
   - Създайте test user
   - Тествайте permissions

## Future Improvements

Планирани подобрения:

- [ ] Refresh tokens
- [ ] Password reset flow
- [ ] Email verification
- [ ] 2FA (Two-Factor Authentication)
- [ ] Rate limiting
- [ ] Audit logging
- [ ] Session management
- [ ] Password policies (complexity requirements)
