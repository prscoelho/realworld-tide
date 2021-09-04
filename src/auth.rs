use std::future::Future;
use std::pin::Pin;

use tide::http::headers::AUTHORIZATION;
use tide::{Next, Request, Response, StatusCode};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref SECRET: String = std::env::var("JWT_KEY").expect("JWT_KEY not set.");
    static ref DECODINGKEY: DecodingKey<'static> = DecodingKey::from_secret(SECRET.as_bytes());
    static ref ENCODINGKEY: EncodingKey = EncodingKey::from_secret(SECRET.as_bytes());
    static ref VALIDATION: Validation = Validation::default();
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,
    name: String,
    exp: i64,
}

pub struct AuthId(pub i32);

pub fn auth_middleware<'a, State: Clone + Send + Sync + 'static>(
    mut request: Request<State>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
    Box::pin(async {
        match validate_request(&request) {
            Some(id) => {
                tide::log::debug!("Auth request id: {}", id);
                request.set_ext(AuthId(id));
                Ok(next.run(request).await)
            }
            None => Ok(Response::new(StatusCode::Unauthorized)),
        }
    })
}

fn validate_request<State: Clone + Send + Sync + 'static>(request: &Request<State>) -> Option<i32> {
    let header_values = request.header(AUTHORIZATION)?;
    let auth = header_values.get(0)?;
    let (name, token) = auth.as_str().split_once(' ')?;
    if name != "Bearer" && name != "Token" {
        return None;
    }

    match decode::<Claims>(token, &DECODINGKEY, &VALIDATION) {
        Ok(token_data) => Some(token_data.claims.sub),
        Err(e) => {
            tide::log::debug!("Bad token\n    jwt error: {}", e);
            None
        }
    }
}

pub fn generate_token(sub: i32, name: &str) -> String {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(60))
        .unwrap()
        .timestamp();

    let claims = Claims {
        sub,
        exp,
        name: name.into(),
    };

    encode(&Header::default(), &claims, &ENCODINGKEY).unwrap()
}

pub fn verify_password(plain_password: &str, hashed_password: &str) -> bool {
    match bcrypt::verify(plain_password, hashed_password) {
        Ok(valid) => valid,
        Err(e) => {
            tide::log::info!("Error verifying password\n    bcrypt error: {}", e);
            false
        }
    }
}

pub fn generate_password(plain_password: &str) -> anyhow::Result<String> {
    bcrypt::hash(plain_password, bcrypt::DEFAULT_COST).map_err(Into::into)
}
