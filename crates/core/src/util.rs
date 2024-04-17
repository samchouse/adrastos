use std::borrow::Cow;

use axum_extra::extract::{cookie::Cookie, CookieJar};
use validator::ValidationError;

use crate::error::Error;

#[derive(Debug)]
pub struct Cookies {
    pub is_logged_in: Cookie<'static>,
    pub refresh_token: Cookie<'static>,
}

pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn create_validation_error(code: &str, message: Option<String>) -> ValidationError {
    ValidationError {
        code: Cow::from(code.to_owned()),
        message: message.map(Cow::from),
        params: Default::default(),
    }
}

pub fn get_auth_cookies(jar: &CookieJar) -> Result<Cookies, Error> {
    let Some(is_logged_in_cookie) = jar.get("isLoggedIn") else {
        return Err(Error::Unauthorized);
    };

    let Some(refresh_token_cookie) = jar.get("refreshToken") else {
        return Err(Error::Unauthorized);
    };

    Ok(Cookies {
        is_logged_in: is_logged_in_cookie.clone(),
        refresh_token: refresh_token_cookie.clone(),
    })
}
