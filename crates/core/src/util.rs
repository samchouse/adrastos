use actix_web::HttpRequest;

use crate::handlers::Error;

pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn get_refresh_token(req: &HttpRequest) -> Result<String, Error> {
    let Ok(cookies) = req.cookies() else {
        return Err(Error::InternalServerError { error: "An error occurred reading cookies".into() });
    };

    let Some(cookie) = cookies.iter().find(|cookie| cookie.name() == "refreshToken") else {
        return Err(Error::Unauthorized);
    };

    Ok(cookie.value().into())
}
