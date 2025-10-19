use async_graphql::{Context, FieldResult, Object};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

use crate::data::chart_of_accounts::load_chart_of_accounts;
use crate::entities::account::{AccountWithBalance, CreateAccountInput, UpdateAccountInput};
use crate::entities::journal_entry::{
    CreateJournalEntryInput, JournalEntryFilter, JournalEntryWithLines, UpdateJournalEntryInput,
};
use crate::entities::{account, company, counterpart, entry_line, journal_entry};

#[derive(Default)]
pub struct AccountingQuery;

#[Object]
impl AccountingQuery {
    /// Get all companies
    async fn companies(&self, ctx: &Context<'_>) -> FieldResult<Vec<company::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let companies = company::Entity::find()
            .filter(company::Column::IsActive.eq(true))
            .order_by_asc(company::Column::Name)
            .all(db)
            .await?;
        Ok(companies)
    }

    /// Get company by ID
    async fn company(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Option<company::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let company = company::Entity::find_by_id(id).one(db).await?;
        Ok(company)
    }

    /// Get accounts with optional filtering
    async fn accounts(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        parent_id: Option<i32>,
        is_analytical: Option<bool>,
    ) -> FieldResult<Vec<account::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = account::Entity::find()
            .filter(account::Column::CompanyId.eq(company_id))
            .filter(account::Column::IsActive.eq(true));

        if let Some(parent) = parent_id {
            query = query.filter(account::Column::ParentId.eq(parent));
        }

        if let Some(analytical) = is_analytical {
            query = query.filter(account::Column::IsAnalytical.eq(analytical));
        }

        let accounts = query.order_by_asc(account::Column::Code).all(db).await?;

        Ok(accounts)
    }

    /// Get account hierarchy (chart of accounts)
    async fn account_hierarchy(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        include_inactive: Option<bool>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<account::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = account::Entity::find().filter(account::Column::CompanyId.eq(company_id));

        // Only filter by active status if include_inactive is not true
        if !include_inactive.unwrap_or(false) {
            query = query.filter(account::Column::IsActive.eq(true));
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let accounts = query.order_by_asc(account::Column::Code).all(db).await?;

        Ok(accounts)
    }

    /// Get counterparts
    async fn counterparts(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<counterpart::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = counterpart::Entity::find()
            .filter(counterpart::Column::CompanyId.eq(company_id))
            .filter(counterpart::Column::IsActive.eq(true));

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let counterparts = query.order_by_asc(counterpart::Column::Name).all(db).await?;

        Ok(counterparts)
    }

    /// Get journal entries with filtering
    async fn journal_entries(
        &self,
        ctx: &Context<'_>,
        filter: Option<JournalEntryFilter>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<journal_entry::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = journal_entry::Entity::find();

        if let Some(f) = filter {
            let mut condition = Condition::all();

            if let Some(company_id) = f.company_id {
                condition = condition.add(journal_entry::Column::CompanyId.eq(company_id));
            }

            if let Some(from_date) = f.from_date {
                condition = condition.add(journal_entry::Column::DocumentDate.gte(from_date));
            }

            if let Some(to_date) = f.to_date {
                condition = condition.add(journal_entry::Column::DocumentDate.lte(to_date));
            }

            if let Some(account_id) = f.account_id {
                // Filter by entries that have lines for this account
                let entry_ids = entry_line::Entity::find()
                    .filter(entry_line::Column::AccountId.eq(account_id))
                    .all(db)
                    .await?
                    .into_iter()
                    .map(|line| line.journal_entry_id)
                    .collect::<Vec<_>>();

                if !entry_ids.is_empty() {
                    condition = condition.add(journal_entry::Column::Id.is_in(entry_ids));
                }
            }

            if let Some(is_posted) = f.is_posted {
                condition = condition.add(journal_entry::Column::IsPosted.eq(is_posted));
            }

            if let Some(created_by) = f.created_by {
                condition = condition.add(journal_entry::Column::CreatedBy.eq(created_by));
            }

            if let Some(document_number) = f.document_number {
                condition =
                    condition.add(journal_entry::Column::DocumentNumber.contains(&document_number));
            }

            query = query.filter(condition);
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let entries = query
            .order_by_desc(journal_entry::Column::DocumentDate)
            .order_by_desc(journal_entry::Column::Id)
            .all(db)
            .await?;

        Ok(entries)
    }

    /// Get journal entry with lines
    async fn journal_entry_with_lines(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<Option<JournalEntryWithLines>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        if let Some(entry) = journal_entry::Entity::find_by_id(id).one(db).await? {
            let lines = entry_line::Entity::find()
                .filter(entry_line::Column::JournalEntryId.eq(id))
                .order_by_asc(entry_line::Column::LineOrder)
                .all(db)
                .await?;

            Ok(Some(JournalEntryWithLines {
                journal_entry: entry,
                lines,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get account balances
    async fn account_balances(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        as_of_date: Option<NaiveDate>,
    ) -> FieldResult<Vec<AccountWithBalance>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let cutoff_date = as_of_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        let accounts = account::Entity::find()
            .filter(account::Column::CompanyId.eq(company_id))
            .filter(account::Column::IsActive.eq(true))
            .filter(account::Column::IsAnalytical.eq(true))
            .all(db)
            .await?;

        let mut results = Vec::new();

        for account in accounts {
            // Calculate balance from entry lines up to the cutoff date
            let lines = entry_line::Entity::find()
                .filter(entry_line::Column::AccountId.eq(account.id))
                .inner_join(journal_entry::Entity)
                .filter(journal_entry::Column::DocumentDate.lte(cutoff_date))
                .filter(journal_entry::Column::IsPosted.eq(true))
                .all(db)
                .await?;

            let mut debit_balance = Decimal::ZERO;
            let mut credit_balance = Decimal::ZERO;

            for line in lines {
                debit_balance += line.debit_amount;
                credit_balance += line.credit_amount;
            }

            let net_balance = debit_balance - credit_balance;

            results.push(AccountWithBalance {
                account,
                debit_balance,
                credit_balance,
                net_balance,
            });
        }

        Ok(results)
    }

    /// Get trial balance
    async fn trial_balance(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        as_of_date: NaiveDate,
    ) -> FieldResult<Vec<AccountWithBalance>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let _db = db.as_ref();

        // This would be a complex query in a real system
        // For now, return account balances (simplified version)
        self.account_balances(ctx, company_id, Some(as_of_date))
            .await
    }

    /// Count entry lines (debit/credit rows) for a period
    async fn count_entry_lines(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
        is_posted: Option<bool>,
    ) -> FieldResult<i64> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = entry_line::Entity::find()
            .inner_join(journal_entry::Entity)
            .filter(journal_entry::Column::CompanyId.eq(company_id));

        if let Some(from) = from_date {
            query = query.filter(journal_entry::Column::AccountingDate.gte(from));
        }

        if let Some(to) = to_date {
            query = query.filter(journal_entry::Column::AccountingDate.lte(to));
        }

        if let Some(posted) = is_posted {
            query = query.filter(journal_entry::Column::IsPosted.eq(posted));
        }

        let count = query.count(db).await?;

        Ok(count as i64)
    }
}

#[derive(Default)]
pub struct AccountingMutation;

#[Object]
impl AccountingMutation {
    /// Create company
    async fn create_company(
        &self,
        ctx: &Context<'_>,
        input: crate::entities::company::CreateCompanyInput,
    ) -> FieldResult<company::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let company_model = company::ActiveModel::from(input);
        let company = company::Entity::insert(company_model)
            .exec_with_returning(db)
            .await?;

        // Load the default chart of accounts for the new company
        if let Err(e) = load_chart_of_accounts(db, company.id).await {
            tracing::error!(
                "Failed to load chart of accounts for company {}: {}",
                company.id,
                e
            );
            // We don't fail the company creation if chart loading fails
            // but log the error for debugging
        } else {
            tracing::info!(
                "Successfully loaded chart of accounts for company {}",
                company.id
            );
        }

        Ok(company)
    }

    /// Update company
    async fn update_company(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: crate::entities::company::UpdateCompanyInput,
    ) -> FieldResult<company::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut company: company::ActiveModel = company::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Company not found"))?
            .into();

        if let Some(name) = input.name {
            company.name = Set(name);
        }
        if let Some(eik) = input.eik {
            company.eik = Set(eik);
        }
        if let Some(vat_number) = input.vat_number {
            company.vat_number = Set(Some(vat_number));
        }
        if let Some(address) = input.address {
            company.address = Set(Some(address));
        }
        if let Some(city) = input.city {
            company.city = Set(Some(city));
        }
        if let Some(country) = input.country {
            company.country = Set(Some(country));
        }
        if let Some(phone) = input.phone {
            company.phone = Set(Some(phone));
        }
        if let Some(email) = input.email {
            company.email = Set(Some(email));
        }
        if let Some(contact_person) = input.contact_person {
            company.contact_person = Set(Some(contact_person));
        }
        if let Some(manager_name) = input.manager_name {
            company.manager_name = Set(Some(manager_name));
        }
        if let Some(authorized_person) = input.authorized_person {
            company.authorized_person = Set(Some(authorized_person));
        }
        if let Some(manager_egn) = input.manager_egn {
            company.manager_egn = Set(Some(manager_egn));
        }
        if let Some(authorized_person_egn) = input.authorized_person_egn {
            company.authorized_person_egn = Set(Some(authorized_person_egn));
        }
        if let Some(is_active) = input.is_active {
            company.is_active = Set(is_active);
        }

        company.updated_at = Set(chrono::Utc::now());

        let updated_company = company.update(db).await?;
        Ok(updated_company)
    }

    /// Create account
    async fn create_account(
        &self,
        ctx: &Context<'_>,
        input: CreateAccountInput,
    ) -> FieldResult<account::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Check if account code already exists in company
        let existing = account::Entity::find()
            .filter(account::Column::CompanyId.eq(input.company_id))
            .filter(account::Column::Code.eq(&input.code))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err("Account code already exists in this company".into());
        }

        let account_model = account::ActiveModel::from(input);
        let account = account::Entity::insert(account_model)
            .exec_with_returning(db)
            .await?;

        Ok(account)
    }

    /// Create journal entry with lines
    async fn create_journal_entry(
        &self,
        ctx: &Context<'_>,
        input: CreateJournalEntryInput,
    ) -> FieldResult<JournalEntryWithLines> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Validate that debits equal credits
        let mut total_debits = Decimal::ZERO;
        let mut total_credits = Decimal::ZERO;

        for line_input in &input.lines {
            if let Some(debit) = line_input.debit_amount {
                total_debits += debit;
            }
            if let Some(credit) = line_input.credit_amount {
                total_credits += credit;
            }
        }

        if total_debits != total_credits {
            return Err(format!(
                "Debits ({}) must equal credits ({})",
                total_debits, total_credits
            )
            .into());
        }

        // Create journal entry
        let mut entry_model = journal_entry::ActiveModel::from(input.clone());
        entry_model.created_by = Set(1); // TODO: Get from auth context
        entry_model.total_amount = Set(total_debits);

        let entry = journal_entry::Entity::insert(entry_model)
            .exec_with_returning(db)
            .await?;

        // Create entry lines
        let mut lines = Vec::new();
        for (index, line_input) in input.lines.into_iter().enumerate() {
            let line_input_proper = entry_line::CreateEntryLineInput {
                account_id: line_input.account_id,
                debit_amount: line_input.debit_amount,
                credit_amount: line_input.credit_amount,
                description: line_input.description,
                counterpart_id: line_input.counterpart_id,
                vat_rate_id: None, // Not available in journal_entry::CreateEntryLineInput
                vat_amount: line_input.vat_amount,
                currency_code: line_input.currency_code,
                currency_amount: line_input.currency_amount,
                exchange_rate: line_input.exchange_rate,
                quantity: line_input.quantity,
                unit_of_measure_code: line_input.unit_of_measure_code,
                line_order: Some((index + 1) as i32),
            };
            let mut line_model = entry_line::ActiveModel::from(line_input_proper);
            line_model.journal_entry_id = Set(entry.id);
            line_model.line_order = Set((index + 1) as i32);

            let line = entry_line::Entity::insert(line_model)
                .exec_with_returning(db)
                .await?;
            lines.push(line);
        }

        Ok(JournalEntryWithLines {
            journal_entry: entry,
            lines,
        })
    }

    /// Post journal entry (make it permanent)
    async fn post_journal_entry(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<journal_entry::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut entry: journal_entry::ActiveModel =
            if let Some(entry) = journal_entry::Entity::find_by_id(id).one(db).await? {
                if entry.is_posted {
                    return Err("Journal entry is already posted".into());
                }
                entry.into()
            } else {
                return Err("Journal entry not found".into());
            };

        entry.is_posted = Set(true);
        entry.posted_by = Set(Some(1)); // TODO: Get from auth context
        entry.posted_at = Set(Some(chrono::Utc::now()));

        let updated_entry = journal_entry::Entity::update(entry).exec(db).await?;
        Ok(updated_entry)
    }

    /// Update journal entry
    async fn update_journal_entry(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateJournalEntryInput,
    ) -> FieldResult<journal_entry::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Find the existing entry
        let existing_entry = journal_entry::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or("Journal entry not found")?;

        if existing_entry.is_posted {
            return Err("Cannot update posted journal entry".into());
        }

        // Update journal entry fields
        let mut entry_model: journal_entry::ActiveModel = existing_entry.into();
        if let Some(document_date) = input.document_date {
            entry_model.document_date = Set(document_date);
        }
        if let Some(vat_date) = input.vat_date {
            entry_model.vat_date = Set(Some(vat_date));
        }
        if let Some(accounting_date) = input.accounting_date {
            entry_model.accounting_date = Set(accounting_date);
        }
        if let Some(document_number) = input.document_number {
            entry_model.document_number = Set(Some(document_number));
        }
        if let Some(description) = input.description {
            entry_model.description = Set(description);
        }

        // Handle lines if provided
        if let Some(lines_input) = input.lines {
            // Delete existing lines
            entry_line::Entity::delete_many()
                .filter(entry_line::Column::JournalEntryId.eq(id))
                .exec(db)
                .await?;

            // Validate that debits equal credits
            let mut total_debits = Decimal::ZERO;
            let mut total_credits = Decimal::ZERO;

            for line_input in &lines_input {
                if let Some(debit) = line_input.debit_amount {
                    total_debits += debit;
                }
                if let Some(credit) = line_input.credit_amount {
                    total_credits += credit;
                }
            }

            if total_debits != total_credits {
                return Err(format!(
                    "Debits ({}) must equal credits ({})",
                    total_debits, total_credits
                )
                .into());
            }

            // Create new lines
            for (index, line_input) in lines_input.into_iter().enumerate() {
                let line_input_proper = entry_line::CreateEntryLineInput {
                    account_id: line_input.account_id,
                    debit_amount: line_input.debit_amount,
                    credit_amount: line_input.credit_amount,
                    description: line_input.description,
                    counterpart_id: line_input.counterpart_id,
                    vat_rate_id: None,
                    vat_amount: line_input.vat_amount,
                    currency_code: line_input.currency_code,
                    currency_amount: line_input.currency_amount,
                    exchange_rate: line_input.exchange_rate,
                    quantity: line_input.quantity,
                    unit_of_measure_code: line_input.unit_of_measure_code,
                    line_order: Some((index + 1) as i32),
                };
                let mut line_model = entry_line::ActiveModel::from(line_input_proper);
                line_model.journal_entry_id = Set(id);
                entry_line::Entity::insert(line_model).exec(db).await?;
            }

            entry_model.total_amount = Set(total_debits);
            entry_model.total_vat_amount = Set(Decimal::ZERO); // Calculate from lines if needed
        }

        let updated_entry = journal_entry::Entity::update(entry_model).exec(db).await?;
        Ok(updated_entry)
    }

    /// Unpost journal entry (return to draft status)
    async fn unpost_journal_entry(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<journal_entry::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut entry: journal_entry::ActiveModel =
            if let Some(entry) = journal_entry::Entity::find_by_id(id).one(db).await? {
                if !entry.is_posted {
                    return Err("Journal entry is not posted".into());
                }
                entry.into()
            } else {
                return Err("Journal entry not found".into());
            };

        entry.is_posted = Set(false);
        entry.posted_at = Set(None);

        let updated_entry = journal_entry::Entity::update(entry).exec(db).await?;
        Ok(updated_entry)
    }

    /// Delete journal entry
    async fn delete_journal_entry(&self, ctx: &Context<'_>, id: i32) -> FieldResult<bool> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Find the journal entry
        let entry = journal_entry::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or("Journal entry not found")?;

        // Check if it's posted
        if entry.is_posted {
            return Err("Cannot delete posted journal entry. Unpost it first.".into());
        }

        // Delete associated entry lines first (foreign key constraint)
        entry_line::Entity::delete_many()
            .filter(entry_line::Column::JournalEntryId.eq(id))
            .exec(db)
            .await?;

        // Delete the journal entry
        journal_entry::Entity::delete_by_id(id).exec(db).await?;

        Ok(true)
    }

    /// Unpost multiple journal entries
    async fn unpost_journal_entries(&self, ctx: &Context<'_>, ids: Vec<i32>) -> FieldResult<i32> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Find all entries to be unposted
        let entries = journal_entry::Entity::find()
            .filter(journal_entry::Column::Id.is_in(ids.clone()))
            .all(db)
            .await?;

        let mut unposted_count = 0;
        for entry in entries {
            if entry.is_posted {
                let mut entry_model: journal_entry::ActiveModel = entry.into();
                entry_model.is_posted = Set(false);
                entry_model.posted_at = Set(None);
                journal_entry::Entity::update(entry_model).exec(db).await?;
                unposted_count += 1;
            }
        }

        Ok(unposted_count)
    }

    /// Delete multiple journal entries
    async fn delete_journal_entries(&self, ctx: &Context<'_>, ids: Vec<i32>) -> FieldResult<i32> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Find all entries to be deleted
        let entries = journal_entry::Entity::find()
            .filter(journal_entry::Column::Id.is_in(ids.clone()))
            .all(db)
            .await?;

        // Check if any are posted
        for entry in &entries {
            if entry.is_posted {
                return Err(format!(
                    "Cannot delete posted journal entry with number {}. Unpost it first.",
                    entry.entry_number
                )
                .into());
            }
        }

        // Delete associated entry lines first (foreign key constraint)
        entry_line::Entity::delete_many()
            .filter(entry_line::Column::JournalEntryId.is_in(ids.clone()))
            .exec(db)
            .await?;

        // Delete the journal entries
        let delete_result = journal_entry::Entity::delete_many()
            .filter(journal_entry::Column::Id.is_in(ids))
            .exec(db)
            .await?;

        Ok(delete_result.rows_affected as i32)
    }

    /// Create counterpart
    async fn create_counterpart(
        &self,
        ctx: &Context<'_>,
        input: crate::entities::counterpart::CreateCounterpartInput,
    ) -> FieldResult<counterpart::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let counterpart_model = counterpart::ActiveModel::from(input);
        let counterpart = counterpart::Entity::insert(counterpart_model)
            .exec_with_returning(db)
            .await?;

        Ok(counterpart)
    }

    /// Update counterpart
    async fn update_counterpart(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: crate::entities::counterpart::UpdateCounterpartInput,
    ) -> FieldResult<counterpart::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut counterpart_model: counterpart::ActiveModel = counterpart::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Counterpart not found"))?
            .into();

        let crate::entities::counterpart::UpdateCounterpartInput {
            name,
            eik,
            vat_number,
            street,
            address,
            city,
            postal_code,
            country,
            phone,
            email,
            contact_person,
            counterpart_type,
            is_customer,
            is_supplier,
            is_vat_registered,
            is_active,
        } = input;

        if let Some(name) = name {
            counterpart_model.name = Set(name);
        }
        if let Some(eik) = eik {
            counterpart_model.eik = Set(Some(eik));
        }
        if let Some(vat_number) = vat_number {
            counterpart_model.vat_number = Set(Some(vat_number));
        }
        if let Some(street) = street {
            counterpart_model.street = Set(Some(street));
        }
        if let Some(address) = address {
            counterpart_model.address = Set(Some(address));
        }
        if let Some(city) = city {
            counterpart_model.city = Set(Some(city));
        }
        if let Some(postal_code) = postal_code {
            counterpart_model.postal_code = Set(Some(postal_code));
        }
        if let Some(country) = country {
            counterpart_model.country = Set(Some(country));
        }
        if let Some(phone) = phone {
            counterpart_model.phone = Set(Some(phone));
        }
        if let Some(email) = email {
            counterpart_model.email = Set(Some(email));
        }
        if let Some(contact_person) = contact_person {
            counterpart_model.contact_person = Set(Some(contact_person));
        }
        if let Some(counterpart_type) = counterpart_type {
            counterpart_model.counterpart_type = Set(counterpart_type);
        }
        if let Some(is_customer) = is_customer {
            counterpart_model.is_customer = Set(is_customer);
        }
        if let Some(is_supplier) = is_supplier {
            counterpart_model.is_supplier = Set(is_supplier);
        }
        if let Some(is_vat_registered) = is_vat_registered {
            counterpart_model.is_vat_registered = Set(is_vat_registered);
        }
        if let Some(is_active) = is_active {
            counterpart_model.is_active = Set(is_active);
        }

        counterpart_model.updated_at = Set(chrono::Utc::now());

        let updated_counterpart = counterpart_model.update(db).await?;

        Ok(updated_counterpart)
    }
}
