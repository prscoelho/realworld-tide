use std::borrow::{Borrow, Cow};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use tide::prelude::json;
use tide::{Response, StatusCode};

#[derive(Debug)]
pub enum AppError {
    InvalidLogin,
    EmailTaken,
    UsernameTaken,
    Validator(&'static str),
}

impl AppError {
    fn text(&self) -> &'static str {
        match self {
            AppError::InvalidLogin => "invalid login credentials",
            AppError::EmailTaken => "email already taken",
            AppError::UsernameTaken => "username already taken",
            AppError::Validator(text) => *text,
        }
    }
    fn json_text(&self) -> String {
        // curly braces are escaped with double bracket
        format!("{{\"errors\":{{\"body\": [\"{}\"]}}}}", self)
    }
}

#[derive(Debug)]
pub struct AppErrors(Vec<AppError>);

impl AppErrors {
    pub fn new() -> Self {
        AppErrors(Vec::new())
    }

    pub fn add(&mut self, e: AppError) {
        self.0.push(e);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn messages(&self) -> Vec<&str> {
        self.0.iter().map(|e| e.text()).collect()
    }
}

impl Display for AppErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

impl Error for AppError {}
impl Error for AppErrors {}

pub async fn error_middleware(mut res: Response) -> tide::Result {
    if let Some(e) = res.downcast_error::<AppError>() {
        let json_text = e.json_text();
        res.set_body(json_text);
        res.set_content_type(tide::http::mime::JSON);
        res.set_status(StatusCode::UnprocessableEntity);
    } else if let Some(e) = res.downcast_error::<AppErrors>() {
        let json = json!({
            "errors": {
                "body": e.messages()
            }
        });
        res.set_body(json);
        res.set_status(StatusCode::UnprocessableEntity);
    }
    Ok(res)
}

// convert validator errors to app errors, otherwise password validation failure
// ends up being logged (with password value in plaintext) in the logging middleware
pub fn convert_error_validation(err: validator::ValidationErrors) -> AppErrors {
    let mut app_errors = AppErrors::new();
    let errors = err.field_errors();
    for (_, err) in errors.iter() {
        for e in err.iter() {
            if let Some(text) = e.message.borrow() {
                if let Cow::Borrowed(text) = text {
                    app_errors.add(AppError::Validator(text))
                }
            }
        }
    }
    app_errors
}
