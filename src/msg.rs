// This file defines all the request/response objects
// The realworld spec requires defining two structs for most requests,
// one for the request name and another for the data.
// Which is why you see a few redundant request structs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(email(message = "invalid email"))]
    pub email: Option<String>,
    #[validate(length(
        min = 3,
        max = 16,
        message = "invalid username, must be between 3-16 characters"
    ))]
    pub username: Option<String>,
    #[validate(length(min = 6, message = "invalid password, must be at least 6 characters"))]
    pub password: Option<String>,
    #[validate(url(message = "invalid image, must be a url"))]
    pub image: Option<String>,
    #[validate(length(min = 1, message = "invalid bio, must not be empty"))]
    pub bio: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    #[serde(rename = "user")]
    pub update_user: UpdateUser,
}

#[derive(Deserialize)]
pub struct LoginUserRequest {
    #[serde(rename = "user")]
    pub login_user: LoginUser,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub user: UserJson,
}

#[derive(Serialize)]
pub struct UserJson {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Deserialize)]
pub struct NewUserRequest {
    #[serde(rename = "user")]
    pub new_user: NewUser,
}
#[derive(Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(
        min = 3,
        max = 16,
        message = "invalid username, must be between 3-16 characters"
    ))]
    pub username: String,
    #[validate(email(message = "invalid email"))]
    pub email: String,
    #[validate(length(min = 6, message = "invalid password, must be at least 6 characters"))]
    pub password: String,
}
