# Изисквани версии за компилация

Този проект изисква следните **точни** версии на инструментите, за да се компилира правилно.

## Версии

| Инструмент | Версия | Проверка команда |
|------------|--------|------------------|
| **Rust**   | `1.91.0` | `rustc --version` |
| **Cargo**  | `1.91.0` | `cargo --version` |
| **Node.js**| `22.20.0` | `node --version` |
| **npm**    | `11.6.2` | `npm --version` |
| **Docker** | `20.10+` | `docker --version` |
| **Docker Compose** | `2.0+` | `docker compose version` |

## Docker Images

Dockerfile-овете използват следните base images:

- **Backend**: `rust:1.91-alpine` (Dockerfile.accounting, ред 2)
- **Frontend**: `node:22.20-alpine` (frontend/Dockerfile, ред 2)

## Инсталация на правилните версии

### На локална машина (за development)

#### Rust 1.91.0
```bash
# Чрез rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install 1.91.0
rustup default 1.91.0
```

#### Node.js 22.20.0
```bash
# Чрез nvm (препоръчително)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 22.20.0
nvm use 22.20.0
nvm alias default 22.20.0
```

### На VPS (за production)

На VPS **НЕ** е нужно да инсталираш Rust и Node.js локално - Docker контейнерите ще използват правилните версии автоматично от базовите images.

Необходимо е само:
- Docker 20.10+
- Docker Compose 2.0+

```bash
# Проверка
docker --version
docker compose version
```

## Важно

**Не променяй версиите** в Dockerfile-овете без да обновиш локалната среда до същите версии, иначе може да има compilation несъвместимости!

Ако обновиш локалните версии, обнови и този файл и Dockerfile-овете.

---
Последна актуализация: 2025-11-04
