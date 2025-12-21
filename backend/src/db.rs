use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub async fn init_db(database_url: &str) -> anyhow::Result<DatabaseConnection> {
    let db = Database::connect(database_url).await?;

    Migrator::up(&db, None).await?;

    tracing::info!("Database connected and migrations applied");

    Ok(db)
}
