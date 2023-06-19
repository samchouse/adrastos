use std::borrow::Cow;

use actix_web::{cookie::Cookie, HttpRequest};
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

pub fn get_auth_cookies(req: &HttpRequest) -> Result<Cookies, Error> {
    let Ok(cookies) = req.cookies() else {
        return Err(Error::InternalServerError("An error occurred reading cookies".into()));
    };

    let Some(is_logged_in_cookie) = cookies.iter().find(|cookie| cookie.name() == "isLoggedIn") else {
        return Err(Error::Unauthorized);
    };

    let Some(refresh_token_cookie) = cookies.iter().find(|cookie| cookie.name() == "refreshToken") else {
        return Err(Error::Unauthorized);
    };

    Ok(Cookies {
        is_logged_in: {
            let mut cookie = is_logged_in_cookie.clone();
            cookie.set_path("/");

            cookie
        },
        refresh_token: refresh_token_cookie.clone(),
    })
}
