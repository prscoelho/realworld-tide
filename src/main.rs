#![forbid(unsafe_code)]

#[macro_use]
extern crate lazy_static;

mod auth;
mod database;
mod error;
mod msg;

use std::sync::Arc;

use async_lock::Semaphore;
use async_std::task::spawn_blocking;
use tide::utils::After;

#[derive(Clone)]
pub struct State {
    pub db: database::Database,
    pub blocking_semaphore: Arc<Semaphore>,
}

// same constraints as `spawn_blocking`
pub async fn heavy_computation<F, T>(semaphore: &Arc<Semaphore>, f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let _lock = semaphore.acquire_arc().await;
    spawn_blocking(f).await
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    dotenv::dotenv().unwrap();

    let state = State {
        db: database::Database::new().await,
        blocking_semaphore: Arc::new(Semaphore::new(8)),
    };

    let mut app = tide::with_state(state);
    app.with(After(error::error_middleware));

    app.at("/api/users/login").post(routes::login);
    app.at("/api/users").post(routes::register_user);
    app.at("/api/user")
        .with(auth::auth_middleware)
        .get(routes::get_user)
        .put(routes::update_user);
    app.listen("127.0.0.1:8080").await?;

    Ok(())
}

mod routes {
    use tide::{Body, Request, Response, StatusCode};
    use validator::Validate;

    use crate::auth::{generate_password, verify_password, AuthId};
    use crate::error::{convert_error_validation, AppError, AppErrors};
    use crate::msg::{LoginUserRequest, NewUserRequest, UpdateUserRequest};
    use crate::{heavy_computation, State};

    pub async fn login(mut req: Request<State>) -> tide::Result<Response> {
        // 422 on failure
        let LoginUserRequest { login_user } = req.body_json().await?;

        // grab user from Database and validate password
        let user = match req.state().db.get_user_by_email(&login_user.email).await {
            Ok(user) => user,
            Err(_) => return Err(tide::Error::from(AppError::InvalidLogin)),
        };

        let hash = user.hash.clone();
        let valid_password = heavy_computation(&req.state().blocking_semaphore, move || {
            verify_password(&login_user.password, &hash)
        })
        .await;

        if !valid_password {
            return Err(tide::Error::from(AppError::InvalidLogin));
        }
        // generate token and create user response
        let user_response = user.to_user_response();
        Body::from_json(&user_response).map(Into::into)
    }

    pub async fn get_user(req: Request<State>) -> tide::Result<Body> {
        let AuthId(id) = *req.ext().unwrap();

        let user = req.state().db.get_user_by_id(id).await?;

        let user_response = user.to_user_response();
        Body::from_json(&user_response)
    }

    pub async fn register_user(mut req: Request<State>) -> tide::Result<tide::Response> {
        let NewUserRequest { new_user } = req.body_json().await?;

        new_user.validate().map_err(convert_error_validation)?;

        let db = &req.state().db;

        let mut db_errors = AppErrors::new();

        if db.contains_email(&new_user.email).await? {
            db_errors.add(AppError::EmailTaken);
        }
        if db.contains_username(&new_user.username).await? {
            db_errors.add(AppError::UsernameTaken);
        }
        if !db_errors.is_empty() {
            return Err(tide::Error::from(db_errors));
        }

        let plain_password = new_user.password.clone();
        let hash = heavy_computation(&req.state().blocking_semaphore, move || {
            generate_password(&plain_password)
        })
        .await?;

        let user = db
            .register_user(new_user.email, new_user.username, hash)
            .await?;

        let user_response = user.to_user_response();
        let mut response = Response::new(StatusCode::Created);
        let body = Body::from_json(&user_response)?;
        response.set_body(body);
        Ok(response)
    }

    pub async fn update_user(mut req: Request<State>) -> tide::Result<Response> {
        let AuthId(id) = *req.ext().unwrap();

        let UpdateUserRequest { update_user } = req.body_json().await?;

        update_user.validate().map_err(convert_error_validation)?;

        let hash = match update_user.password.as_ref() {
            Some(pass) => Some(generate_password(&pass)?),
            None => None,
        };
        let user = req.state().db.update_user(id, update_user, hash).await?;

        let user_response = user.to_user_response();
        Body::from_json(&user_response).map(Into::into)
    }
}
