#!/bin/bash

# Критичен backup скрипт за миграция
# ВНИМАНИЕ: Стартирай преди всяка миграционна стъпка!

set -e

# Конфигурация
DB_HOST=${DB_HOST:-localhost}
DB_PORT=${DB_PORT:-5432}
DB_NAME=${DB_NAME:-accounting}
DB_USER=${DB_USER:-postgres}
BACKUP_DIR="./migration_backups/$(date +%Y%m%d_%H%M%S)"

echo "🔒 Започване на критичен backup за миграция..."
mkdir -p "$BACKUP_DIR"

# 1. ПЪЛЕН DATABASE DUMP (най-важно!)
echo "📦 Създаване на пълен database backup..."
pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    -f "$BACKUP_DIR/full_database_backup.sql" \
    --verbose --no-password

# 2. СМЕТКОПЛАН - критични таблици отделно
echo "📊 Backup на сметкоплан и счетоводни данни..."
pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    --table=accounts \
    --table=chart_of_accounts \
    --table=journal_entries \
    --table=entry_lines \
    -f "$BACKUP_DIR/chart_of_accounts_backup.sql" \
    --verbose --no-password

# 3. КОМПАНИИ И ПОТРЕБИТЕЛИ
echo "👥 Backup на компании и потребители..."
pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    --table=companies \
    --table=users \
    --table=user_groups \
    --table=company_users \
    -f "$BACKUP_DIR/users_companies_backup.sql" \
    --verbose --no-password

# 4. КОНТРАГЕНТИ И ДДС
echo "🏢 Backup на контрагенти и ДДС данни..."
pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    --table=counterparts \
    --table=vat_returns \
    --table=vat_rates \
    -f "$BACKUP_DIR/counterparts_vat_backup.sql" \
    --verbose --no-password

# 5. ИМПОРТИ И ИСТОРИЧЕСКИ ДАННИ
echo "📥 Backup на импорт данни..."
pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    --table=controlisy_imports \
    --table=bank_imports \
    -f "$BACKUP_DIR/imports_backup.sql" \
    --verbose --no-password

# 6. EXPORT НА СМЕТКОПЛАН В CSV (за анализ)
echo "📋 Export на сметкоплан в CSV формат..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    -c "COPY (SELECT code, name, account_type, account_class, level, parent_code, is_active, is_vat_applicable FROM accounts ORDER BY code) TO STDOUT WITH CSV HEADER" \
    > "$BACKUP_DIR/chart_of_accounts.csv"

# 7. СХЕМА НА БАЗАТА (без данни)
echo "🏗️ Export на database схема..."
pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    --schema-only \
    -f "$BACKUP_DIR/database_schema.sql" \
    --verbose --no-password

# 8. СПИСЪК НА ВСИЧКИ ТАБЛИЦИ
echo "📝 Създаване на списък с таблици..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    -c "SELECT schemaname, tablename, tableowner FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename" \
    > "$BACKUP_DIR/tables_list.txt"

# 9. СТАТИСТИКИ НА ДАННИТЕ
echo "📈 Събиране на статистики..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    -c "
    SELECT 
        'accounts' as table_name, COUNT(*) as record_count FROM accounts
    UNION ALL
    SELECT 'journal_entries', COUNT(*) FROM journal_entries
    UNION ALL  
    SELECT 'entry_lines', COUNT(*) FROM entry_lines
    UNION ALL
    SELECT 'companies', COUNT(*) FROM companies
    UNION ALL
    SELECT 'users', COUNT(*) FROM users
    UNION ALL
    SELECT 'counterparts', COUNT(*) FROM counterparts
    ORDER BY table_name;
    " > "$BACKUP_DIR/data_statistics.txt"

# 10. КОМПРЕСИРАНЕ НА BACKUP-А
echo "🗜️ Компресиране на backup файловете..."
cd "$BACKUP_DIR"
tar -czf "../backup_$(basename $BACKUP_DIR).tar.gz" .
cd - > /dev/null

# 11. ПРОВЕРКА НА BACKUP-А
echo "✅ Проверка на backup файловете..."
BACKUP_SIZE=$(du -sh "$BACKUP_DIR" | cut -f1)
FILE_COUNT=$(find "$BACKUP_DIR" -type f | wc -l)

echo ""
echo "🎉 BACKUP ЗАВЪРШЕН УСПЕШНО!"
echo "📁 Локация: $BACKUP_DIR"
echo "💾 Размер: $BACKUP_SIZE"
echo "📄 Файлове: $FILE_COUNT"
echo ""
echo "🔍 Съдържание:"
ls -la "$BACKUP_DIR"
echo ""

# 12. СЪЗДАВАНЕ НА RESTORE СКРИПТ
cat > "$BACKUP_DIR/restore_instructions.txt" << EOF
=== ИНСТРУКЦИИ ЗА ВЪЗСТАНОВЯВАНЕ ===

За пълно възстановяване:
1. Създай празна база данни
2. Изпълни: psql -h HOST -p PORT -U USER -d DATABASE -f full_database_backup.sql

За възстановяване само на сметкоплан:
1. psql -h HOST -p PORT -U USER -d DATABASE -f chart_of_accounts_backup.sql

За възстановяване на потребители:
1. psql -h HOST -p PORT -U USER -d DATABASE -f users_companies_backup.sql

ВАЖНО: 
- Тествай restore на тестова база преди production!
- Провери data_statistics.txt за броя записи
- Сравни със статистиките след restore

Дата на backup: $(date)
EOF

echo "📋 Създадени са инструкции за възстановяване в: $BACKUP_DIR/restore_instructions.txt"
echo ""
echo "⚠️  ВАЖНО: Запази този backup на сигурно място преди миграцията!"
echo "⚠️  Тествай restore процедурата на тестова среда!"