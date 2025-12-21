#!/bin/bash

################################################################################
# Admin Initialization Script
#
# This script creates the initial admin user for the accounting system.
# It must be run with sudo/root privileges.
#
# Usage: sudo ./init_admin.sh
################################################################################

set -e

# Colors for output
RED='\033[0:31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   exit 1
fi

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  RS AC BG Admin Initialization${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Load database configuration
if [ -f "../configdb.json" ]; then
    DB_HOST=$(jq -r '.host' ../configdb.json)
    DB_PORT=$(jq -r '.port' ../configdb.json)
    DB_NAME=$(jq -r '.database' ../configdb.json)
    DB_USER=$(jq -r '.username' ../configdb.json)
    DB_PASSWORD=$(jq -r '.password' ../configdb.json)
else
    echo -e "${YELLOW}configdb.json not found. Using defaults...${NC}"
    DB_HOST="${DB_HOST:-localhost}"
    DB_PORT="${DB_PORT:-5432}"
    DB_NAME="${DB_NAME:-accounting}"
    DB_USER="${DB_USER:-app}"
    read -sp "Enter database password: " DB_PASSWORD
    echo ""
fi

# Generate a strong random password for admin
ADMIN_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-24)

# Admin user details
ADMIN_USERNAME="admin"
ADMIN_EMAIL="admin@localhost"
ADMIN_FIRST_NAME="System"
ADMIN_LAST_NAME="Administrator"

echo -e "${YELLOW}Creating admin user group...${NC}"

# Create admin user group if it doesn't exist
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME <<EOF
INSERT INTO user_groups (name, description, can_create_companies, can_edit_companies, can_delete_companies, can_manage_users, can_view_reports, can_post_entries, created_at)
VALUES ('Administrators', 'System administrators with full permissions', true, true, true, true, true, true, NOW())
ON CONFLICT DO NOTHING
RETURNING id;
EOF

# Get admin group ID
ADMIN_GROUP_ID=$(PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c "SELECT id FROM user_groups WHERE name = 'Administrators' LIMIT 1;" | tr -d ' ')

if [ -z "$ADMIN_GROUP_ID" ]; then
    echo -e "${RED}Error: Failed to create or find admin group${NC}"
    exit 1
fi

echo -e "${GREEN}Admin group ID: $ADMIN_GROUP_ID${NC}"

# Hash the password using bcrypt (requires bcrypt command-line tool or we'll use a Rust binary)
# For now, we'll create a temporary Rust program to hash the password
echo -e "${YELLOW}Hashing admin password...${NC}"

# Create temporary Rust program for password hashing
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Cargo.toml" <<'CARGO_EOF'
[package]
name = "hash_password"
version = "0.1.0"
edition = "2021"

[dependencies]
bcrypt = "0.16"
CARGO_EOF

cat > "$TEMP_DIR/src/main.rs" <<'RUST_EOF'
use bcrypt::{hash, DEFAULT_COST};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <password>", args[0]);
        std::process::exit(1);
    }

    let password = &args[1];
    match hash(password, DEFAULT_COST) {
        Ok(hashed) => print!("{}", hashed),
        Err(e) => {
            eprintln!("Error hashing password: {}", e);
            std::process::exit(1);
        }
    }
}
RUST_EOF

mkdir -p "$TEMP_DIR/src"
cd "$TEMP_DIR"
cargo build --release --quiet 2>/dev/null || {
    echo -e "${RED}Error: Failed to build password hasher. Make sure Rust is installed.${NC}"
    rm -rf "$TEMP_DIR"
    exit 1
}

PASSWORD_HASH=$(./target/release/hash_password "$ADMIN_PASSWORD")
cd - > /dev/null
rm -rf "$TEMP_DIR"

echo -e "${YELLOW}Creating admin user...${NC}"

# Create admin user
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME <<EOF
INSERT INTO users (
    username,
    email,
    password_hash,
    first_name,
    last_name,
    group_id,
    is_active,
    document_period_start,
    document_period_end,
    document_period_active,
    accounting_period_start,
    accounting_period_end,
    accounting_period_active,
    vat_period_start,
    vat_period_end,
    vat_period_active,
    created_at,
    updated_at
)
VALUES (
    '$ADMIN_USERNAME',
    '$ADMIN_EMAIL',
    '$PASSWORD_HASH',
    '$ADMIN_FIRST_NAME',
    '$ADMIN_LAST_NAME',
    $ADMIN_GROUP_ID,
    true,
    '2024-01-01',
    '2099-12-31',
    true,
    '2024-01-01',
    '2099-12-31',
    true,
    '2024-01-01',
    '2099-12-31',
    true,
    NOW(),
    NOW()
)
ON CONFLICT (username) DO UPDATE
SET
    password_hash = EXCLUDED.password_hash,
    is_active = true,
    updated_at = NOW()
RETURNING id;
EOF

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Admin User Created Successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}Admin credentials:${NC}"
echo -e "  Username: ${GREEN}$ADMIN_USERNAME${NC}"
echo -e "  Password: ${GREEN}$ADMIN_PASSWORD${NC}"
echo ""
echo -e "${RED}IMPORTANT: Save this password securely!${NC}"
echo -e "${RED}This password will NOT be shown again.${NC}"
echo ""
echo -e "${YELLOW}Writing credentials to /root/.rs-ac-bg-admin (readable only by root)${NC}"
echo "RS AC BG Admin Credentials" > /root/.rs-ac-bg-admin
echo "Generated: $(date)" >> /root/.rs-ac-bg-admin
echo "" >> /root/.rs-ac-bg-admin
echo "Username: $ADMIN_USERNAME" >> /root/.rs-ac-bg-admin
echo "Password: $ADMIN_PASSWORD" >> /root/.rs-ac-bg-admin
chmod 600 /root/.rs-ac-bg-admin

echo ""
echo -e "${GREEN}Done! You can now login at:${NC}"
echo -e "  POST http://localhost:8080/api/auth/login"
echo ""
echo -e "${YELLOW}Example login request:${NC}"
cat <<'EXAMPLE'
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "<your-generated-password>"
  }'
EXAMPLE
echo ""
