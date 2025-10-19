use async_graphql::dataloader::*;
use sea_orm::*;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use crate::entities::counterpart;

/// DataLoader for batching Counterpart queries
pub struct CounterpartLoader {
    pub db: Arc<DatabaseConnection>,
}

impl CounterpartLoader {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl Loader<i32> for CounterpartLoader {
    type Value = counterpart::Model;
    type Error = Arc<DbErr>;

    fn load(
        &self,
        keys: &[i32],
    ) -> impl Future<Output = Result<HashMap<i32, Self::Value>, Self::Error>> + Send {
        let db = self.db.clone();
        let keys = keys.to_vec();

        async move {
            let counterparts = counterpart::Entity::find()
                .filter(counterpart::Column::Id.is_in(keys))
                .all(db.as_ref())
                .await
                .map_err(Arc::new)?;

            Ok(counterparts
                .into_iter()
                .map(|counterpart| (counterpart.id, counterpart))
                .collect())
        }
    }
}
