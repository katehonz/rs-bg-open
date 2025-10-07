# RS-BG-Open - Bulgarian Accounting System (Open Source)

A modern web-based accounting system built with Rust (backend) and React (frontend) with full support for Bulgarian legislation.

## 📖 Introduction

RS-BG-Open is an open source accounting system created specifically for the needs of Bulgarian companies. The system provides a complete set of features needed for accounting, VAT reporting, depreciation, INTRASTAT declarations, and more.

## 🚀 Key Features

- ✅ **Full Bulgarian Chart of Accounts** - Automatically loaded when creating a company
- ✅ **Journal Entries** - Double-entry bookkeeping
- ✅ **VAT Processing** - Purchases, sales, declarations
- ✅ **Fixed Assets** - Depreciation and management
- ✅ **Currency Rates** - Automatic updates from the Bulgarian National Bank (BNB)
- ✅ **Controlisy Import** - XML format
- ✅ **SAF-T Export** - Standard audit file
- ✅ **INTRASTAT Module** - Intra-community trade declarations
- ✅ **Reports** - P&L, Balance Sheet, Trial Balance

## 📋 Requirements

- PostgreSQL 14+
- Rust 1.75+
- Node.js 22 LTS (recommended 22.20.0)
- 100MB free space (25MB for production)

## ⚡ Quick Start

### 1. Clone the project
```bash
git clone https://github.com/katehonz/rs-bg-open.git
cd rs-bg-open
```

### 2. Configuration
```bash
cp configdb.example.json configdb.json
# Edit configdb.json with your database settings
```

### 3. Database
```bash
createdb rs_ac_bg
```

### 4. Backend
```bash
cd backend
cargo build --release
./target/release/backend
```

### 5. Frontend
```bash
cd frontend
npm install
npm run dev
```

Open http://localhost:5173

## 📦 Production Deployment

### Backend
```bash
cargo build --release
# Result: backend/target/release/backend (20MB)
```

### Frontend
```bash
./build-frontend.sh
# Result: frontend/dist/ (5MB)
```

**Important:** For production you only need:
- `backend` binary file
- `configdb.json`
- `frontend/dist/` content

Do NOT upload `node_modules/` (500+ MB)!

## 🐳 Docker Support

The system supports Docker deployment:

```bash
docker-compose up -d
```

## 📚 Documentation

Detailed documentation will be added soon.

## 🏗️ Technologies

### Backend
- **Rust** - Actix-Web + async-graphql + SeaORM
- **PostgreSQL** - Database
- **GraphQL** - API

### Frontend
- **React** - UI library
- **Vite** - Build tool
- **TailwindCSS** - Styles
- **Apollo Client** - GraphQL client

## 🔧 Configuration

All settings are made in the `configdb.json` file. Use `configdb.example.json` as a template.

## License

This project is licensed under the LGPL-3.0 license - see the [LICENSE](LICENSE) file for more information.