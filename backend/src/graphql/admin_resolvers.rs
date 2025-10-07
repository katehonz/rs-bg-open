use async_graphql::{Context, FieldResult, InputObject, Object};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Related, Set,
};
use serde::{Deserialize, Serialize};

use crate::entities::{
    company::{ActiveModel as CompanyActiveModel, Entity as CompanyEntity, Model as CompanyModel},
    user::{ActiveModel as UserActiveModel, Entity as UserEntity, Model as UserModel},
    user_company::{
        ActiveModel as UserCompanyActiveModel, CreateUserCompanyInput, Entity as UserCompanyEntity,
        Model as UserCompanyModel, UpdateUserCompanyInput, UserCompanyRole, UserCompanyWithDetails,
    },
    user_group::{Entity as UserGroupEntity, Model as UserGroupModel},
};

#[derive(InputObject)]
pub struct AdminCreateUserInput {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub group_id: i32,
    pub is_active: Option<bool>,
}

#[derive(InputObject)]
pub struct AdminUpdateUserInput {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub group_id: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(InputObject)]
pub struct AdminCreateCompanyInput {
    pub name: String,
    pub eik: String,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub manager_name: Option<String>,
    pub authorized_person: Option<String>,
    pub manager_egn: Option<String>,
    pub authorized_person_egn: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(InputObject)]
pub struct AdminUpdateCompanyInput {
    pub name: Option<String>,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub manager_name: Option<String>,
    pub authorized_person: Option<String>,
    pub manager_egn: Option<String>,
    pub authorized_person_egn: Option<String>,
    pub is_active: Option<bool>,
    pub contragent_api_url: Option<String>,
    pub contragent_api_key: Option<String>,
    pub enable_vies_validation: Option<bool>,
    pub enable_ai_mapping: Option<bool>,
    pub auto_validate_on_import: Option<bool>,
}

#[derive(Default)]
pub struct AdminQuery;

#[Object]
impl AdminQuery {
    // Users
    async fn users(&self, ctx: &Context<'_>) -> FieldResult<Vec<UserModel>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let users = UserEntity::find()
            .order_by_asc(crate::entities::user::Column::Username)
            .all(db)
            .await?;

        Ok(users)
    }

    async fn user(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Option<UserModel>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let user = UserEntity::find_by_id(id).one(db).await?;
        Ok(user)
    }

    // Companies
    async fn companies(&self, ctx: &Context<'_>) -> FieldResult<Vec<CompanyModel>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let companies = CompanyEntity::find()
            .order_by_asc(crate::entities::company::Column::Name)
            .all(db)
            .await?;

        Ok(companies)
    }

    async fn company(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Option<CompanyModel>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let company = CompanyEntity::find_by_id(id).one(db).await?;
        Ok(company)
    }

    // User-Company relationships
    async fn user_companies(&self, ctx: &Context<'_>) -> FieldResult<Vec<UserCompanyWithDetails>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let user_companies = UserCompanyEntity::find()
            .find_with_related(UserEntity)
            .all(db)
            .await?;

        let mut result = Vec::new();

        for (user_company, users) in user_companies {
            // Get company info
            let company = CompanyEntity::find_by_id(user_company.company_id)
                .one(db)
                .await?;

            result.push(UserCompanyWithDetails {
                id: user_company.id,
                user_id: user_company.user_id,
                company_id: user_company.company_id,
                role: user_company.role,
                is_active: user_company.is_active,
                created_at: user_company.created_at,
                updated_at: user_company.updated_at,
                user: users.into_iter().next(),
                company,
            });
        }

        Ok(result)
    }

    async fn users_by_company(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<Vec<UserCompanyWithDetails>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let user_companies = UserCompanyEntity::find()
            .filter(crate::entities::user_company::Column::CompanyId.eq(company_id))
            .find_with_related(UserEntity)
            .all(db)
            .await?;

        let company = CompanyEntity::find_by_id(company_id).one(db).await?;

        let mut result = Vec::new();

        for (user_company, users) in user_companies {
            result.push(UserCompanyWithDetails {
                id: user_company.id,
                user_id: user_company.user_id,
                company_id: user_company.company_id,
                role: user_company.role,
                is_active: user_company.is_active,
                created_at: user_company.created_at,
                updated_at: user_company.updated_at,
                user: users.into_iter().next(),
                company: company.clone(),
            });
        }

        Ok(result)
    }

    async fn companies_by_user(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
    ) -> FieldResult<Vec<UserCompanyWithDetails>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let user_companies = UserCompanyEntity::find()
            .filter(crate::entities::user_company::Column::UserId.eq(user_id))
            .find_with_related(CompanyEntity)
            .all(db)
            .await?;

        let user = UserEntity::find_by_id(user_id).one(db).await?;

        let mut result = Vec::new();

        for (user_company, companies) in user_companies {
            result.push(UserCompanyWithDetails {
                id: user_company.id,
                user_id: user_company.user_id,
                company_id: user_company.company_id,
                role: user_company.role,
                is_active: user_company.is_active,
                created_at: user_company.created_at,
                updated_at: user_company.updated_at,
                user: user.clone(),
                company: companies.into_iter().next(),
            });
        }

        Ok(result)
    }

    // User Groups
    async fn user_groups(&self, ctx: &Context<'_>) -> FieldResult<Vec<UserGroupModel>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let groups = UserGroupEntity::find()
            .order_by_asc(crate::entities::user_group::Column::Name)
            .all(db)
            .await?;

        Ok(groups)
    }
}

#[derive(Default)]
pub struct AdminMutation;

#[Object]
impl AdminMutation {
    // User operations
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: AdminCreateUserInput,
    ) -> FieldResult<UserModel> {
        let db = ctx.data::<DatabaseConnection>()?;

        // Hash password
        let password_hash = bcrypt::hash(input.password, bcrypt::DEFAULT_COST)?;

        let user = UserActiveModel {
            username: Set(input.username),
            email: Set(input.email),
            password_hash: Set(password_hash),
            first_name: Set(input.first_name),
            last_name: Set(input.last_name),
            group_id: Set(input.group_id),
            is_active: Set(input.is_active.unwrap_or(true)),
            // Default periods - admin can set these later
            document_period_start: Set(chrono::Utc::now().date_naive()),
            document_period_end: Set(chrono::Utc::now().date_naive()),
            document_period_active: Set(true),
            accounting_period_start: Set(chrono::Utc::now().date_naive()),
            accounting_period_end: Set(chrono::Utc::now().date_naive()),
            accounting_period_active: Set(true),
            vat_period_start: Set(chrono::Utc::now().date_naive()),
            vat_period_end: Set(chrono::Utc::now().date_naive()),
            vat_period_active: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let result = user.insert(db).await?;
        Ok(result)
    }

    async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: AdminUpdateUserInput,
    ) -> FieldResult<UserModel> {
        let db = ctx.data::<DatabaseConnection>()?;

        let user = UserEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or("User not found")?;

        let mut user: UserActiveModel = user.into();

        if let Some(username) = input.username {
            user.username = Set(username);
        }
        if let Some(email) = input.email {
            user.email = Set(email);
        }
        if let Some(first_name) = input.first_name {
            user.first_name = Set(first_name);
        }
        if let Some(last_name) = input.last_name {
            user.last_name = Set(last_name);
        }
        if let Some(group_id) = input.group_id {
            user.group_id = Set(group_id);
        }
        if let Some(is_active) = input.is_active {
            user.is_active = Set(is_active);
        }

        user.updated_at = Set(chrono::Utc::now());

        let result = user.update(db).await?;
        Ok(result)
    }

    async fn delete_user(&self, ctx: &Context<'_>, id: i32) -> FieldResult<bool> {
        let db = ctx.data::<DatabaseConnection>()?;

        let result = UserEntity::delete_by_id(id).exec(db).await?;
        Ok(result.rows_affected > 0)
    }

    // Company operations
    async fn create_company(
        &self,
        ctx: &Context<'_>,
        input: AdminCreateCompanyInput,
    ) -> FieldResult<CompanyModel> {
        let db = ctx.data::<DatabaseConnection>()?;

        let company = CompanyActiveModel {
            name: Set(input.name),
            eik: Set(input.eik),
            vat_number: Set(input.vat_number),
            address: Set(input.address),
            city: Set(input.city),
            country: Set(input.country),
            phone: Set(input.phone),
            email: Set(input.email),
            contact_person: Set(input.contact_person),
            manager_name: Set(input.manager_name),
            authorized_person: Set(input.authorized_person),
            manager_egn: Set(input.manager_egn),
            authorized_person_egn: Set(input.authorized_person_egn),
            is_active: Set(input.is_active.unwrap_or(true)),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let result = company.insert(db).await?;
        Ok(result)
    }

    async fn update_company(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: AdminUpdateCompanyInput,
    ) -> FieldResult<CompanyModel> {
        let db = ctx.data::<DatabaseConnection>()?;

        let company = CompanyEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or("Company not found")?;

        let mut company: CompanyActiveModel = company.into();

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
        if let Some(contragent_api_url) = input.contragent_api_url {
            company.contragent_api_url = Set(Some(contragent_api_url));
        }
        if let Some(contragent_api_key) = input.contragent_api_key {
            company.contragent_api_key = Set(Some(contragent_api_key));
        }
        if let Some(enable_vies_validation) = input.enable_vies_validation {
            company.enable_vies_validation = Set(enable_vies_validation);
        }
        if let Some(enable_ai_mapping) = input.enable_ai_mapping {
            company.enable_ai_mapping = Set(enable_ai_mapping);
        }
        if let Some(auto_validate_on_import) = input.auto_validate_on_import {
            company.auto_validate_on_import = Set(auto_validate_on_import);
        }

        company.updated_at = Set(chrono::Utc::now());

        let result = company.update(db).await?;
        Ok(result)
    }

    async fn delete_company(&self, ctx: &Context<'_>, id: i32) -> FieldResult<bool> {
        let db = ctx.data::<DatabaseConnection>()?;

        let result = CompanyEntity::delete_by_id(id).exec(db).await?;
        Ok(result.rows_affected > 0)
    }

    // User-Company relationship operations
    async fn assign_user_to_company(
        &self,
        ctx: &Context<'_>,
        input: CreateUserCompanyInput,
    ) -> FieldResult<UserCompanyModel> {
        let db = ctx.data::<DatabaseConnection>()?;

        let user_company = UserCompanyActiveModel {
            user_id: Set(input.user_id),
            company_id: Set(input.company_id),
            role: Set(input.role),
            is_active: Set(input.is_active.unwrap_or(true)),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let result = user_company.insert(db).await?;
        Ok(result)
    }

    async fn update_user_company(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateUserCompanyInput,
    ) -> FieldResult<UserCompanyModel> {
        let db = ctx.data::<DatabaseConnection>()?;

        let user_company = UserCompanyEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or("User-Company relationship not found")?;

        let mut user_company: UserCompanyActiveModel = user_company.into();

        if let Some(role) = input.role {
            user_company.role = Set(role);
        }
        if let Some(is_active) = input.is_active {
            user_company.is_active = Set(is_active);
        }

        user_company.updated_at = Set(chrono::Utc::now());

        let result = user_company.update(db).await?;
        Ok(result)
    }

    async fn remove_user_from_company(&self, ctx: &Context<'_>, id: i32) -> FieldResult<bool> {
        let db = ctx.data::<DatabaseConnection>()?;

        let result = UserCompanyEntity::delete_by_id(id).exec(db).await?;
        Ok(result.rows_affected > 0)
    }
}
