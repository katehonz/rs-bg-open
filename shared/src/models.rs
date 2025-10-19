use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(async_graphql::SimpleObject))]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(async_graphql::InputObject))]
pub struct CreateUserInput {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(async_graphql::InputObject))]
pub struct UpdateUserInput {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(async_graphql::SimpleObject))]
pub struct Post {
    pub id: uuid::Uuid,
    pub title: String,
    pub content: String,
    pub author_id: uuid::Uuid,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(async_graphql::InputObject))]
pub struct CreatePostInput {
    pub title: String,
    pub content: String,
    pub author_id: uuid::Uuid,
    pub published: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(async_graphql::InputObject))]
pub struct UpdatePostInput {
    pub title: Option<String>,
    pub content: Option<String>,
    pub published: Option<bool>,
}
