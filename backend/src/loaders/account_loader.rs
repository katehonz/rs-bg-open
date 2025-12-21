use async_graphql::dataloader::*;
use sea_orm::*;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use crate::entities::account;

/// DataLoader for batching Account queries
pub struct AccountLoader {
    pub db: Arc<DatabaseConnection>,
}

impl AccountLoader {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl Loader<i32> for AccountLoader {
    type Value = account::Model;
    type Error = Arc<DbErr>;

    fn load(
        &self,
        keys: &[i32],
    ) -> impl Future<Output = Result<HashMap<i32, Self::Value>, Self::Error>> + Send {
        let db = self.db.clone();
        let keys = keys.to_vec();

        async move {
            let accounts = account::Entity::find()
                .filter(account::Column::Id.is_in(keys))
                .all(db.as_ref())
                .await
                .map_err(Arc::new)?;

            Ok(accounts
                .into_iter()
                .map(|account| (account.id, account))
                .collect())
        }
    }
}
