use std::borrow::Cow;

use actix_web::{HttpRequest, cookie::Cookie};
use validator::ValidationError;

use crate::error::Error;

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

pub fn get_refresh_token_cookie(req: &HttpRequest) -> Result<Cookie<'static>, Error> {
    let Ok(cookies) = req.cookies() else {
        return Err(Error::InternalServerError("An error occurred reading cookies".into()));
    };

    let Some(cookie) = cookies.iter().find(|cookie| cookie.name() == "refreshToken") else {
        return Err(Error::Unauthorized);
    };

    Ok(cookie.clone())
}
