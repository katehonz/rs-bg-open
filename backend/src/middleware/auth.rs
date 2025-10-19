use crate::auth::{extract_bearer_token, get_user_from_claims, validate_token, AuthenticatedUser, JwtConfig};
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use sea_orm::DatabaseConnection;
use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
};

pub struct AuthMiddleware {
    pub jwt_config: Arc<JwtConfig>,
    pub db: Arc<DatabaseConnection>,
}

impl AuthMiddleware {
    pub fn new(jwt_config: Arc<JwtConfig>, db: Arc<DatabaseConnection>) -> Self {
        Self { jwt_config, db }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            jwt_config: self.jwt_config.clone(),
            db: self.db.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    jwt_config: Arc<JwtConfig>,
    db: Arc<DatabaseConnection>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let jwt_config = self.jwt_config.clone();
        let db = self.db.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let token = match extract_bearer_token(&req) {
                Some(t) => t,
                None => {
                    let (request, _pl) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Missing or invalid Authorization header"
                        }))
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(request, response));
                }
            };

            // Validate token
            let claims = match validate_token(&token, &jwt_config) {
                Ok(c) => c,
                Err(e) => {
                    let (request, _pl) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": format!("Invalid token: {}", e)
                        }))
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(request, response));
                }
            };

            // Get user from database
            let user = match get_user_from_claims(&claims, &db).await {
                Ok(u) => u,
                Err(e) => {
                    let (request, _pl) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": format!("Authentication failed: {}", e)
                        }))
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(request, response));
                }
            };

            // Insert user into request extensions
            req.extensions_mut().insert(user);

            // Call the next service
            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

/// Extract authenticated user from request extensions (for use in handlers)
pub fn get_auth_user(req: &ServiceRequest) -> Option<AuthenticatedUser> {
    req.extensions().get::<AuthenticatedUser>().cloned()
}
