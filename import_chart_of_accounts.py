#!/usr/bin/env python3
import csv
import psycopg2
from psycopg2.extras import execute_batch
import sys

# Database connection parameters
DB_CONFIG = {
    'host': 'localhost',
    'database': 'rs-ac-bg',
    'user': 'postgres',
    'password': 'pas+123'
}

def parse_account_type(type_str):
    """Convert Bulgarian account type to enum value"""
    type_map = {
        'ASSET': 'ASSET',
        'LIABILITY': 'LIABILITY', 
        'EQUITY': 'EQUITY',
        'REVENUE': 'REVENUE',
        'EXPENSE': 'EXPENSE'
    }
    return type_map.get(type_str, 'ASSET')

def parse_vat_direction(direction_str):
    """Convert VAT direction to enum value"""
    direction_map = {
        'NONE': 'NONE',
        'INPUT': 'INPUT',
        'OUTPUT': 'OUTPUT',
        'BOTH': 'BOTH'
    }
    return direction_map.get(direction_str, 'NONE')

def get_account_class(code):
    """Determine account class from code"""
    if not code:
        return 1
    first_digit = code[0]
    if first_digit.isdigit():
        return int(first_digit)
    return 1

def import_accounts(csv_file_path, company_id=1):
    """Import accounts from CSV file into database"""
    
    conn = None
    try:
        # Connect to database
        conn = psycopg2.connect(**DB_CONFIG)
        cur = conn.cursor()
        
        # First, clear existing accounts for this company (except the test ones)
        print("Clearing existing accounts...")
        cur.execute("""
            DELETE FROM accounts 
            WHERE company_id = %s 
            AND code NOT IN ('101', '201', '301')
        """, (company_id,))
        
        # Read CSV file
        with open(csv_file_path, 'r', encoding='utf-8') as file:
            reader = csv.DictReader(file)
            
            accounts_to_insert = []
            parent_map = {}  # Map synthetic account codes to IDs
            
            # First pass - collect all accounts
            all_accounts = list(reader)
            
            # Insert synthetic accounts first
            print("Inserting synthetic accounts...")
            synthetic_accounts = [acc for acc in all_accounts if acc['Тип сметка'] == 'СИНТЕТИЧНА']
            
            for account in synthetic_accounts:
                code = account['Код'].strip()
                name = account['Име на сметка'].strip()
                account_type = parse_account_type(account['Тип'].strip() if account['Тип'] else 'ASSET')
                account_class = get_account_class(code)
                is_vat_applicable = account['ДДС приложимост'].strip() == 'ДА'
                vat_direction = parse_vat_direction(account['ДДС посока'].strip() if account['ДДС посока'] else 'NONE')
                
                # Check if account already exists
                cur.execute("""
                    SELECT id FROM accounts 
                    WHERE code = %s AND company_id = %s
                """, (code, company_id))
                
                existing = cur.fetchone()
                
                if existing:
                    parent_map[code] = existing[0]
                    print(f"  Account {code} already exists, skipping...")
                else:
                    cur.execute("""
                        INSERT INTO accounts (
                            code, name, account_type, account_class, 
                            parent_id, level, is_vat_applicable, vat_direction,
                            is_active, is_analytical, company_id
                        ) VALUES (
                            %s, %s, %s, %s, 
                            NULL, 1, %s, %s,
                            true, false, %s
                        ) RETURNING id
                    """, (
                        code, name, account_type, account_class,
                        is_vat_applicable, vat_direction, company_id
                    ))
                    
                    account_id = cur.fetchone()[0]
                    parent_map[code] = account_id
                    print(f"  Inserted synthetic account: {code} - {name}")
            
            # Insert analytical accounts
            print("\nInserting analytical accounts...")
            analytical_accounts = [acc for acc in all_accounts if acc['Тип сметка'] == 'АНАЛИТИЧНА']
            
            for account in analytical_accounts:
                code = account['Код'].strip()
                name = account['Име на сметка'].strip()
                parent_code = account['Синтетична сметка'].strip() if account['Синтетична сметка'] else None
                account_type = parse_account_type(account['Тип'].strip() if account['Тип'] else 'ASSET')
                account_class = get_account_class(code)
                is_vat_applicable = account['ДДС приложимост'].strip() == 'ДА'
                vat_direction = parse_vat_direction(account['ДДС посока'].strip() if account['ДДС посока'] else 'NONE')
                
                # Find parent ID
                parent_id = parent_map.get(parent_code) if parent_code else None
                level = 2 if parent_id else 1
                
                # Check if account already exists
                cur.execute("""
                    SELECT id FROM accounts 
                    WHERE code = %s AND company_id = %s
                """, (code, company_id))
                
                if cur.fetchone():
                    print(f"  Account {code} already exists, skipping...")
                    continue
                
                cur.execute("""
                    INSERT INTO accounts (
                        code, name, account_type, account_class, 
                        parent_id, level, is_vat_applicable, vat_direction,
                        is_active, is_analytical, company_id
                    ) VALUES (
                        %s, %s, %s, %s, 
                        %s, %s, %s, %s,
                        true, true, %s
                    )
                """, (
                    code, name, account_type, account_class,
                    parent_id, level, is_vat_applicable, vat_direction, company_id
                ))
                
                print(f"  Inserted analytical account: {code} - {name}")
            
            # Commit transaction
            conn.commit()
            
            # Get total count
            cur.execute("SELECT COUNT(*) FROM accounts WHERE company_id = %s", (company_id,))
            total_count = cur.fetchone()[0]
            
            print(f"\nSuccessfully imported accounts! Total accounts in database: {total_count}")
            
        cur.close()
        conn.close()
        
    except psycopg2.Error as e:
        print(f"Database error: {e}")
        if conn:
            conn.rollback()
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    csv_file = "/home/dvg/z-nim-proloq/rs-ac-bg/file/ac_chart.csv"
    print(f"Importing chart of accounts from: {csv_file}")
    import_accounts(csv_file)
    print("Import completed!")