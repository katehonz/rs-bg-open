mod auth;
mod config;
mod data;
mod db;
mod entities;
mod graphql;
mod loaders;
mod middleware;
mod rest;
mod services;

use actix_cors::Cors;
use actix_web::{middleware as actix_middleware, web, App, HttpMessage, HttpServer};
use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use crate::auth::{AuthenticatedUser, JwtConfig};
use crate::config::Config;
use crate::db::init_db;
use crate::graphql::{Mutation, Query};
use crate::loaders::{AccountLoader, CounterpartLoader};
use crate::middleware::auth::AuthMiddleware;
use crate::services::contragent::ContragentService;
use crate::services::depreciation_service::DepreciationService;
use crate::services::invoice_processing::InvoiceProcessingService;
use crate::services::maintenance::MaintenanceService;
use async_graphql::dataloader::DataLoader;

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub struct AppState {
    pub schema: AppSchema,
    pub db: Arc<DatabaseConnection>,
}

async fn graphql_handler(
    schema: web::Data<AppSchema>,
    req: GraphQLRequest,
    db: web::Data<Arc<DatabaseConnection>>,
    http_req: actix_web::HttpRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner().data(db.as_ref().as_ref().clone());

    // Add authenticated user to context if available (from middleware)
    if let Some(user) = http_req.extensions().get::<AuthenticatedUser>() {
        request = request.data(user.clone());
    }

    schema.execute(request).await.into()
}

async fn graphiql() -> actix_web::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/graphql").finish()))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let db = Arc::new(init_db(&config.database_url).await?);
    let jwt_config = Arc::new(JwtConfig::from_env());
    let contragent_service = Arc::new(ContragentService::new());
    let invoice_processing_service =
        Arc::new(InvoiceProcessingService::new(contragent_service.clone()));
    let maintenance_service = Arc::new(MaintenanceService::new(config.clone()));
    let depreciation_service = Arc::new(DepreciationService::new());

    // Create DataLoaders for batching and caching
    let account_loader = DataLoader::new(
        AccountLoader::new(db.clone()),
        tokio::spawn,
    );
    let counterpart_loader = DataLoader::new(
        CounterpartLoader::new(db.clone()),
        tokio::spawn,
    );

    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db.clone())
        .data(jwt_config.clone())
        .data(account_loader)
        .data(counterpart_loader)
        .data(contragent_service.clone())
        .data(maintenance_service.clone())
        .data(invoice_processing_service.clone())
        .data(depreciation_service.clone())
        .finish();

    tracing::info!("Starting server at http://{}:{}", config.host, config.port);
    tracing::info!(
        "GraphiQL playground: http://{}:{}/graphiql",
        config.host,
        config.port
    );

    HttpServer::new(move || {
        let auth_middleware = AuthMiddleware::new(jwt_config.clone(), db.clone());

        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(jwt_config.clone()))
            .app_data(web::Data::new(maintenance_service.clone()))
            .app_data(web::Data::new(invoice_processing_service.clone()))
            .wrap(actix_middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            // Public endpoints (no authentication required)
            .route("/health", web::get().to(rest::auth_api::health))
            .route("/api/auth/login", web::post().to(rest::auth_api::login))
            .route("/graphiql", web::get().to(graphiql))
            // Protected endpoints (authentication required)
            .service(
                web::resource("/graphql")
                    .wrap(auth_middleware)
                    .route(web::post().to(graphql_handler))
                    .route(web::get().to(graphql_handler)),
            )
            // REST API endpoints for XML/JSON file operations only
            .service(
                web::scope("/api/controlisy")
                    .route("/parse", web::post().to(rest::controlisy_api::parse_file))
                    .route("/import", web::post().to(rest::controlisy_api::import_file))
                    .route(
                        "/imports/{company_id}",
                        web::get().to(rest::controlisy_api::list_imports),
                    )
                    .route(
                        "/import/{import_id}",
                        web::get().to(rest::controlisy_api::get_import),
                    )
                    .route(
                        "/import/{import_id}",
                        web::put().to(rest::controlisy_api::update_import),
                    )
                    .route(
                        "/imports/{import_id}",
                        web::delete().to(rest::controlisy_api::delete_import),
                    )
                    .route(
                        "/process/{import_id}",
                        web::post().to(rest::controlisy_api::process_import),
                    )
                    .route(
                        "/update/{import_id}",
                        web::put().to(rest::controlisy_api::update_staged_data),
                    )
                    .route(
                        "/review/{import_id}",
                        web::post().to(rest::controlisy_api::mark_reviewed),
                    ),
            )
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await?;

    Ok(())
}
