# GitLab Upload Instructions

## Стъпка 1: Създайте нов проект в GitLab

1. Влезте в GitLab: https://gitlab.com
2. Кликнете "New project"
3. Изберете "Create blank project"
4. Въведете име: `rs-ac-bg`
5. Изберете visibility level (Private/Internal/Public)
6. **НЕ** инициализирайте с README (вече имаме)
7. Кликнете "Create project"

## Стъпка 2: Подготовка на локалния проект

```bash
# Влезте в директорията на проекта
cd /home/dvg/z-nim-proloq/rs-ac-bg

# Инициализирайте git ако още не е
git init

# Добавете всички файлове
git add .

# Създайте първи commit
git commit -m "Initial commit: RS-AC-BG Bulgarian Accounting System"
```

## Стъпка 3: Свържете с GitLab и качете

Заменете `YOUR_USERNAME` с вашето GitLab потребителско име:

```bash
# Добавете GitLab remote
git remote add origin https://gitlab.com/YOUR_USERNAME/rs-ac-bg.git

# Или ако предпочитате SSH:
git remote add origin git@gitlab.com:YOUR_USERNAME/rs-ac-bg.git

# Качете кода
git push -u origin main
```

Ако branch-ът ви е `master` вместо `main`:
```bash
git push -u origin master
```

## Стъпка 4: Проверка

След качването проверете:
1. Всички файлове са качени
2. `.gitignore` работи правилно (няма `node_modules/`, `target/`, etc.)
3. README.md се показва правилно
4. CI/CD pipeline стартира автоматично

## Алтернативен метод - с нов remote

Ако вече имате съществуващ remote:

```bash
# Вижте текущите remotes
git remote -v

# Премахнете стария remote ако е нужно
git remote remove origin

# Добавете новия GitLab remote
git remote add gitlab https://gitlab.com/YOUR_USERNAME/rs-ac-bg.git

# Качете
git push -u gitlab main
```

## Важни файлове за GitLab

✅ **Подготвени файлове:**
- `.gitignore` - игнорира ненужните файлове
- `README.md` - документация на проекта
- `.gitlab-ci.yml` - CI/CD pipeline конфигурация
- `configdb.example.json` - примерна конфигурация

❌ **Файлове които НЕ се качват (в .gitignore):**
- `configdb.json` - съдържа пароли
- `node_modules/` - 500+ MB зависимости
- `target/` - компилирани файлове
- `*.log` - log файлове
- `*.db` - база данни файлове

## Допълнителни настройки в GitLab

### 1. CI/CD Variables
Отидете в Settings → CI/CD → Variables и добавете:
- `DATABASE_URL` - за тестове
- `DEPLOY_SERVER` - за deployment
- `DEPLOY_KEY` - SSH ключ за deployment

### 2. Protected branches
Settings → Repository → Protected branches:
- Защитете `main` branch
- Позволете само maintainers да правят push

### 3. Merge requests
Settings → General → Merge requests:
- Enable merge request approvals
- Require 1 approval за production

## Проблеми и решения

### Грешка: large files
Ако получите грешка за големи файлове:
```bash
# Проверете кои файлове са големи
find . -size +100M

# Уверете се че са в .gitignore
# Премахнете от git ако вече са добавени
git rm --cached path/to/large/file
```

### Грешка: permission denied
За SSH достъп:
```bash
# Генерирайте SSH ключ
ssh-keygen -t ed25519 -C "your.email@example.com"

# Копирайте публичния ключ
cat ~/.ssh/id_ed25519.pub

# Добавете в GitLab: Settings → SSH Keys
```

### Забравена парола в configdb.json
```bash
# Проверете дали не е в staged files
git status

# Ако е, премахнете я
git rm --cached configdb.json
git commit -m "Remove config with passwords"
```

## След качването

1. **Проверете CI/CD**: Pipelines → вижте дали build-ът минава
2. **Добавете колеги**: Project information → Members
3. **Настройте webhooks**: Settings → Webhooks (за notifications)
4. **Създайте milestones**: Issues → Milestones
5. **Документирайте API**: Wiki или `/docs` папка