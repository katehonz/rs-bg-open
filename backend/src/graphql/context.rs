use crate::auth::AuthenticatedUser;
use async_graphql::{Context, Error, ErrorExtensions, Result};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// Extract authenticated user from GraphQL context
pub fn get_current_user<'a>(ctx: &'a Context<'_>) -> Result<&'a AuthenticatedUser> {
    ctx.data::<AuthenticatedUser>()
        .map_err(|_| Error::new("Unauthorized").extend_with(|_, e| e.set("code", "UNAUTHORIZED")))
}

/// Check if current user can manage users
pub async fn require_can_manage_users<'a>(ctx: &'a Context<'_>) -> Result<&'a AuthenticatedUser> {
    use crate::auth::user_can_manage_users;
    use crate::entities::user_group;
    use sea_orm::EntityTrait;

    let user = get_current_user(ctx)?;
    let db = ctx.data::<Arc<DatabaseConnection>>()?;

    let group = user_group::Entity::find_by_id(user.group_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| Error::new("User group not found"))?;

    if !group.can_manage_users {
        return Err(Error::new("Permission denied: cannot manage users")
            .extend_with(|_, e| e.set("code", "FORBIDDEN")));
    }

    Ok(user)
}

/// Check if current user can create companies
pub async fn require_can_create_companies<'a>(ctx: &'a Context<'_>) -> Result<&'a AuthenticatedUser> {
    use crate::entities::user_group;
    use sea_orm::EntityTrait;

    let user = get_current_user(ctx)?;
    let db = ctx.data::<Arc<DatabaseConnection>>()?;

    let group = user_group::Entity::find_by_id(user.group_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| Error::new("User group not found"))?;

    if !group.can_create_companies {
        return Err(Error::new("Permission denied: cannot create companies")
            .extend_with(|_, e| e.set("code", "FORBIDDEN")));
    }

    Ok(user)
}

/// Check if current user can edit companies
pub async fn require_can_edit_companies<'a>(ctx: &'a Context<'_>) -> Result<&'a AuthenticatedUser> {
    use crate::entities::user_group;
    use sea_orm::EntityTrait;

    let user = get_current_user(ctx)?;
    let db = ctx.data::<Arc<DatabaseConnection>>()?;

    let group = user_group::Entity::find_by_id(user.group_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| Error::new("User group not found"))?;

    if !group.can_edit_companies {
        return Err(Error::new("Permission denied: cannot edit companies")
            .extend_with(|_, e| e.set("code", "FORBIDDEN")));
    }

    Ok(user)
}

/// Check if current user can post accounting entries
pub async fn require_can_post_entries<'a>(ctx: &'a Context<'_>) -> Result<&'a AuthenticatedUser> {
    use crate::entities::user_group;
    use sea_orm::EntityTrait;

    let user = get_current_user(ctx)?;
    let db = ctx.data::<Arc<DatabaseConnection>>()?;

    let group = user_group::Entity::find_by_id(user.group_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| Error::new("User group not found"))?;

    if !group.can_post_entries {
        return Err(Error::new("Permission denied: cannot post entries")
            .extend_with(|_, e| e.set("code", "FORBIDDEN")));
    }

    Ok(user)
}

/// Check if current user has access to a specific company
pub async fn require_company_access<'a>(
    ctx: &'a Context<'_>,
    company_id: i32,
) -> Result<&'a AuthenticatedUser> {
    use crate::auth::user_can_access_company;

    let user = get_current_user(ctx)?;
    let db = ctx.data::<Arc<DatabaseConnection>>()?;

    user_can_access_company(user.id, company_id, db.as_ref())
        .await
        .map_err(|e| Error::new(e).extend_with(|_, ext| ext.set("code", "FORBIDDEN")))?;

    Ok(user)
}

/// Check if current user is admin for a specific company
pub async fn require_company_admin<'a>(
    ctx: &'a Context<'_>,
    company_id: i32,
) -> Result<&'a AuthenticatedUser> {
    use crate::auth::user_is_company_admin;

    let user = get_current_user(ctx)?;
    let db = ctx.data::<Arc<DatabaseConnection>>()?;

    let is_admin = user_is_company_admin(user.id, company_id, db.as_ref())
        .await
        .map_err(|e| Error::new(e))?;

    if !is_admin {
        return Err(
            Error::new("Permission denied: must be company admin")
                .extend_with(|_, e| e.set("code", "FORBIDDEN")),
        );
    }

    Ok(user)
}
