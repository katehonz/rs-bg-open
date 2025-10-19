use crate::auth::{generate_token, JwtConfig};
use crate::entities::user::{
    AuthResponse, CreateUserInput, InputPeriods, LoginInput, UpdateInputPeriodsInput,
    UpdateUserInput, UserFilter, UserWithRole,
};
use crate::entities::{user, user_group};
use crate::graphql::context::require_can_manage_users;
use async_graphql::{Context, FieldResult, Object};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    Set,
};
use std::sync::Arc;
use validator::Validate;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// Get all users with optional filtering
    async fn users(
        &self,
        ctx: &Context<'_>,
        filter: Option<UserFilter>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<user::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = user::Entity::find();

        if let Some(f) = filter {
            let mut condition = Condition::all();

            if let Some(group_id) = f.group_id {
                condition = condition.add(user::Column::GroupId.eq(group_id));
            }

            if let Some(is_active) = f.is_active {
                condition = condition.add(user::Column::IsActive.eq(is_active));
            }

            if let Some(search_term) = f.search_term {
                let search_condition = Condition::any()
                    .add(user::Column::Username.contains(&search_term))
                    .add(user::Column::Email.contains(&search_term))
                    .add(user::Column::FirstName.contains(&search_term))
                    .add(user::Column::LastName.contains(&search_term));
                condition = condition.add(search_condition);
            }

            if let Some(has_doc_period) = f.has_document_period_active {
                condition = condition.add(user::Column::DocumentPeriodActive.eq(has_doc_period));
            }

            if let Some(has_acc_period) = f.has_accounting_period_active {
                condition = condition.add(user::Column::AccountingPeriodActive.eq(has_acc_period));
            }

            if let Some(has_vat_period) = f.has_vat_period_active {
                condition = condition.add(user::Column::VatPeriodActive.eq(has_vat_period));
            }

            query = query.filter(condition);
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let users = query.order_by_asc(user::Column::Username).all(db).await?;

        Ok(users)
    }

    /// Get user by ID
    async fn user(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Option<user::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let user = user::Entity::find_by_id(id).one(db).await?;
        Ok(user)
    }

    /// Get user by username
    async fn user_by_username(
        &self,
        ctx: &Context<'_>,
        username: String,
    ) -> FieldResult<Option<user::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let user = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await?;
        Ok(user)
    }

    /// Get user with role information
    async fn user_with_role(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<Option<UserWithRole>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let user_with_group = user::Entity::find_by_id(id)
            .find_also_related(user_group::Entity)
            .one(db)
            .await?;

        if let Some((user, Some(role))) = user_with_group {
            Ok(Some(UserWithRole {
                input_periods: InputPeriods::from(&user),
                user,
                role,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get user's input periods
    async fn user_input_periods(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
    ) -> FieldResult<Option<InputPeriods>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        if let Some(user) = user::Entity::find_by_id(user_id).one(db).await? {
            Ok(Some(InputPeriods::from(&user)))
        } else {
            Ok(None)
        }
    }

    /// Check if user can input for specific date and type
    async fn can_user_input(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        input_type: String, // "document", "accounting", "vat"
        date: chrono::NaiveDate,
    ) -> FieldResult<bool> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        if let Some(user) = user::Entity::find_by_id(user_id).one(db).await? {
            let can_input = match input_type.as_str() {
                "document" => user.can_input_document(date),
                "accounting" => user.can_input_accounting(date),
                "vat" => user.can_input_vat(date),
                _ => false,
            };
            Ok(can_input)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    /// Create a new user (admin only)
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: CreateUserInput,
    ) -> FieldResult<user::Model> {
        // Check permission: only users with can_manage_users permission can create users
        require_can_manage_users(ctx).await?;

        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Validate input
        input
            .validate()
            .map_err(|e| format!("Validation error: {:?}", e))?;

        // Check if username already exists
        let existing_user = user::Entity::find()
            .filter(user::Column::Username.eq(&input.username))
            .one(db)
            .await?;

        if existing_user.is_some() {
            return Err("Username already exists".into());
        }

        // Check if email already exists
        let existing_email = user::Entity::find()
            .filter(user::Column::Email.eq(&input.email))
            .one(db)
            .await?;

        if existing_email.is_some() {
            return Err("Email already exists".into());
        }

        let user_model = user::ActiveModel::from(input);
        let user = user::Entity::insert(user_model)
            .exec_with_returning(db)
            .await?;

        Ok(user)
    }

    /// Update user information (admin only)
    async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateUserInput,
    ) -> FieldResult<user::Model> {
        // Check permission: only users with can_manage_users permission can update users
        require_can_manage_users(ctx).await?;

        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut user: user::ActiveModel =
            if let Some(user) = user::Entity::find_by_id(id).one(db).await? {
                user.into()
            } else {
                return Err("User not found".into());
            };

        if let Some(username) = input.username {
            // Check if new username already exists
            let existing = user::Entity::find()
                .filter(user::Column::Username.eq(&username))
                .filter(user::Column::Id.ne(id))
                .one(db)
                .await?;

            if existing.is_some() {
                return Err("Username already exists".into());
            }
            user.username = Set(username);
        }

        if let Some(email) = input.email {
            // Check if new email already exists
            let existing = user::Entity::find()
                .filter(user::Column::Email.eq(&email))
                .filter(user::Column::Id.ne(id))
                .one(db)
                .await?;

            if existing.is_some() {
                return Err("Email already exists".into());
            }
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

        let updated_user = user::Entity::update(user).exec(db).await?;
        Ok(updated_user)
    }

    /// Update user's input periods (admin only)
    async fn update_user_input_periods(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        input: UpdateInputPeriodsInput,
    ) -> FieldResult<user::Model> {
        // Check permission: only users with can_manage_users permission can update periods
        require_can_manage_users(ctx).await?;

        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut user: user::ActiveModel =
            if let Some(user) = user::Entity::find_by_id(user_id).one(db).await? {
                user.into()
            } else {
                return Err("User not found".into());
            };

        // Update document periods
        if let Some(start) = input.document_period_start {
            user.document_period_start = Set(start);
        }
        if let Some(end) = input.document_period_end {
            user.document_period_end = Set(end);
        }
        if let Some(active) = input.document_period_active {
            user.document_period_active = Set(active);
        }

        // Update accounting periods
        if let Some(start) = input.accounting_period_start {
            user.accounting_period_start = Set(start);
        }
        if let Some(end) = input.accounting_period_end {
            user.accounting_period_end = Set(end);
        }
        if let Some(active) = input.accounting_period_active {
            user.accounting_period_active = Set(active);
        }

        // Update VAT periods
        if let Some(start) = input.vat_period_start {
            user.vat_period_start = Set(start);
        }
        if let Some(end) = input.vat_period_end {
            user.vat_period_end = Set(end);
        }
        if let Some(active) = input.vat_period_active {
            user.vat_period_active = Set(active);
        }

        let updated_user = user::Entity::update(user).exec(db).await?;
        Ok(updated_user)
    }

    /// Change user password (admin only)
    async fn change_user_password(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        new_password: String,
    ) -> FieldResult<bool> {
        // Check permission: only users with can_manage_users permission can change passwords
        require_can_manage_users(ctx).await?;

        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        if new_password.len() < 6 {
            return Err("Password must be at least 6 characters long".into());
        }

        let mut user: user::ActiveModel =
            if let Some(user) = user::Entity::find_by_id(user_id).one(db).await? {
                user.into()
            } else {
                return Err("User not found".into());
            };

        let password_hash = user::Model::hash_password(&new_password)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        user.password_hash = Set(password_hash);

        user::Entity::update(user).exec(db).await?;
        Ok(true)
    }

    /// Login user
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> FieldResult<AuthResponse> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let jwt_config = ctx.data::<Arc<JwtConfig>>()?;

        let user = user::Entity::find()
            .filter(user::Column::Username.eq(&input.username))
            .filter(user::Column::IsActive.eq(true))
            .one(db)
            .await?;

        if let Some(user) = user {
            let is_valid = user
                .verify_password(&input.password)
                .map_err(|e| format!("Password verification failed: {}", e))?;

            if is_valid {
                // Generate real JWT token
                let token = generate_token(&user, jwt_config.as_ref())
                    .map_err(|e| format!("Token generation failed: {}", e))?;
                let expires_at = chrono::Utc::now()
                    + chrono::Duration::hours(jwt_config.expiration_hours);

                Ok(AuthResponse {
                    user,
                    token,
                    expires_at,
                })
            } else {
                Err("Invalid credentials".into())
            }
        } else {
            Err("Invalid credentials".into())
        }
    }

    /// Deactivate user (admin only)
    async fn deactivate_user(&self, ctx: &Context<'_>, user_id: i32) -> FieldResult<bool> {
        // Check permission: only users with can_manage_users permission can deactivate users
        require_can_manage_users(ctx).await?;

        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut user: user::ActiveModel =
            if let Some(user) = user::Entity::find_by_id(user_id).one(db).await? {
                user.into()
            } else {
                return Err("User not found".into());
            };

        user.is_active = Set(false);
        user.document_period_active = Set(false);
        user.accounting_period_active = Set(false);
        user.vat_period_active = Set(false);

        user::Entity::update(user).exec(db).await?;
        Ok(true)
    }
}
